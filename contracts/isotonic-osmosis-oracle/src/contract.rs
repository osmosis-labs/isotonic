#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Env, MessageInfo};
use cw2::set_contract_version;
use osmo_bindings::{OsmosisMsg, OsmosisQuery};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:isotonic-osmosis-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type Response = cosmwasm_std::Response<OsmosisMsg>;
type Deps<'a> = cosmwasm_std::Deps<'a, OsmosisQuery>;
type DepsMut<'a> = cosmwasm_std::DepsMut<'a, OsmosisQuery>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        controller: deps.api.addr_validate(&msg.controller)?,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Execution entry point
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;
    match msg {
        RegisterPool {
            pool_id,
            denom1,
            denom2,
        } => exec::register_pool(deps, info, pool_id, &denom1, &denom2),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    let res = match msg {
        Price { sell, buy } => to_binary(&query::price(deps, env, sell, buy)?)?,
    };

    Ok(res)
}

mod exec {
    use cosmwasm_std::ensure_eq;

    use crate::state::POOLS;

    use super::*;

    pub fn register_pool(
        deps: DepsMut,
        info: MessageInfo,
        pool_id: u64,
        denom1: &str,
        denom2: &str,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        ensure_eq!(info.sender, cfg.controller, ContractError::Unauthorized {});

        let denoms = if denom1 < denom2 {
            (denom1, denom2)
        } else {
            (denom2, denom1)
        };
        POOLS.save(deps.storage, denoms, &pool_id)?;

        Ok(Response::new()
            .add_attribute("action", "set_price")
            .add_attribute("sender", info.sender))
    }
}

mod query {
    use cosmwasm_std::QueryRequest;
    use osmo_bindings::{SpotPriceResponse, Swap};

    use crate::{msg::PriceResponse, state::POOLS};

    use super::*;

    pub fn price(
        deps: Deps,
        _env: Env,
        sell: String,
        buy: String,
    ) -> Result<PriceResponse, ContractError> {
        let denoms = if sell < buy {
            (sell.as_str(), buy.as_str())
        } else {
            (buy.as_str(), sell.as_str())
        };

        let pool_id = POOLS
            .may_load(deps.storage, denoms)?
            .ok_or(ContractError::NoInfo {})?;

        let price: SpotPriceResponse =
            deps.querier
                .query(&QueryRequest::Custom(OsmosisQuery::SpotPrice {
                    swap: Swap {
                        pool_id,
                        denom_in: sell,
                        denom_out: buy,
                    },
                    with_swap_fee: false,
                }))?;

        Ok(PriceResponse { rate: price.price })
    }
}

#[cfg(test)]
mod tests {
    use crate::state::POOLS;

    use super::*;

    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        Order,
    };

    mod helpers {
        use super::*;

        use std::collections::HashMap;

        use cosmwasm_std::{
            testing::{MockApi, MockQuerier, MockStorage},
            OwnedDeps,
        };

        pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, MockQuerier, OsmosisQuery> {
            OwnedDeps {
                storage: MockStorage::default(),
                api: MockApi::default(),
                querier: MockQuerier::default(),
                custom_query_type: std::marker::PhantomData,
            }
        }

        pub fn instantiate_contract(deps: DepsMut, admin: &str) {
            instantiate(
                deps,
                mock_env(),
                mock_info(admin, &[]),
                InstantiateMsg {
                    controller: "admin".to_string(),
                },
            )
            .unwrap();
        }

        pub fn list_pools(deps: Deps) -> HashMap<(String, String), u64> {
            POOLS
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<Result<_, _>>()
                .unwrap()
        }
    }

    #[test]
    fn register_pool() {
        let mut deps = helpers::mock_dependencies();
        helpers::instantiate_contract(deps.as_mut(), "admin");

        assert!(helpers::list_pools(deps.as_ref()).is_empty());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::RegisterPool {
                pool_id: 2,
                denom1: "ATOM".to_string(),
                denom2: "OSMO".to_string(),
            },
        )
        .unwrap();

        let pools = helpers::list_pools(deps.as_ref());
        assert_eq!(pools.len(), 1);
        assert_eq!(
            pools.get(&("ATOM".to_string(), "OSMO".to_string())),
            Some(&2)
        );
    }

    #[test]
    fn register_pool_reorders_denoms() {
        let mut deps = helpers::mock_dependencies();
        helpers::instantiate_contract(deps.as_mut(), "admin");

        assert!(helpers::list_pools(deps.as_ref()).is_empty());

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::RegisterPool {
                pool_id: 2,
                denom1: "OSMO".to_string(),
                denom2: "ATOM".to_string(),
            },
        )
        .unwrap();

        let pools = helpers::list_pools(deps.as_ref());
        assert_eq!(pools.len(), 1);
        assert_eq!(
            pools.get(&("ATOM".to_string(), "OSMO".to_string())),
            Some(&2)
        );
    }

    #[test]
    fn register_pool_unauthorized() {
        let mut deps = helpers::mock_dependencies();
        helpers::instantiate_contract(deps.as_mut(), "admin");

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("eve", &[]),
            ExecuteMsg::RegisterPool {
                pool_id: 2,
                denom1: "OSMO".to_string(),
                denom2: "ATOM".to_string(),
            },
        );

        assert_eq!(Err(ContractError::Unauthorized {}), res);
    }
}
