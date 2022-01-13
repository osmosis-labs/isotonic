#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdResult, SubMsg, Timestamp, Uint128, WasmMsg,
};
use cw0::parse_reply_instantiate_data;
use cw2::set_contract_version;
use utils::interest::Interest;

use crate::error::ContractError;
use crate::msg::{
    CreditLineResponse, ExecuteMsg, InstantiateMsg, QueryMsg, QueryTotalCreditLine,
    TransferableAmountResponse,
};
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
        common_token: msg.common_token,
        collateral_ratio: msg.collateral_ratio,
        price_oracle: msg.price_oracle,
        credit_agency: info.sender.clone(),
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
            execute::transfer_from(deps, info, source, destination, amount, liquidation_price)
        }
    }
}

// Available credit line helpers
mod cr_utils {
    use super::*;

    use cosmwasm_std::Fraction;

    // TODO: Check for rounding error https://github.com/confio/lendex/issues/40
    pub fn divide(top: Uint128, bottom: Decimal) -> Uint128 {
        top * bottom.inv().unwrap_or_else(Decimal::zero)
    }

    fn query_available_tokens(
        deps: Deps,
        config: &Config,
        account: String,
    ) -> Result<Uint128, ContractError> {
        dbg!(account.clone());
        let credit: CreditLineResponse = dbg!(deps.querier.query_wasm_smart(
            &config.credit_agency,
            &QueryTotalCreditLine::TotalCreditLine { account },
        )?);
        // Available credit for that account amongst all markets
        let available_common = credit.credit_line.saturating_sub(credit.debt);
        // Price is defined as common/local
        // (see price_ratio_from_oracle function from this file)
        let available = divide(
            available_common,
            query::price_local_per_common(deps, config)?,
        );
        Ok(available)
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
        let available = dbg!(query_available_tokens(deps, config, account.into())?);
        let can_transfer = dbg!(divide(available, config.collateral_ratio));
        Ok(can_transfer)
    }
}

mod execute {
    use super::*;

    /// Function that is supposed to be called before every mint/burn operation.
    /// It calculates ratio for increasing both btokens and ltokens.a
    /// btokens formula:
    /// b_ratio = calculate_interest() * epochs_passed * epoch_length / 31.556.736
    /// ltokens formula:
    /// l_ratio = b_supply() * b_ratio / l_supply()
    fn charge_interest(deps: DepsMut, env: Env) -> Result<Vec<SubMsg>, ContractError> {
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

    /// Handler for `ExecuteMsg::Repay`
    pub fn repay(mut deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let funds_sent = validate_funds(&info.funds, &cfg.market_token)?;

        let debt = query::btoken_balance(deps.as_ref(), &cfg, &info.sender)?;
        // If there are more tokens sent then there are to repay, burn only desired
        // amount and return the difference
        let repay_amount = std::cmp::min(funds_sent, debt);

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps.branch(), env)?;
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
        if funds > btokens_balance {
            return Err(ContractError::LiquidationInsufficientBTokens {
                account: account.to_string(),
                btokens: btokens_balance,
            });
        }

        let mut response = Response::new();

        // Create rebase messagess for tokens based on interest and supply
        let charge_msgs = charge_interest(deps, env)?;
        if !charge_msgs.is_empty() {
            response = response.add_submessages(charge_msgs);
        }

        // TODO: Return overpay?
        let msg = to_binary(&lendex_token::msg::ExecuteMsg::BurnFrom {
            owner: account.to_string(),
            amount: lendex_token::DisplayAmount::raw(amount),
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
    /// it assumes that amount is in common denom (from CA)
    pub fn transfer_from(
        deps: DepsMut,
        info: MessageInfo,
        source: Addr,
        destination: Addr,
        amount: Uint128,
        liquidation_price: Decimal,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        if cfg.credit_agency != info.sender {
            return Err(ContractError::LiquidationRequiresCreditAgency {});
        }

        // calculate repaid value
        let price_rate = query::price_local_per_common(deps.as_ref(), &cfg)?;
        let repaid_value = cr_utils::divide(amount * price_rate, liquidation_price);

        // transfer claimed amount of ltokens from account source to destination
        let msg = to_binary(&lendex_token::msg::ExecuteMsg::TransferFrom {
            sender: source.to_string(),
            recipient: destination.to_string(),
            amount: lendex_token::DisplayAmount::raw(repaid_value),
        })?;
        let transfer_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: cfg.ltoken_contract.to_string(),
            msg,
            funds: vec![],
        });

        Ok(Response::new()
            .add_attribute("action", "transfer_from")
            .add_attribute("from", source)
            .add_attribute("to", destination)
            .add_submessage(transfer_msg))
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;
    let res = match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?)?,
        TransferableAmount { token, account } => {
            let token = deps.api.addr_validate(&token)?;
            to_binary(&query::transferable_amount(deps, token, account)?)?
        }
        Interest {} => to_binary(&query::calculate_interest(deps)?)?,
        CreditLine { account } => {
            let account = deps.api.addr_validate(&account)?;
            to_binary(&query::credit_line(deps, account)?)?
        }
    };
    Ok(res)
}

mod query {
    use super::*;

    use cosmwasm_std::{Decimal, Uint128};
    use cw20::BalanceResponse;
    use lendex_oracle::msg::{PriceResponse, QueryMsg as OracleQueryMsg};
    use lendex_token::msg::{QueryMsg as TokenQueryMsg, TokenInfoResponse};

    use crate::msg::InterestResponse;
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
    ) -> Result<Uint128, ContractError> {
        Ok(token_balance(deps, &config.btoken_contract, account.to_string())?.balance)
    }

    fn ltoken_balance(
        deps: Deps,
        config: &Config,
        account: impl ToString,
    ) -> Result<Uint128, ContractError> {
        Ok(token_balance(deps, &config.ltoken_contract, account.to_string())?.balance)
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

    pub fn calculate_interest(deps: Deps) -> Result<InterestResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        interest(&config, &token_info(deps, &config)?)
    }

    pub fn token_info(deps: Deps, config: &Config) -> Result<TokensInfo, ContractError> {
        let ltoken_contract = &config.ltoken_contract;
        let ltoken: TokenInfoResponse = deps
            .querier
            .query_wasm_smart(ltoken_contract, &TokenQueryMsg::TokenInfo {})?;
        let btoken_contract = &config.btoken_contract;
        let btoken: TokenInfoResponse = deps
            .querier
            .query_wasm_smart(btoken_contract, &TokenQueryMsg::TokenInfo {})?;
        Ok(TokensInfo { ltoken, btoken })
    }

    /// Handler for `QueryMsg::Interest`
    pub fn interest(
        config: &Config,
        tokens_info: &TokensInfo,
    ) -> Result<InterestResponse, ContractError> {
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

    /// Ratio is for sell market_token / buy common_token
    pub fn price_local_per_common(deps: Deps, config: &Config) -> Result<Decimal, ContractError> {
        // If denoms are the same, just return 1:1
        if config.common_token == config.market_token {
            Ok(Decimal::one())
        } else {
            let price_response: PriceResponse = deps.querier.query_wasm_smart(
                config.price_oracle.clone(),
                &OracleQueryMsg::Price {
                    sell: config.market_token.clone(),
                    buy: config.common_token.clone(),
                },
            )?;
            Ok(price_response.rate)
        }
    }

    /// Handler for `QueryMsg::CreditLine`
    pub fn credit_line(deps: Deps, account: Addr) -> Result<CreditLineResponse, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        let collateral = ltoken_balance(deps, &config, &account)?;
        let debt = btoken_balance(deps, &config, &account)?;
        if collateral.is_zero() && debt.is_zero() {
            return Ok(CreditLineResponse::zero());
        }

        let price_ratio = price_local_per_common(deps, &config)?;
        let collateral = collateral * price_ratio;
        let debt = debt * price_ratio;
        let credit_line = collateral * config.collateral_ratio;
        Ok(CreditLineResponse {
            collateral,
            debt,
            credit_line,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn price_ratio_doesnt_need_query_if_common_token_matches_market_token() {
            let deps = mock_dependencies();
            let market_token = "market_token".to_owned();
            let config = Config {
                ltoken_contract: Addr::unchecked("ltoken_contract"),
                btoken_contract: Addr::unchecked("btoken_contract"),
                name: "lendex".to_owned(),
                symbol: "LDX".to_owned(),
                decimals: 9,
                token_id: 2,
                market_token: market_token.clone(),
                rates: Interest::Linear {
                    base: Decimal::percent(3),
                    slope: Decimal::percent(20),
                },
                interest_charge_period: 300,
                last_charged: 300,
                common_token: market_token,
                collateral_ratio: Decimal::percent(50),
                price_oracle: "price_oracle".to_owned(),
                credit_agency: Addr::unchecked("credit_agency"),
            };
            // common_token is same as market_token
            let ratio = price_ratio_from_oracle(deps.as_ref(), &config).unwrap();
            assert_eq!(ratio, Decimal::one());
        }
    }
}
