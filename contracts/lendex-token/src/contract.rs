#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::display_amount::DisplayAmount;
use crate::error::ContractError;
use crate::msg::{
    BalanceResponse, ControllerQuery, Cw20ReceiveMsg, ExecuteMsg, InstantiateMsg,
    MultiplierResponse, QueryMsg, TokenInfoResponse, TransferableAmountResp,
};
use crate::state::{TokenInfo, BALANCES, CONTROLLER, MULTIPLIER, TOKEN_INFO, TOTAL_SUPPLY};

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
    MULTIPLIER.save(deps.storage, &Decimal::one())?;

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
                .map_err(|_| ContractError::insufficient_tokens(balance, amount))
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
    amount: DisplayAmount,
) -> Result<Response, ContractError> {
    let multiplier = MULTIPLIER.load(deps.storage)?;
    let amount = amount.to_stored_amount(multiplier);

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
    amount: DisplayAmount,
    msg: Binary,
) -> Result<Response, ContractError> {
    let multiplier = MULTIPLIER.load(deps.storage)?;
    let amount = amount.to_stored_amount(multiplier);

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
    amount: DisplayAmount,
) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    let multiplier = MULTIPLIER.load(deps.storage)?;
    let amount = amount.to_stored_amount(multiplier);

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
pub fn burn(
    deps: DepsMut,
    info: MessageInfo,
    account: String,
    amount: DisplayAmount,
) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    let multiplier = MULTIPLIER.load(deps.storage)?;
    let account = deps.api.addr_validate(&account)?;
    let amount = amount.to_stored_amount(multiplier);

    if info.sender != controller {
        return Err(ContractError::Unauthorized {});
    }

    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    BALANCES.update(
        deps.storage,
        &account,
        |balance: Option<Uint128>| -> Result<_, ContractError> {
            let balance = balance.unwrap_or_default();
            balance
                .checked_sub(amount)
                .map_err(|_| ContractError::insufficient_tokens(balance, amount))
        },
    )?;

    TOTAL_SUPPLY.update(deps.storage, |supply| -> Result<_, ContractError> {
        supply
            .checked_sub(amount)
            .map_err(|_| ContractError::insufficient_tokens(supply, amount))
    })?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender.to_string())
        .add_attribute("account", account)
        .add_attribute("amount", amount);
    Ok(res)
}

/// Handler for `ExecuteMsg::Rebase`
pub fn rebase(deps: DepsMut, info: MessageInfo, ratio: Decimal) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    if info.sender != controller {
        return Err(ContractError::Unauthorized {});
    }

    MULTIPLIER.update(deps.storage, |multiplier: Decimal| -> StdResult<_> {
        Ok(multiplier * ratio)
    })?;

    let res = Response::new()
        .add_attribute("action", "rebase")
        .add_attribute("ratio", ratio.to_string());

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
        Burn { account, amount } => burn(deps, info, account, amount),
        Rebase { ratio } => rebase(deps, info, ratio),
    }
}

/// Handler for `QueryMsg::Balance`
pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let multiplier = MULTIPLIER.load(deps.storage)?;

    let address = deps.api.addr_validate(&address)?;
    let stored_balance = BALANCES
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    let balance = DisplayAmount::from_stored_amount(multiplier, stored_balance);
    Ok(BalanceResponse { balance })
}

/// Handler for `QueryMsg::TokenInfo`
pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let multiplier = MULTIPLIER.load(deps.storage)?;
    let token_info = TOKEN_INFO.load(deps.storage)?;
    let total_supply =
        DisplayAmount::from_stored_amount(multiplier, TOTAL_SUPPLY.load(deps.storage)?);

    Ok(TokenInfoResponse {
        name: token_info.name,
        symbol: token_info.symbol,
        decimals: token_info.decimals,
        total_supply,
    })
}

/// Handler for `QueryMsg::Multiplier`
pub fn query_multiplier(deps: Deps) -> StdResult<MultiplierResponse> {
    let multiplier = MULTIPLIER.load(deps.storage)?;

    Ok(MultiplierResponse { multiplier })
}

/// `QueryMsg` entry point
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Balance { address } => to_binary(&query_balance(deps, address)?),
        TokenInfo {} => to_binary(&query_token_info(deps)?),
        Multiplier {} => to_binary(&query_multiplier(deps)?),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    use super::*;

    #[test]
    fn rebase_works() {
        let mut deps = mock_dependencies();
        let controller = "controller";
        let instantiate_msg = InstantiateMsg {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            controller: controller.to_string(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Multiplier is 1.0 at first
        assert_eq!(Decimal::one(), MULTIPLIER.load(&deps.storage).unwrap());

        // We rebase by 1.2, multiplier is 1.2
        let info = mock_info(controller, &[]);
        rebase(deps.as_mut(), info.clone(), Decimal::percent(120)).unwrap();
        assert_eq!(
            Decimal::percent(120),
            MULTIPLIER.load(&deps.storage).unwrap()
        );

        // We rebase by 1.2, multiplier is 1.44
        rebase(deps.as_mut(), info, Decimal::percent(120)).unwrap();
        assert_eq!(
            Decimal::percent(144),
            MULTIPLIER.load(&deps.storage).unwrap()
        );
    }

    #[test]
    fn rebase_query_works() {
        let mut deps = mock_dependencies();
        let controller = "controller";
        let instantiate_msg = InstantiateMsg {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 9,
            controller: controller.to_string(),
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info(controller, &[]);
        rebase(deps.as_mut(), info, Decimal::percent(120)).unwrap();
        assert_eq!(
            Decimal::percent(120),
            MULTIPLIER.load(&deps.storage).unwrap()
        );

        let res = query_multiplier(deps.as_ref()).unwrap();
        assert_eq!(
            MultiplierResponse {
                multiplier: Decimal::percent(120)
            },
            res
        );
    }
}
