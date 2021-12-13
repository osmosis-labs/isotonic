#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TransferableAmountResponse};
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
        name: "Lent ".to_owned() + &msg.name,
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
        name: "Borrowed ".to_owned() + &msg.name,
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

    let mut response = Response::new();

    let addr = deps.api.addr_validate(&res.contract_address)?;
    if id == LTOKEN_INIT_REPLY_ID {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.ltoken_contract = addr.clone();
            response = Response::new().add_attribute("ltoken", addr);
            Ok(config)
        })?
    } else {
        CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
            config.btoken_contract = addr.clone();
            response = Response::new().add_attribute("btoken", addr);
            Ok(config)
        })?
    };

    Ok(response)
}

/// Helper that determines if an address can withdraw the specified amount.
fn can_withdraw(_deps: Deps, _sender: &Addr, _amount: Uint128) -> Result<(), ContractError> {
    // TODO: actual checks here
    Ok(())
}

/// Validates funds sent with the message, that they contain only the base asset. Returns
/// amount of funds sent, or error if:
/// * No funds were passed with the message (`NoFundsSent` error)
/// * Multiple denoms were sent (`ExtraDenoms` error)
/// * A single denom different than cfg.base_asset was sent (`InvalidDenom` error)
pub fn validate_funds(funds: &[Coin], base_asset_denom: &str) -> Result<Uint128, ContractError> {
    match funds {
        [] => Err(ContractError::NoFundsSent {}),
        [Coin { denom, amount }] if denom == base_asset_denom => Ok(*amount),
        [_] => Err(ContractError::InvalidDenom(base_asset_denom.to_string())),
        _ => Err(ContractError::ExtraDenoms(base_asset_denom.to_string())),
    }
}

/// Handler for `ExecuteMsg::Deposit`
pub fn deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let funds_sent = validate_funds(&info.funds, &cfg.base_asset)?;

    let mint_msg = to_binary(&lendex_token::msg::ExecuteMsg::Mint {
        recipient: info.sender.to_string(),
        amount: lendex_token::DisplayAmount::raw(funds_sent),
    })?;
    let wrapped_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: cfg.ltoken_contract.to_string(),
        msg: mint_msg,
        funds: vec![],
    });

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender)
        .add_submessage(wrapped_msg))
}

/// Handler for `ExecuteMsg::Withdraw`
pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    can_withdraw(deps.as_ref(), &info.sender, amount)?;

    // Burn the L tokens
    let burn_msg = to_binary(&lendex_token::msg::ExecuteMsg::BurnFrom {
        owner: info.sender.to_string(),
        amount: lendex_token::DisplayAmount::raw(amount),
    })?;
    let wrapped_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: cfg.ltoken_contract.to_string(),
        msg: burn_msg,
        funds: vec![],
    });

    // Send the base assets from contract to lender
    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![coin(amount.u128(), cfg.base_asset)],
    });

    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("sender", info.sender)
        .add_submessage(wrapped_msg)
        .add_message(send_msg))
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
        ExecuteMsg::Deposit {} => deposit(deps, info),
        ExecuteMsg::Withdraw { amount } => withdraw(deps, info, amount),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?),
        TransferableAmount { token, account } => {
            let token = deps.api.addr_validate(&token)?;
            to_binary(&query::transferable_amount(deps, token, account)?)
        }
    }
}

mod query {
    use super::*;

    use cosmwasm_std::{StdError, Uint128};
    use cw20::BalanceResponse;
    use lendex_token::msg::QueryMsg;

    pub fn transferable_amount(
        deps: Deps,
        token: Addr,
        account: String,
    ) -> StdResult<TransferableAmountResponse> {
        let config = CONFIG.load(deps.storage)?;
        if token == config.btoken_contract {
            Ok(TransferableAmountResponse {
                transferable: Uint128::zero(),
            })
        } else if token == config.ltoken_contract {
            let resp: BalanceResponse = deps
                .querier
                .query_wasm_smart(&token, &QueryMsg::Balance { address: account })?;

            Ok(TransferableAmountResponse {
                transferable: resp.balance,
            })
        } else {
            Err(StdError::generic_err(format!(
                "Unrecognized token: {}",
                token.to_string()
            )))
        }
    }
}
