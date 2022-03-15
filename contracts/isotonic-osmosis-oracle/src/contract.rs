#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Env, MessageInfo};
use cw2::set_contract_version;
use osmo_bindings::{OsmosisMsg, OsmosisQuery};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:isotonic-oracle";
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
    match msg {
        ExecuteMsg::AddPool {
            pool_id,
            denom1,
            denom2,
        } => exec::add_pool(deps, info, pool_id, &denom1, &denom2),
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

    /// Handler for `ExecuteMsg::SetPrice`
    pub fn add_pool(
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
                    with_swap_fee: true,
                }))?;

        Ok(PriceResponse { rate: price.price })
    }
}
