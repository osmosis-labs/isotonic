#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg,
    WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInstantiateResponse};
use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lendex-market";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const LTOKEN_INIT_REPLY_ID: u64 = 1;
const BTOKEN_INIT_REPLY_ID: u64 = 2;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let ltoken_msg = lendex_token::msg::InstantiateMsg {
        name: "Lent".to_owned() + &msg.name,
        symbol: "L".to_owned() + &msg.symbol,
        decimals: msg.decimals,
        controller: env.contract.address.to_string(),
    };
    let ltoken_instantiate = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.token_id,
        msg: to_binary(&ltoken_msg)?,
        funds: vec![],
        label: format!("ltoken_contract_{}", env.contract.address),
    };
    let btoken_msg = lendex_token::msg::InstantiateMsg {
        name: "Borrowed".to_owned() + &msg.name,
        symbol: "B".to_owned() + &msg.symbol,
        decimals: msg.decimals,
        controller: env.contract.address.to_string(),
    };
    let btoken_instantiate = WasmMsg::Instantiate {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.token_id,
        msg: to_binary(&btoken_msg)?,
        funds: vec![],
        label: format!("btoken_contract_{}", env.contract.address),
    };

    let cfg = Config {
        // those will be overwritten in a response
        ltoken_contract: Addr::unchecked(""),
        btoken_contract: Addr::unchecked(""),
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        token_id: msg.token_id,
        base_asset: msg.base_asset,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_submessage(SubMsg::reply_on_success(
            ltoken_instantiate,
            LTOKEN_INIT_REPLY_ID,
        ))
        .add_submessage(SubMsg::reply_on_success(
            btoken_instantiate,
            BTOKEN_INIT_REPLY_ID,
        )))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        LTOKEN_INIT_REPLY_ID | BTOKEN_INIT_REPLY_ID => token_instantiate_reply(deps, env, msg),
        _ => Err(ContractError::UnrecognisedReply(msg.id)),
    }
}

pub fn token_instantiate_reply(
    deps: DepsMut,
    _env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    let id = msg.id;
    let res =
        parse_reply_instantiate_data(msg).map_err(|err| ContractError::ReplyParseFailure {
            id,
            err: err.to_string(),
        })?;

    let addr = deps.api.addr_validate(&res.contract_address)?;
    if id == LTOKEN_INIT_REPLY_ID {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.ltoken_contract = addr.clone();
            Ok(config)
        })?
    } else {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.btoken_contract = addr.clone();
            Ok(config)
        })?
    };

    let data = TokenInstantiateResponse {
        token_contract: addr,
    };

    let resp = Response::new().set_data(to_binary(&data)?);

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?),
    }
}
