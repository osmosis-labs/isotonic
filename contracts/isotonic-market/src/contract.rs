#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::contract::query::token_info;
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, QueryTotalCreditLine, SudoMsg,
    TransferableAmountResponse,
};
use crate::state::{Config, CONFIG, RESERVE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:isotonic-market";
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

    let ltoken_msg = isotonic_token::msg::InstantiateMsg {
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
    let btoken_msg = isotonic_token::msg::InstantiateMsg {
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
        market_cap: msg.market_cap,
        rates: msg.interest_rate.validate()?,
        interest_charge_period: msg.interest_charge_period,
        last_charged: env.block.time.seconds()
            - env.block.time.seconds() % msg.interest_charge_period,
        common_token: msg.common_token,
        collateral_ratio: msg.collateral_ratio,
        price_oracle: msg.price_oracle,
        credit_agency: info.sender.clone(),
        reserve_factor: msg.reserve_factor,
    };
    CONFIG.save(deps.storage, &cfg)?;

    RESERVE.save(deps.storage, &Uint128::zero())?;

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
        RepayTo { account, amount } => {
            let account = deps.api.addr_validate(&account)?;
            execute::repay_to(deps, env, info, account, amount)
        }
        TransferFrom {
            source,
            destination,
            amount,
            liquidation_price,
        } => {
            let source = deps.api.addr_validate(&source)?;
            let destination = deps.api.addr_validate(&destination)?;
            execute::transfer_from(
                deps,
                env,
                info,
                source,
                destination,
                amount,
                liquidation_price,
            )
        }
        AdjustCommonToken { new_token } => {
            execute::adjust_common_token(deps, info.sender, new_token)
        }
    }
}

// Available credit line helpers
mod cr_utils {
    use utils::credit_line::CreditLineResponse;

    use super::*;

    use cosmwasm_std::Fraction;

    // TODO: Check for rounding error https://github.com/confio/isotonic/issues/40
    pub fn divide(top: Uint128, bottom: Decimal) -> Uint128 {
        top * bottom.inv().unwrap_or_else(Decimal::zero)
    }

    fn available_local_tokens(
        deps: Deps,
        common_tokens: Uint128,
    ) -> Result<Uint128, ContractError> {
        // Price is defined as common/local
        // (see price_market_local_per_common function from this file)
        Ok(divide(
            common_tokens,
            query::price_market_local_per_common(deps)?.rate_sell_per_buy,
        ))
    }

    fn query_available_tokens(
        deps: Deps,
        config: &Config,
        account: String,
    ) -> Result<Uint128, ContractError> {
        let credit: CreditLineResponse = deps.querier.query_wasm_smart(
            &config.credit_agency,
            &QueryTotalCreditLine::TotalCreditLine { account },
        )?;
        let credit = credit.validate(&config.common_token)?;

        // Available credit for that account amongst all markets
        let available_common = credit.credit_line.saturating_sub(credit.debt);
        let available_local = available_local_tokens(deps, available_common)?;
        Ok(available_local)
    }

    /// Helper that determines if an address can borrow the specified amount.
    pub fn can_borrow(
        deps: Deps,
        config: &Config,
        account: impl Into<String>,
        amount: Uint128,
    ) -> Result<bool, ContractError> {
        let available = query_available_tokens(deps, config, account.into())?;
        Ok(amount <= available)
    }

    /// Helper returning amount of tokens available to transfer/withdraw
    pub fn transferable_amount(
        deps: Deps,
        config: &Config,
        account: impl Into<String>,
    ) -> Result<Uint128, ContractError> {
        let available = query_available_tokens(deps, config, account.into())?;
        let can_transfer = divide(available, config.collateral_ratio);
        Ok(can_transfer)
    }
}

mod execute {
    use crate::{
        interest::{calculate_interest, epochs_passed, InterestUpdate},
        msg::CreditAgencyExecuteMsg,
    };

    use super::*;

    /// Function that is supposed to be called before every mint/burn operation.
    /// It calculates ratio for increasing both btokens and ltokens.a
    /// btokens formula:
    /// b_ratio = calculate_interest() * epochs_passed * epoch_length / 31.556.736
    /// ltokens formula:
    /// l_ratio = b_supply() * b_ratio / l_supply()
    pub fn charge_interest(deps: DepsMut, env: Env) -> Result<Vec<SubMsg>, ContractError> {
        use isotonic_token::msg::ExecuteMsg;

        let mut cfg = CONFIG.load(deps.storage)?;
        let epochs_passed = epochs_passed(&cfg, env)?;
        cfg.last_charged += epochs_passed * cfg.interest_charge_period;
        CONFIG.save(deps.storage, &cfg)?;

        if epochs_passed == 0 {
            return Ok(vec![]);
        }

        if let Some(InterestUpdate {
            reserve,
            ltoken_ratio,
            btoken_ratio,
        }) = calculate_interest(deps.as_ref(), epochs_passed)?
        {
            RESERVE.save(deps.storage, &reserve)?;

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
        } else {
            Ok(vec![])
        }
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

    fn enter_market(cfg: &Config, account: &Addr) -> StdResult<SubMsg> {
        let msg = to_binary(&CreditAgencyExecuteMsg::EnterMarket {
            account: account.to_string(),
        })?;

        Ok(SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.credit_agency.to_string(),
            msg,
            funds: vec![],
        }))
    }

    /// Handler for `ExecuteMsg::Deposit`
    pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let funds_sent = validate_funds(&info.funds, &cfg.market_token)?;

        let mut response = Response::new();

        if let Some(cap) = cfg.market_cap {
            let ltoken_supply = query::token_info(deps.as_ref(), &cfg)?
                .ltoken
                .total_supply
                .display_amount();
            if ltoken_supply + funds_sent > cap {
                return Err(ContractError::DepositOverCap {
                    attempted_deposit: funds_sent,
                    ltoken_supply,
                    cap,
                });
            }
        }

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps, env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

        let mint_msg = to_binary(&isotonic_token::msg::ExecuteMsg::Mint {
            recipient: info.sender.to_string(),
            amount: isotonic_token::DisplayAmount::raw(funds_sent),
        })?;
        let wrapped_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ltoken_contract.to_string(),
            msg: mint_msg,
            funds: vec![],
        });

        response = response
            .add_attribute("action", "deposit")
            .add_attribute("sender", info.sender.clone())
            .add_submessage(wrapped_msg)
            .add_submessage(enter_market(&cfg, &info.sender)?);
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

        if cr_utils::transferable_amount(deps.as_ref(), &cfg, &info.sender)? < amount {
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
        let burn_msg = to_binary(&isotonic_token::msg::ExecuteMsg::BurnFrom {
            owner: info.sender.to_string(),
            amount: isotonic_token::DisplayAmount::raw(amount),
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

    /// Handler for `ExecuteMsg::Borrow`
    pub fn borrow(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;

        if !cr_utils::can_borrow(deps.as_ref(), &cfg, &info.sender, amount)? {
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
        let msg = to_binary(&isotonic_token::msg::ExecuteMsg::Mint {
            recipient: info.sender.to_string(),
            amount: isotonic_token::DisplayAmount::raw(amount),
        })?;
        let mint_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.btoken_contract.to_string(),
            msg,
            funds: vec![],
        });

        // Sent tokens to sender's account
        let bank_msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(amount.u128(), &cfg.market_token)],
        });

        response = response
            .add_attribute("action", "borrow")
            .add_attribute("sender", info.sender.clone())
            .add_submessage(mint_msg)
            .add_submessage(enter_market(&cfg, &info.sender)?)
            .add_message(bank_msg);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::Repay`
    pub fn repay(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let funds_sent = validate_funds(&info.funds, &cfg.market_token)?;

        let debt = query::btoken_balance(deps.as_ref(), &cfg, &info.sender)?;
        // If there are more tokens sent then there are to repay, burn only desired
        // amount and return the difference
        let repay_amount = std::cmp::min(funds_sent, debt.amount);

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps.branch(), env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

        let msg = to_binary(&isotonic_token::msg::ExecuteMsg::BurnFrom {
            owner: info.sender.to_string(),
            amount: isotonic_token::DisplayAmount::raw(repay_amount),
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

    /// Handler for `ExecuteMsg::RepayTo`
    /// Requires sender to be a Credit Agency, otherwise fails
    pub fn repay_to(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        account: Addr,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        if cfg.credit_agency != info.sender {
            return Err(ContractError::LiquidationRequiresCreditAgency {});
        }

        let funds = validate_funds(&info.funds, &cfg.market_token)?;

        let btokens_balance = query::btoken_balance(deps.as_ref(), &cfg, &account)?;
        // if account has less btokens then caller wants to pay off, liquidation fails
        if funds > btokens_balance.amount {
            return Err(ContractError::LiquidationInsufficientBTokens {
                account: account.to_string(),
                btokens: btokens_balance.amount,
            });
        }

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps, env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

        let msg = to_binary(&isotonic_token::msg::ExecuteMsg::BurnFrom {
            owner: account.to_string(),
            amount: isotonic_token::DisplayAmount::raw(amount),
        })?;
        let burn_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.btoken_contract.to_string(),
            msg,
            funds: vec![],
        });

        response = response
            .add_attribute("action", "repay_to")
            .add_attribute("sender", info.sender)
            .add_attribute("debtor", account)
            .add_submessage(burn_msg);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::TransferFrom`
    /// Requires sender to be a Credit Agency, otherwise fails
    /// Amount must be in common denom (from CA)
    pub fn transfer_from(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        source: Addr,
        destination: Addr,
        amount: Coin,
        liquidation_price: Decimal,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        if cfg.credit_agency != info.sender {
            return Err(ContractError::LiquidationRequiresCreditAgency {});
        }
        let amount = validate_funds(&[amount], &cfg.common_token)?;

        let mut response = Response::new();

        // charge interests before transferring tokens
        let charge_msgs = charge_interest(deps.branch(), env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

        // calculate repaid value
        let price_rate = query::price_market_local_per_common(deps.as_ref())?.rate_sell_per_buy;
        let repaid_value = cr_utils::divide(amount, price_rate * liquidation_price);

        // transfer claimed amount of ltokens from account source to destination
        let msg = to_binary(&isotonic_token::msg::ExecuteMsg::TransferFrom {
            sender: source.to_string(),
            recipient: destination.to_string(),
            amount: isotonic_token::DisplayAmount::raw(repaid_value),
        })?;
        let transfer_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ltoken_contract.to_string(),
            msg,
            funds: vec![],
        });

        response = response
            .add_submessage(enter_market(&cfg, &destination)?)
            .add_attribute("action", "transfer_from")
            .add_attribute("from", source)
            .add_attribute("to", destination)
            .add_submessage(transfer_msg);
        Ok(response)
    }

    /// Handler for `ExecuteMsg::AdjustCommonToken`
    pub fn adjust_common_token(
        deps: DepsMut,
        sender: Addr,
        new_token: String,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;

        if sender != cfg.credit_agency {
            return Err(ContractError::Unauthorized {});
        }

        cfg.common_token = new_token;

        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;
    let res = match msg {
        Configuration {} => to_binary(&query::config(deps, env)?)?, //&CONFIG.load(deps.storage)?
        TokensBalance { account } => to_binary(&query::tokens_balance(deps, env, account)?)?,
        TransferableAmount { token, account } => {
            let token = deps.api.addr_validate(&token)?;
            to_binary(&query::transferable_amount(deps, token, account)?)?
        }
        Interest {} => {
            let cfg = CONFIG.load(deps.storage)?;
            to_binary(&query::interest(&cfg, &token_info(deps, &cfg)?)?)?
        }
        PriceMarketLocalPerCommon {} => to_binary(&query::price_market_local_per_common(deps)?)?,
        CreditLine { account } => {
            let account = deps.api.addr_validate(&account)?;
            to_binary(&query::credit_line(deps, account)?)?
        }
        Reserve {} => to_binary(&query::reserve(deps, env)?)?,
    };
    Ok(res)
}

mod query {
    use super::*;

    use cosmwasm_std::{coin, Coin, Decimal, Uint128};
    use cw20::BalanceResponse;
    use isotonic_oracle::msg::{PriceResponse, QueryMsg as OracleQueryMsg};
    use isotonic_token::msg::QueryMsg as TokenQueryMsg;
    use utils::credit_line::{CreditLineResponse, CreditLineValues};
    use utils::price::{coin_times_price_rate, PriceRate};

    use crate::interest::{calculate_interest, epochs_passed, token_supply, utilisation};
    use crate::msg::{InterestResponse, ReserveResponse, TokensBalanceResponse};
    use crate::state::TokensInfo;

    fn token_balance(
        deps: Deps,
        token_contract: &Addr,
        address: String,
    ) -> StdResult<BalanceResponse> {
        deps.querier
            .query_wasm_smart(token_contract, &TokenQueryMsg::Balance { address })
    }

    pub fn btoken_balance(
        deps: Deps,
        config: &Config,
        account: impl ToString,
    ) -> Result<Coin, ContractError> {
        Ok(coin(
            token_balance(deps, &config.btoken_contract, account.to_string())?
                .balance
                .u128(),
            config.market_token.clone(),
        ))
    }

    fn ltoken_balance(
        deps: Deps,
        config: &Config,
        account: impl ToString,
    ) -> Result<Coin, ContractError> {
        Ok(coin(
            token_balance(deps, &config.ltoken_contract, account.to_string())?
                .balance
                .u128(),
            config.market_token.clone(),
        ))
    }

    /// Handler for `QueryMsg::Config`
    pub fn config(deps: Deps, env: Env) -> Result<Config, ContractError> {
        let mut config = CONFIG.load(deps.storage)?;

        let unhandled_charge_period = epochs_passed(&config, env)?;
        config.last_charged += unhandled_charge_period * config.interest_charge_period;

        Ok(config)
    }

    /// Handler for `QueryMsg::TokensBalance`
    pub fn tokens_balance(
        deps: Deps,
        env: Env,
        account: String,
    ) -> Result<TokensBalanceResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let mut ltokens = ltoken_balance(deps, &config, account.clone())?.amount;
        let mut btokens = btoken_balance(deps, &config, account)?.amount;

        if let Some(update) = calculate_interest(deps, epochs_passed(&config, env)?)? {
            ltokens += ltokens * update.ltoken_ratio;
            btokens += btokens * update.btoken_ratio;
        }

        Ok(TokensBalanceResponse { ltokens, btokens })
    }

    /// Handler for `QueryMsg::TransferableAmount`
    pub fn transferable_amount(
        deps: Deps,
        token: Addr,
        account: String,
    ) -> Result<TransferableAmountResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if token == config.btoken_contract {
            Ok(TransferableAmountResponse {
                transferable: Uint128::zero(),
            })
        } else if token == config.ltoken_contract {
            let transferable = cr_utils::transferable_amount(deps, &config, &account)?;
            Ok(TransferableAmountResponse { transferable })
        } else {
            Err(ContractError::UnrecognisedToken(token.to_string()))
        }
    }

    pub fn token_info(deps: Deps, config: &Config) -> Result<TokensInfo, ContractError> {
        token_supply(deps, config)
    }

    /// Handler for `QueryMsg::Interest`
    pub fn interest(
        config: &Config,
        tokens_info: &TokensInfo,
    ) -> Result<InterestResponse, ContractError> {
        let utilisation = utilisation(tokens_info);

        let interest = config.rates.calculate_interest_rate(utilisation);

        Ok(InterestResponse {
            interest,
            utilisation,
            charge_period: Timestamp::from_seconds(config.interest_charge_period),
        })
    }

    /// Handler for `QueryMsg::PriceMarketLocalPerCommon`
    /// Ratio is for sell market_token / buy common_token
    pub fn price_market_local_per_common(deps: Deps) -> Result<PriceRate, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        // If denoms are the same, just return 1:1
        if config.common_token == config.market_token {
            Ok(PriceRate {
                sell_denom: config.market_token.clone(),
                buy_denom: config.common_token,
                rate_sell_per_buy: Decimal::one(),
            })
        } else {
            let price_response: PriceResponse = deps.querier.query_wasm_smart(
                config.price_oracle.clone(),
                &OracleQueryMsg::Price {
                    sell: config.market_token.clone(),
                    buy: config.common_token.clone(),
                },
            )?;
            Ok(PriceRate {
                sell_denom: config.market_token.clone(),
                buy_denom: config.common_token,
                rate_sell_per_buy: price_response.rate,
            })
        }
    }

    /// Handler for `QueryMsg::CreditLine`
    pub fn credit_line(deps: Deps, account: Addr) -> Result<CreditLineResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let collateral = ltoken_balance(deps, &config, &account)?;
        let debt = btoken_balance(deps, &config, &account)?;
        if collateral.amount.is_zero() && debt.amount.is_zero() {
            return Ok(CreditLineValues::zero().make_response(config.common_token));
        }

        let price_ratio = price_market_local_per_common(deps)?;
        let collateral = coin_times_price_rate(&collateral, &price_ratio)?;
        let debt = coin_times_price_rate(&debt, &price_ratio)?.amount;
        let credit_line = collateral.amount * config.collateral_ratio;
        Ok(CreditLineValues::new(collateral.amount, credit_line, debt)
            .make_response(config.common_token))
    }

    /// Handler for `QueryMsg::Reserve`
    pub fn reserve(deps: Deps, env: Env) -> Result<ReserveResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let reserve = calculate_interest(deps, epochs_passed(&config, env)?)?
            .map(|update| update.reserve)
            .unwrap_or(RESERVE.load(deps.storage)?);

        Ok(ReserveResponse { reserve })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    use SudoMsg::*;
    match msg {
        AdjustCollateralRatio { new_ratio } => sudo::adjust_collateral_ratio(deps, new_ratio),
        AdjustReserveFactor { new_factor } => sudo::adjust_reserve_factor(deps, new_factor),
        AdjustPriceOracle { new_oracle } => sudo::adjust_price_oracle(deps, new_oracle),
        AdjustMarketCap { new_cap } => sudo::adjust_market_cap(deps, new_cap),
        AdjustInterestRates { new_interest_rates } => {
            sudo::adjust_interest_rates(deps, env, new_interest_rates)
        }
    }
}

mod sudo {
    use super::*;

    use utils::interest::Interest;

    pub fn adjust_collateral_ratio(
        deps: DepsMut,
        new_ratio: Decimal,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.collateral_ratio = new_ratio;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_reserve_factor(
        deps: DepsMut,
        new_factor: Decimal,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.reserve_factor = new_factor;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_price_oracle(
        deps: DepsMut,
        new_oracle: String,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.price_oracle = new_oracle;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_market_cap(
        deps: DepsMut,
        new_cap: Option<Uint128>,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.market_cap = new_cap;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_interest_rates(
        mut deps: DepsMut,
        env: Env,
        new_interest_rates: Interest,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        let charge_msgs = execute::charge_interest(deps.branch(), env)?;
        let mut response = Response::new();
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }
        let interest_rates = new_interest_rates.validate()?;
        cfg.rates = interest_rates;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(response)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    CONFIG.update::<_, StdError>(deps.storage, |mut cfg| {
        if let Some(token_id) = msg.isotonic_token_id {
            cfg.token_id = token_id;
        }
        Ok(cfg)
    })?;

    Ok(Response::new())
}
