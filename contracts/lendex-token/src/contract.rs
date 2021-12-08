#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{CanTransferResp, ControllerQuery, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{TokenInfo, BALANCES, TOKEN_INFO, TOTAL_SUPPLY};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lendex-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        controller: deps.api.addr_validate(&msg.controller)?,
    };
    TOKEN_INFO.save(deps.storage, &token_info)?;
    TOTAL_SUPPLY.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new())
}

/// Ensures, that tokens can be transferred from given account
fn can_transfer(
    deps: Deps,
    env: &Env,
    account: String,
    amount: Uint128,
) -> Result<(), ContractError> {
    let token_info = TOKEN_INFO.load(deps.storage)?;
    let can_transfer: CanTransferResp = deps.querier.query_wasm_smart(
        token_info.controller,
        &ControllerQuery::CanTransfer {
            token: env.contract.address.to_string(),
            account,
            amount,
        },
    )?;

    match can_transfer {
        CanTransferResp::None => Err(ContractError::CannotTransfer {
            max_transferable: Uint128::zero(),
        }),
        CanTransferResp::Partial(max_transferable) => {
            Err(ContractError::CannotTransfer { max_transferable })
        }
        CanTransferResp::Whole => Ok(()),
    }
}

/// Performs tokens transfer.
fn transfer_tokens(
    deps: DepsMut,
    env: Env,
    sender: String,
    recipient: Addr,
    amount: Uint128,
) -> Result<(), ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // This can be unchecked, as if it is invalid, the controller would refuse transfer.
    // Converting before `can_transfer` check to avoid obsolete string clone.
    let sender_addr = Addr::unchecked(&sender);
    can_transfer(deps.as_ref(), &env, sender, amount)?;

    BALANCES.update(
        deps.storage,
        &sender_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            balance
                .unwrap_or_default()
                .checked_sub(amount)
                .map_err(Into::into)
        },
    )?;

    BALANCES.update(
        deps.storage,
        &recipient,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    Ok(())
}

/// Handler for `ExecuteMsg::Transfer`
fn transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let recipient_addr = deps.api.addr_validate(&recipient)?;
    transfer_tokens(deps, env, info.sender.to_string(), recipient_addr, amount)?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);

    Ok(res)
}

fn send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let recipient_addr = deps.api.addr_validate(&recipient)?;
    transfer_tokens(deps, env, info.sender.to_string(), recipient_addr, amount)?;

    let res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", &info.sender)
        .add_attribute("to", &recipient)
        .add_attribute("amount", amount)
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.into(),
                amount,
                msg,
            }
            .into_cosmos_msg(recipient)?,
        );

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        Transfer { recipient, amount } => transfer(deps, env, info, recipient, amount),
        Send {
            contract,
            amount,
            msg,
        } => send(deps, env, info, contract, amount, msg),
        _ => todo!(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    todo!()
}

#[cfg(test)]
mod tests {}
