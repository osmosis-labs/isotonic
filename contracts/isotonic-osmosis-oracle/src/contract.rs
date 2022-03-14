#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::QueryMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:isotonic-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: Empty,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Execution entry point
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, ContractError> {
    Err(ContractError::NoExecute {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    let res = match msg {
        Price { sell, buy } => to_binary(&query::price(deps, env, &sell, &buy)?)?,
    };

    Ok(res)
}

mod query {
    use crate::msg::PriceResponse;

    use super::*;

    pub fn price(
        _deps: Deps,
        _env: Env,
        _sell: &str,
        _buy: &str,
    ) -> Result<PriceResponse, ContractError> {
        todo!()
    }
}
