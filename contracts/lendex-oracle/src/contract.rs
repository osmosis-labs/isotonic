#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lendex-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        oracle: deps.api.addr_validate(&msg.oracle)?,
        maximum_age: msg.maximum_age,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handler for `ExecuteMsg::SetPrice`
pub fn set_price(
    _deps: DepsMut,
    _info: MessageInfo,
    _sell: String,
    _buy: String,
    _rate: Decimal,
) -> Result<Response, ContractError> {
    todo!()
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
        ExecuteMsg::SetPrice { sell, buy, rate } => set_price(deps, info, sell, buy, rate),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?),
        Price { sell, buy } => to_binary(
            &query::price(deps, &sell, &buy)
                .map_err(|err| StdError::generic_err(err.to_string()))?,
        ),
    }
}

mod query {
    use crate::msg::PriceResponse;

    use super::*;

    pub fn price(_deps: Deps, _sell: &str, _buy: &str) -> Result<PriceResponse, ContractError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_slice,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, StdError,
    };

    use crate::{msg::PriceResponse, time::Duration};

    use super::*;

    fn do_instantiate(deps: DepsMut, oracle: &str, maximum_age: u64) {
        let msg = InstantiateMsg {
            oracle: oracle.to_string(),
            maximum_age: Duration::new(maximum_age),
        };
        let info = mock_info("creator", &[]);
        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    fn set_price(
        deps: DepsMut,
        setter: &str,
        sell: &str,
        buy: &str,
        rate: Decimal,
    ) -> Result<Response, ContractError> {
        execute(
            deps,
            mock_env(),
            mock_info(setter, &[]),
            ExecuteMsg::SetPrice {
                sell: sell.to_string(),
                buy: buy.to_string(),
                rate,
            },
        )
    }

    fn query_price(deps: Deps, env: Env, sell: &str, buy: &str) -> Result<PriceResponse, StdError> {
        let raw = query(
            deps,
            env,
            QueryMsg::Price {
                sell: sell.to_string(),
                buy: buy.to_string(),
            },
        )
        .unwrap();
        from_slice(&raw)
    }

    fn mock_env_after_secs(secs: u64) -> Env {
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(secs);
        env
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        do_instantiate(deps.as_mut(), "oracle", 333);

        let raw = query(deps.as_ref(), mock_env(), QueryMsg::Configuration {}).unwrap();
        let res: Config = from_slice(&raw).unwrap();
        assert_eq!(
            res,
            Config {
                oracle: Addr::unchecked("oracle"),
                maximum_age: Duration::new(333),
            }
        );
    }

    #[test]
    #[ignore]
    fn set_and_query_price() {
        let mut deps = mock_dependencies();
        do_instantiate(deps.as_mut(), "oracle", 333);

        // The oracle can set the price and we can later query it.
        set_price(deps.as_mut(), "oracle", "ATOM", "BTC", Decimal::percent(60)).unwrap();

        let res = query_price(deps.as_ref(), mock_env(), "ATOM", "BTC").unwrap();
        assert_eq!(
            res,
            PriceResponse {
                rate: Decimal::percent(60),
            }
        );
    }

    #[test]
    #[ignore]
    fn set_price_unauthorized() {
        let mut deps = mock_dependencies();
        do_instantiate(deps.as_mut(), "oracle", 333);

        // "some_dude" isn't the address of the oracle here, so we throw
        // an `Unauthorized` error
        let resp = set_price(
            deps.as_mut(),
            "some_dude",
            "ATOM",
            "BTC",
            Decimal::percent(60),
        );
        assert_eq!(resp, Err(ContractError::Unauthorized {}));
    }

    #[test]
    #[ignore]
    fn query_outdated_price() {
        let mut deps = mock_dependencies();
        do_instantiate(deps.as_mut(), "oracle", 333);

        // The oracle can set the price and we can later query it.
        set_price(deps.as_mut(), "oracle", "ATOM", "BTC", Decimal::percent(60)).unwrap();

        // Query after the last record already expired.
        let res = query_price(deps.as_ref(), mock_env_after_secs(355), "ATOM", "BTC").unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err("The prices for this trading pair are outdated")
        );
    }

    #[test]
    #[ignore]
    fn query_nonexistent_price() {
        let mut deps = mock_dependencies();
        do_instantiate(deps.as_mut(), "oracle", 333);

        // Query a trading pair that was never recorded
        let res = query_price(deps.as_ref(), mock_env(), "ATOM", "BTC").unwrap_err();
        assert_eq!(
            res,
            StdError::generic_err("There is no info about the prices for this trading pair")
        );
    }
}
