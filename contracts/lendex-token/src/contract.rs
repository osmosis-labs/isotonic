#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;

use crate::display_amount::DisplayAmount;
use crate::error::ContractError;
use crate::msg::{
    BalanceResponse, ControllerQuery, Cw20ReceiveMsg, ExecuteMsg, FundsResponse, InstantiateMsg,
    MultiplierResponse, QueryMsg, TokenInfoResponse, TransferableAmountResp,
};
use crate::state::{
    Distribution, TokenInfo, WithdrawAdjustment, BALANCES, CONTROLLER, DISTRIBUTION, MULTIPLIER,
    POINTS_SHIFT, TOKEN_INFO, TOTAL_SUPPLY, WITHDRAW_ADJUSTMENT,
};

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

    let distribution = Distribution {
        denom: msg.distributed_token,
        points_per_token: Uint128::zero(),
        points_leftover: 0,
        distributed_total: Uint128::zero(),
        withdrawable_total: Uint128::zero(),
    };

    DISTRIBUTION.save(deps.storage, &distribution)?;

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
    mut deps: DepsMut,
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

    let distribution = DISTRIBUTION.load(deps.storage)?;
    let ppt = distribution.points_per_token.u128();

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
    apply_points_correction(deps.branch(), &sender_addr, ppt, amount.u128() as _)?;

    BALANCES.update(
        deps.storage,
        &recipient,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;
    apply_points_correction(deps.branch(), &sender_addr, ppt, -(amount.u128() as i128))?;

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
    mut deps: DepsMut,
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

    let distribution = DISTRIBUTION.load(deps.storage)?;
    let ppt = distribution.points_per_token.u128();

    let recipient_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &recipient_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;
    apply_points_correction(deps.branch(), &recipient_addr, ppt, amount.u128() as _)?;

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
pub fn burn_from(
    mut deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: DisplayAmount,
) -> Result<Response, ContractError> {
    let controller = CONTROLLER.load(deps.storage)?;
    let multiplier = MULTIPLIER.load(deps.storage)?;
    let owner = deps.api.addr_validate(&owner)?;
    let amount = amount.to_stored_amount(multiplier);

    if info.sender != controller {
        return Err(ContractError::Unauthorized {});
    }

    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let ppt = DISTRIBUTION.load(deps.storage)?.points_per_token;

    BALANCES.update(
        deps.storage,
        &owner,
        |balance: Option<Uint128>| -> Result<_, ContractError> {
            let balance = balance.unwrap_or_default();
            balance
                .checked_sub(amount)
                .map_err(|_| ContractError::insufficient_tokens(balance, amount))
        },
    )?;
    apply_points_correction(
        deps.branch(),
        &owner,
        ppt.u128() as _,
        -(amount.u128() as i128),
    )?;

    TOTAL_SUPPLY.update(deps.storage, |supply| -> Result<_, ContractError> {
        supply
            .checked_sub(amount)
            .map_err(|_| ContractError::insufficient_tokens(supply, amount))
    })?;

    let res = Response::new()
        .add_attribute("action", "burn_from")
        .add_attribute("from", owner)
        .add_attribute("by", info.sender)
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

/// Handler for `ExecuteMsg::Distribute`
pub fn distribute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    let total_supply = TOTAL_SUPPLY.load(deps.storage)?.u128();

    if total_supply == 0 {
        return Err(ContractError::NoHoldersToDistributeTo {});
    }

    let sender = sender
        .map(|sender| deps.api.addr_validate(&sender))
        .transpose()?
        .unwrap_or(info.sender);

    let mut distribution = DISTRIBUTION.load(deps.storage)?;

    let withdrawable: u128 = distribution.withdrawable_total.into();
    let balance: u128 = deps
        .querier
        .query_balance(env.contract.address, distribution.denom.clone())?
        .amount
        .into();

    let amount = balance - withdrawable;
    if amount == 0 {
        return Ok(Response::new());
    }

    let leftover: u128 = distribution.points_leftover.into();
    let points = (amount << POINTS_SHIFT) + leftover;
    let points_per_token = points / total_supply;
    distribution.points_leftover = (points % total_supply) as u64;

    // Everything goes back to 128-bits/16-bytes
    // Full amount is added here to total withdrawable, as it should not be considered on its own
    // on future distributions - even if because of calculation offsets it is not fully
    // distributed, the error is handled by leftover.
    distribution.points_per_token += Uint128::from(points_per_token);
    distribution.distributed_total += Uint128::from(amount);
    distribution.withdrawable_total += Uint128::from(amount);

    DISTRIBUTION.save(deps.storage, &distribution)?;

    let resp = Response::new()
        .add_attribute("action", "distribute_tokens")
        .add_attribute("sender", sender.as_str())
        .add_attribute("denom", &distribution.denom)
        .add_attribute("amount", &amount.to_string());

    Ok(resp)
}

/// Handler for `ExecuteMsg::WithdrawFunds`
fn withdraw_funds(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut distribution = DISTRIBUTION.load(deps.storage)?;
    let mut adjustment = WITHDRAW_ADJUSTMENT
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    let token = withdrawable_funds(deps.as_ref(), &info.sender, &distribution, &adjustment)?;
    if token.amount.is_zero() {
        // Just do nothing
        return Ok(Response::new());
    }

    adjustment.withdrawn_funds += token.amount;
    WITHDRAW_ADJUSTMENT.save(deps.storage, &info.sender, &adjustment)?;
    distribution.withdrawable_total -= token.amount;
    DISTRIBUTION.save(deps.storage, &distribution)?;

    let resp = Response::new()
        .add_attribute("action", "withdraw_tokens")
        .add_attribute("owner", info.sender.as_str())
        .add_attribute("token", &token.denom)
        .add_attribute("amount", &token.amount.to_string())
        .add_submessage(SubMsg::new(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![token],
        }));

    Ok(resp)
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
        BurnFrom { owner, amount } => burn_from(deps, info, owner, amount),
        Rebase { ratio } => rebase(deps, info, ratio),
        Distribute { sender } => distribute(deps, env, info, sender),
        WithdrawFunds {} => withdraw_funds(deps, info),
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

/// Handler for `QueryMsg::DistributedFunds`
pub fn query_distributed_funds(deps: Deps) -> StdResult<FundsResponse> {
    let distribution = DISTRIBUTION.load(deps.storage)?;
    Ok(FundsResponse {
        funds: coin(distribution.distributed_total.into(), &distribution.denom),
    })
}

/// Handler for `QueryMsg::UndistributedFunds`
pub fn query_undistributed_funds(deps: Deps, env: Env) -> StdResult<FundsResponse> {
    let distribution = DISTRIBUTION.load(deps.storage)?;
    let balance = deps
        .querier
        .query_balance(env.contract.address, distribution.denom.clone())?
        .amount;
    Ok(FundsResponse {
        funds: coin(
            (balance - distribution.withdrawable_total).into(),
            &distribution.denom,
        ),
    })
}

/// `QueryMsg` entry point
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Balance { address } => to_binary(&query_balance(deps, address)?),
        TokenInfo {} => to_binary(&query_token_info(deps)?),
        Multiplier {} => to_binary(&query_multiplier(deps)?),
        DistributedFunds {} => to_binary(&query_distributed_funds(deps)?),
        UndistributedFunds {} => to_binary(&query_undistributed_funds(deps, env)?),
    }
}

/// Calculates withdrawable funds from distribution and adjustment info.
pub fn withdrawable_funds(
    deps: Deps,
    owner: &Addr,
    distribution: &Distribution,
    adjustment: &WithdrawAdjustment,
) -> StdResult<Coin> {
    let ppt: u128 = distribution.points_per_token.into();
    let tokens: u128 = BALANCES
        .may_load(deps.storage, owner)?
        .unwrap_or_default()
        .into();
    let correction: i128 = adjustment.points_correction.into();
    let withdrawn: u128 = adjustment.withdrawn_funds.into();
    let points = (ppt * tokens) as i128;
    let points = points + correction;
    let amount = points as u128 >> POINTS_SHIFT;
    let amount = amount - withdrawn;

    Ok(coin(amount, &distribution.denom))
}

/// Applies points correction for given address.
/// `ppt` is current value from `POINTS_PER_TOKEN` - not loaded in function, to
/// avoid multiple queries on bulk updates.
/// `diff` is the weight change
pub fn apply_points_correction(deps: DepsMut, addr: &Addr, ppt: u128, diff: i128) -> StdResult<()> {
    WITHDRAW_ADJUSTMENT.update(deps.storage, addr, |old| -> StdResult<_> {
        let mut old = old.unwrap_or_default();
        let points_correction: i128 = old.points_correction.into();
        old.points_correction = (points_correction - ppt as i128 * diff).into();
        Ok(old)
    })?;
    Ok(())
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
            distributed_token: String::new(),
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
            distributed_token: String::new(),
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
