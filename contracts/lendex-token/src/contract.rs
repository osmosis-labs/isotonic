#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    BalanceResponse, ControllerQuery, Cw20ReceiveMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    TokenInfoResponse, TransferableAmountResp,
};
use crate::state::{TokenInfo, BALANCES, CONTROLLER, TOKEN_INFO, TOTAL_SUPPLY};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lendex-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let token_info = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
    };
    TOKEN_INFO.save(deps.storage, &token_info)?;
    TOTAL_SUPPLY.save(deps.storage, &Uint128::zero())?;
    CONTROLLER.save(deps.storage, &deps.api.addr_validate(&msg.controller)?)?;

    Ok(Response::new())
}

/// Ensures, that tokens can be transferred from given account
fn can_transfer(
    deps: Deps,
    env: &Env,
    account: String,
    amount: Uint128,
) -> Result<(), ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    let transferable: TransferableAmountResp = deps.querier.query_wasm_smart(
        controller,
        &ControllerQuery::TransferableAmount {
            token: env.contract.address.to_string(),
            account,
        },
    )?;

    if amount <= transferable.transferable {
        Ok(())
    } else {
        Err(ContractError::CannotTransfer {
            max_transferable: transferable.transferable,
        })
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
        |balance: Option<Uint128>| -> Result<_, ContractError> {
            let balance = balance.unwrap_or_default();
            balance
                .checked_sub(amount)
                .map_err(|_| ContractError::InsufficientTokens {
                    available: balance,
                    needed: amount,
                })
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

/// Handler for `ExecuteMsg::Send`
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

/// Handler for `ExecuteMsg::Mint`
pub fn mint(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    if info.sender != controller {
        return Err(ContractError::Unauthorized {});
    }

    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let recipient_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &recipient_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    TOTAL_SUPPLY.update(deps.storage, |supply| -> StdResult<_> {
        Ok(supply + amount)
    })?;

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

/// Handler for `ExecuteMsg::Burn`
pub fn burn(deps: DepsMut, info: MessageInfo, amount: Uint128) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    if info.sender != controller {
        return Err(ContractError::Unauthorized {});
    }

    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> Result<_, ContractError> {
            let balance = balance.unwrap_or_default();
            balance
                .checked_sub(amount)
                .map_err(|_| ContractError::InsufficientTokens {
                    available: balance,
                    needed: amount,
                })
        },
    )?;

    TOTAL_SUPPLY.update(deps.storage, |supply| -> Result<_, ContractError> {
        supply
            .checked_sub(amount)
            .map_err(|_| ContractError::InsufficientTokens {
                available: supply,
                needed: amount,
            })
    })?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender.to_string())
        .add_attribute("amount", amount);
    Ok(res)
}

/// Execution entry point
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
        Mint { recipient, amount } => mint(deps, info, recipient, amount),
        Burn { amount } => burn(deps, info, amount),
    }
}

/// Handler for `QueryMsg::Balance`
pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}

/// Handler for `QueryMsg::TokenInfo`
pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let token_info = TOKEN_INFO.load(deps.storage)?;
    let total_supply = TOTAL_SUPPLY.load(deps.storage)?;

    Ok(TokenInfoResponse {
        name: token_info.name,
        symbol: token_info.symbol,
        decimals: token_info.decimals,
        total_supply,
    })
}

/// `QueryMsg` entry point
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Balance { address } => to_binary(&query_balance(deps, address)?),
        TokenInfo {} => to_binary(&query_token_info(deps)?),
    }
}
