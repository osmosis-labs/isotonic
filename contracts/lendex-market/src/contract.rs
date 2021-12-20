#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdResult, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw2::set_contract_version;
use cw20::BalanceResponse;
use utils::interest::Interest;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TransferableAmountResponse};
use crate::state::{Config, CONFIG, SECONDS_IN_YEAR};

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
        distributed_token: msg.distributed_token.clone(),
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
        distributed_token: msg.distributed_token.clone(),
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
        market_token: msg.market_token,
        rates: msg.interest_rate,
        interest_charge_period: msg.interest_charge_period,
        last_charged: env.block.time.seconds()
            - env.block.time.seconds() % msg.interest_charge_period,
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

fn token_instantiate_reply(
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
        Deposit {} => execute::deposit(deps, env, info),
        Withdraw { amount } => execute::withdraw(deps, env, info, amount),
        Borrow { amount } => execute::borrow(deps, env, info, amount),
        Repay {} => execute::repay(deps, env, info),
    }
}

mod execute {
    use super::*;

    /// Helper that determines if an address can withdraw the specified amount.
    fn can_withdraw(_deps: Deps, _sender: &Addr, _amount: Uint128) -> Result<bool, ContractError> {
        // TODO: actual checks here
        Ok(true)
    }

    /// Function that is supposed to be called before every mint/burn operation.
    /// It calculates ratio for increasing both btokens and ltokens.a
    /// btokens formula:
    /// b_ratio = calculate_interest() * epochs_passed * epoch_length / 31.556.736
    /// ltokens formula:
    /// l_ratio = b_supply() * b_ratio / l_supply()
    fn charge_interest(deps: DepsMut, env: Env) -> StdResult<Vec<SubMsg>> {
        use lendex_token::msg::ExecuteMsg;

        let mut cfg = CONFIG.load(deps.storage)?;

        let epochs_passed =
            (env.block.time.seconds() - cfg.last_charged) / cfg.interest_charge_period;

        if epochs_passed == 0 {
            return Ok(vec![]);
        }

        let charged_time = epochs_passed * cfg.interest_charge_period;
        cfg.last_charged += charged_time;
        CONFIG.save(deps.storage, &cfg)?;

        let tokens_info = query::token_info(deps.as_ref(), &cfg)?;
        // safety - if there are no ltokens, don't charge interest (would panic later)
        if tokens_info.ltoken.total_supply.display_amount() == Uint128::zero() {
            return Ok(vec![]);
        }

        let interest = query::interest(&cfg, &tokens_info)?;

        // calculate_interest() * epochs_passed * epoch_length / SECONDS_IN_YEAR
        let btoken_ratio: Decimal =
            interest.interest * Decimal::from_ratio(charged_time as u128, SECONDS_IN_YEAR);

        // b_supply() * ratio / l_supply()
        let ltoken_ratio: Decimal = Decimal::from_ratio(
            tokens_info.btoken.total_supply.display_amount() * btoken_ratio,
            tokens_info.ltoken.total_supply.display_amount(),
        );

        let btoken_rebase = to_binary(&ExecuteMsg::Rebase {
            ratio: btoken_ratio + Decimal::one(),
        })?;
        let bwrapped = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.btoken_contract.to_string(),
            msg: btoken_rebase,
            funds: vec![],
        });

        let ltoken_rebase = to_binary(&ExecuteMsg::Rebase {
            ratio: ltoken_ratio + Decimal::one(),
        })?;
        let lwrapped = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ltoken_contract.to_string(),
            msg: ltoken_rebase,
            funds: vec![],
        });

        Ok(vec![bwrapped, lwrapped])
    }

    /// Validates funds sent with the message, that they contain only the base asset. Returns
    /// amount of funds sent, or error if:
    /// * No funds were passed with the message (`NoFundsSent` error)
    /// * Multiple denoms were sent (`ExtraDenoms` error)
    /// * A single denom different than cfg.market_token was sent (`InvalidDenom` error)
    fn validate_funds(funds: &[Coin], market_token_denom: &str) -> Result<Uint128, ContractError> {
        match funds {
            [] => Err(ContractError::NoFundsSent {}),
            [Coin { denom, amount }] if denom == market_token_denom => Ok(*amount),
            [_] => Err(ContractError::InvalidDenom(market_token_denom.to_string())),
            _ => Err(ContractError::ExtraDenoms(market_token_denom.to_string())),
        }
    }

    /// Handler for `ExecuteMsg::Deposit`
    pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let funds_sent = validate_funds(&info.funds, &cfg.market_token)?;

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps, env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

        let mint_msg = to_binary(&lendex_token::msg::ExecuteMsg::Mint {
            recipient: info.sender.to_string(),
            amount: lendex_token::DisplayAmount::raw(funds_sent),
        })?;
        let wrapped_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ltoken_contract.to_string(),
            msg: mint_msg,
            funds: vec![],
        });

        response = response
            .add_attribute("action", "deposit")
            .add_attribute("sender", info.sender)
            .add_submessage(wrapped_msg);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::Withdraw`
    pub fn withdraw(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;

        if !can_withdraw(deps.as_ref(), &info.sender, amount)? {
            return Err(ContractError::CannotWithdraw {
                account: info.sender.to_string(),
                amount,
            });
        }

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps, env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

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
            amount: vec![coin(amount.u128(), cfg.market_token)],
        });

        response = response
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender)
            .add_submessage(wrapped_msg)
            .add_message(send_msg);
        Ok(response)
    }

    fn can_borrow(_deps: Deps, _sender: &Addr, _amount: Uint128) -> Result<bool, ContractError> {
        // TODO: fill implementation
        Ok(true)
    }

    pub fn borrow(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;

        if !can_borrow(deps.as_ref(), &info.sender, amount)? {
            return Err(ContractError::CannotBorrow {
                amount,
                account: info.sender.to_string(),
            });
        }

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps, env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

        // Mint desired amount of btokens,
        let msg = to_binary(&lendex_token::msg::ExecuteMsg::Mint {
            recipient: info.sender.to_string(),
            amount: lendex_token::DisplayAmount::raw(amount),
        })?;
        let mint_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.btoken_contract.to_string(),
            msg,
            funds: vec![],
        });

        // Sent tokens to sender's account
        let bank_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(amount.u128(), cfg.market_token)],
        });

        response = response
            .add_attribute("action", "borrow")
            .add_attribute("sender", info.sender)
            .add_submessage(mint_msg)
            .add_message(bank_msg);
        Ok(response)
    }

    pub fn repay(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let funds_sent = validate_funds(&info.funds, &cfg.market_token)?;

        // Check balance of btokens to repay
        let response: BalanceResponse = deps.querier.query_wasm_smart(
            &cfg.btoken_contract,
            &lendex_token::QueryMsg::Balance {
                address: info.sender.to_string(),
            },
        )?;
        let balance = response.balance;
        // If there are more tokens sent then there are to repay, burn only desired
        // amount and return the difference
        let repay_amount = if funds_sent <= balance {
            funds_sent
        } else {
            balance
        };

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps, env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

        let msg = to_binary(&lendex_token::msg::ExecuteMsg::BurnFrom {
            owner: info.sender.to_string(),
            amount: lendex_token::DisplayAmount::raw(repay_amount),
        })?;
        let burn_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.btoken_contract.to_string(),
            msg,
            funds: vec![],
        });

        response = response
            .add_attribute("action", "repay")
            .add_attribute("sender", info.sender.clone())
            .add_submessage(burn_msg);

        // Return surplus of sent tokens
        if funds_sent > repay_amount {
            let tokens_to_return = funds_sent - repay_amount;
            let bank_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![coin(tokens_to_return.u128(), cfg.market_token)],
            });
            response = response.add_message(bank_msg);
        }

        Ok(response)
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;
    match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?),
        TransferableAmount { token, account } => {
            let token = deps.api.addr_validate(&token)?;
            to_binary(&query::transferable_amount(deps, token, account)?)
        }
        Interest {} => to_binary(&query::calculate_interest(deps)?),
    }
}

mod query {
    use super::*;

    use cosmwasm_std::{Decimal, StdError, Uint128};
    use cw20::BalanceResponse;
    use lendex_token::msg::{QueryMsg, TokenInfoResponse};

    use crate::msg::InterestResponse;
    use crate::state::TokensInfo;

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

    pub fn calculate_interest(deps: Deps) -> StdResult<InterestResponse> {
        let config = CONFIG.load(deps.storage)?;
        interest(&config, &token_info(deps, &config)?)
    }

    pub fn token_info(deps: Deps, config: &Config) -> StdResult<TokensInfo> {
        let ltoken_contract = &config.ltoken_contract;
        let ltoken: TokenInfoResponse = deps
            .querier
            .query_wasm_smart(ltoken_contract, &QueryMsg::TokenInfo {})?;
        let btoken_contract = &config.btoken_contract;
        let btoken: TokenInfoResponse = deps
            .querier
            .query_wasm_smart(btoken_contract, &QueryMsg::TokenInfo {})?;
        Ok(TokensInfo { ltoken, btoken })
    }

    pub fn interest(config: &Config, tokens_info: &TokensInfo) -> StdResult<InterestResponse> {
        let utilisation = if tokens_info.ltoken.total_supply.is_zero() {
            Decimal::zero()
        } else {
            Decimal::from_ratio(
                tokens_info.btoken.total_supply.display_amount(),
                tokens_info.ltoken.total_supply.display_amount(),
            )
        };

        let interest = match config.rates {
            Interest::Linear { base, slope } => base + slope * utilisation,
        };

        Ok(InterestResponse {
            interest,
            utilisation,
            charge_period: Timestamp::from_seconds(config.interest_charge_period),
        })
    }
}
