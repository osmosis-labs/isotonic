#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
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
        Price { sell, buy } => to_binary(&query::price(deps, &sell, &buy)?),
    }
}

mod query {
    use crate::msg::PriceResponse;

    use super::*;

    pub fn price(_deps: Deps, _sell: &str, _buy: &str) -> StdResult<PriceResponse> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        from_slice,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };

    use crate::time::Duration;

    use super::*;

    fn do_instantiate(deps: DepsMut, oracle: &str, maximum_age: u64) {
        let msg = InstantiateMsg {
            oracle: oracle.to_string(),
            maximum_age: Duration::new(maximum_age),
        };
        let info = mock_info("creator", &[]);
        instantiate(deps, mock_env(), info, msg).unwrap();
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
}
