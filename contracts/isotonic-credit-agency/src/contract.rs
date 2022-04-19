#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Env, MessageInfo, Reply};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use osmo_bindings::{OsmosisMsg, OsmosisQuery};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, SudoMsg};
use crate::state::{Config, CONFIG, NEXT_REPLY_ID};

use either::Either;
use utils::token::Token;

pub type Response = cosmwasm_std::Response<OsmosisMsg>;
pub type SubMsg = cosmwasm_std::SubMsg<OsmosisMsg>;
pub type Deps<'a> = cosmwasm_std::Deps<'a, OsmosisQuery>;
pub type DepsMut<'a> = cosmwasm_std::DepsMut<'a, OsmosisQuery>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:isotonic-credit-agency";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        gov_contract: deps.api.addr_validate(&msg.gov_contract)?,
        isotonic_market_id: msg.isotonic_market_id,
        isotonic_token_id: msg.isotonic_token_id,
        reward_token: msg
            .reward_token
            .native()
            .ok_or(ContractError::Cw20TokensNotSupported)?,
        common_token: msg
            .common_token
            .native()
            .ok_or(ContractError::Cw20TokensNotSupported)?,
        liquidation_price: msg.liquidation_price,
    };
    CONFIG.save(deps.storage, &cfg)?;
    NEXT_REPLY_ID.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
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
        CreateMarket(market_cfg) => execute::create_market(deps, env, info, market_cfg),
        Liquidate {
            account,
            collateral_denom,
        } => {
            let account = deps.api.addr_validate(&account)?;
            execute::liquidate(
                deps,
                info,
                account,
                collateral_denom
                    .native()
                    .ok_or(ContractError::Cw20TokensNotSupported)?,
            )
        }
        EnterMarket { account } => {
            let account = deps.api.addr_validate(&account)?;
            execute::enter_market(deps, info, account)
        }
        ExitMarket { market } => {
            let market = deps.api.addr_validate(&market)?;
            execute::exit_market(deps, info, market)
        }
        RepayWithCollateral {
            max_collateral,
            amount_to_repay,
        } => execute::repay_with_collateral(deps, info.sender, max_collateral, amount_to_repay),
    }
}

mod execute {
    use super::*;

    use cosmwasm_std::{coin, ensure_eq, Coin, StdError, SubMsg, WasmMsg};
    use utils::{
        credit_line::{CreditLineResponse, CreditLineValues},
        price::{coin_times_price_rate, PriceRate},
    };

    use crate::{
        msg::MarketConfig,
        state::{MarketState, ENTERED_MARKETS, MARKETS, REPLY_IDS},
    };
    use isotonic_market::{
        msg::{ExecuteMsg as MarketExecuteMsg, QueryMsg as MarketQueryMsg},
        state::Config as MarketConfiguration,
    };

    pub fn create_market(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        market_cfg: MarketConfig,
    ) -> Result<Response, ContractError> {
        let market_token = market_cfg
            .market_token
            .native()
            .ok_or(ContractError::Cw20TokensNotSupported)?;

        let cfg = CONFIG.load(deps.storage)?;
        ensure_eq!(
            info.sender,
            cfg.gov_contract,
            ContractError::Unauthorized {}
        );

        // Collateral ratio must be lower then liquidation price, otherwise
        // liquidation could decrese debt less then it decreases potential credit.
        if market_cfg.collateral_ratio >= cfg.liquidation_price {
            return Err(ContractError::MarketCfgCollateralFailure {});
        }

        if let Some(state) = MARKETS.may_load(deps.storage, &market_token)? {
            use MarketState::*;

            let err = match state {
                Instantiating => ContractError::MarketCreating(market_token),
                Ready(_) => ContractError::MarketAlreadyExists(market_token),
            };
            return Err(err);
        }
        MARKETS.save(deps.storage, &market_token, &MarketState::Instantiating)?;

        let reply_id =
            NEXT_REPLY_ID.update(deps.storage, |id| -> Result<_, StdError> { Ok(id + 1) })?;
        REPLY_IDS.save(deps.storage, reply_id, &market_token)?;

        let market_msg = isotonic_market::msg::InstantiateMsg {
            name: market_cfg.name,
            symbol: market_cfg.symbol,
            decimals: market_cfg.decimals,
            token_id: cfg.isotonic_token_id,
            market_token: Token::Native(market_token.clone()),
            market_cap: market_cfg.market_cap,
            interest_rate: market_cfg.interest_rate,
            distributed_token: Token::Native(cfg.reward_token),
            interest_charge_period: market_cfg.interest_charge_period,
            common_token: Token::Native(cfg.common_token),
            collateral_ratio: market_cfg.collateral_ratio,
            price_oracle: market_cfg.price_oracle,
            reserve_factor: market_cfg.reserve_factor,
        };
        let market_instantiate = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: cfg.isotonic_market_id,
            msg: to_binary(&market_msg)?,
            funds: vec![],
            label: format!("market_contract_{}", market_token),
        };

        Ok(Response::new()
            .add_attribute("action", "create_market")
            .add_attribute("sender", info.sender)
            .add_submessage(SubMsg::reply_on_success(market_instantiate, reply_id)))
    }

    pub fn liquidate(
        deps: DepsMut,
        info: MessageInfo,
        account: Addr,
        collateral_denom: String,
    ) -> Result<Response, ContractError> {
        // assert that only one denom was sent and it matches the existing market
        if info.funds.is_empty() || info.funds.len() != 1 {
            return Err(ContractError::LiquidationOnlyOneDenomRequired {});
        }
        let funds = info.funds[0].clone();
        let cfg = CONFIG.load(deps.storage)?;
        // assert that given account actually has more debt then credit
        let total_credit_line = query::total_credit_line(deps.as_ref(), account.to_string())?;
        let total_credit_line = total_credit_line.validate(&Token::Native(cfg.common_token))?;
        if total_credit_line.debt <= total_credit_line.credit_line {
            return Err(ContractError::LiquidationNotAllowed {});
        }

        // Count btokens and burn then on account
        // this requires that market returns error if repaying more then balance
        let debt_market = query::market(deps.as_ref(), funds.denom.clone())?.market;
        let msg = to_binary(&isotonic_market::msg::ExecuteMsg::RepayTo {
            account: account.to_string(),
            amount: funds.amount,
        })?;
        let repay_from_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: debt_market.to_string(),
            msg,
            funds: vec![funds.clone()],
        });

        // find price rate of collateral denom
        let price_response: PriceRate = deps.querier.query_wasm_smart(
            debt_market.to_string(),
            &MarketQueryMsg::PriceMarketLocalPerCommon {},
        )?;

        // find market with wanted collateral_denom
        let collateral_market = query::market(deps.as_ref(), collateral_denom.clone())?.market;

        // transfer claimed amount as reward
        let msg = to_binary(&isotonic_market::msg::ExecuteMsg::TransferFrom {
            source: account.to_string(),
            destination: info.sender.to_string(),
            // transfer repaid amount represented as amount of common tokens, which is
            // calculated into collateral_denom's amount later in the market
            amount: coin_times_price_rate(&funds, &price_response)?.amount,
            liquidation_price: cfg.liquidation_price,
        })?;
        let transfer_from_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: collateral_market.to_string(),
            msg,
            funds: vec![],
        });

        Ok(Response::new()
            .add_attribute("action", "liquidate")
            .add_attribute("liquidator", info.sender)
            .add_attribute("account", account)
            .add_attribute("collateral_denom", collateral_denom)
            .add_submessage(repay_from_msg)
            .add_submessage(transfer_from_msg))
    }

    pub fn enter_market(
        deps: DepsMut,
        info: MessageInfo,
        account: Addr,
    ) -> Result<Response, ContractError> {
        let market = info.sender;

        ENTERED_MARKETS.update(deps.storage, &account, |maybe_set| -> Result<_, StdError> {
            let mut markets = maybe_set.unwrap_or_default();
            markets.insert(market.clone());
            Ok(markets)
        })?;

        Ok(Response::new()
            .add_attribute("action", "enter_market")
            .add_attribute("market", market)
            .add_attribute("account", account))
    }

    pub fn exit_market(
        deps: DepsMut,
        info: MessageInfo,
        market: Addr,
    ) -> Result<Response, ContractError> {
        let common_token = CONFIG.load(deps.storage)?.common_token;
        let mut markets = ENTERED_MARKETS
            .may_load(deps.storage, &info.sender)?
            .unwrap_or_default();

        if !markets.contains(&market) {
            return Err(ContractError::NotOnMarket {
                address: info.sender,
                market: market.clone(),
            });
        }

        let market_credit_line: CreditLineResponse = deps.querier.query_wasm_smart(
            market.clone(),
            &MarketQueryMsg::CreditLine {
                account: info.sender.to_string(),
            },
        )?;

        if !market_credit_line.debt.amount.is_zero() {
            return Err(ContractError::DebtOnMarket {
                address: info.sender,
                market,
                debt: market_credit_line.debt,
            });
        }

        // It can be removed before everything is checked, as if anything would fail, this removal
        // would not be applied. And in `reduced_credit_line` we don't want this market to be
        // there, so removing early.
        markets.remove(&market);

        let reduced_credit_line = markets
            .iter()
            .map(|market| -> Result<CreditLineValues, ContractError> {
                let price_response: CreditLineResponse = deps.querier.query_wasm_smart(
                    market.clone(),
                    &MarketQueryMsg::CreditLine {
                        account: info.sender.to_string(),
                    },
                )?;
                let price_response =
                    price_response.validate(&Token::Native(common_token.clone()))?;
                Ok(price_response)
            })
            .try_fold(
                CreditLineValues::zero(),
                |total, credit_line| match credit_line {
                    Ok(cl) => Ok(total + cl),
                    Err(err) => Err(err),
                },
            )?;

        if reduced_credit_line.credit_line < reduced_credit_line.debt {
            return Err(ContractError::NotEnoughCollat {
                debt: reduced_credit_line.debt,
                credit_line: reduced_credit_line.credit_line,
                collateral: reduced_credit_line.collateral,
            });
        }

        ENTERED_MARKETS.save(deps.storage, &info.sender, &markets)?;

        Ok(Response::new()
            .add_attribute("action", "exit_market")
            .add_attribute("market", market)
            .add_attribute("account", info.sender))
    }

    pub fn repay_with_collateral(
        deps: DepsMut,
        sender: Addr,
        max_collateral: Coin,
        amount_to_repay: Coin,
    ) -> Result<Response, ContractError> {
        let collateral_market = query::market(deps.as_ref(), max_collateral.clone().denom)?.market;
        let debt_market = query::market(deps.as_ref(), amount_to_repay.clone().denom)?.market;

        let markets = ENTERED_MARKETS
            .may_load(deps.storage, &sender)?
            .unwrap_or_default();

        if !markets.contains(&collateral_market) {
            return Err(ContractError::NotOnMarket {
                address: sender,
                market: collateral_market,
            });
        } else if !markets.contains(&debt_market) {
            return Err(ContractError::NotOnMarket {
                address: sender,
                market: debt_market,
            });
        }

        let tcr = query::total_credit_line(deps.as_ref(), sender.to_string())?;
        let cfg = CONFIG.load(deps.storage)?;

        let collateral_market_cfg: MarketConfiguration = deps
            .querier
            .query_wasm_smart(collateral_market.clone(), &MarketQueryMsg::Configuration {})?;

        let collateral_per_common_rate: PriceRate = deps.querier.query_wasm_smart(
            collateral_market.clone(),
            &MarketQueryMsg::PriceMarketLocalPerCommon {},
        )?;
        let collateral_per_common_rate = collateral_per_common_rate.rate_sell_per_buy;
        let max_collateral = coin(
            (max_collateral.amount * collateral_per_common_rate).u128(),
            cfg.common_token.clone(),
        );

        let debt_per_common_rate: PriceRate = deps.querier.query_wasm_smart(
            debt_market.clone(),
            &MarketQueryMsg::PriceMarketLocalPerCommon {},
        )?;
        let debt_per_common_rate = debt_per_common_rate.rate_sell_per_buy;
        let amount_to_repay_common = coin(
            (amount_to_repay.amount * debt_per_common_rate).u128(),
            cfg.common_token,
        );

        let util_collateral: utils::coin::Coin = max_collateral.clone().into();
        let simulated_credit_line = tcr
            .credit_line
            .checked_sub(util_collateral * collateral_market_cfg.collateral_ratio)?;
        let simulated_debt = tcr.debt.checked_sub(amount_to_repay_common.into())?;
        if simulated_debt > simulated_credit_line {
            return Err(ContractError::RepayingLoanUsingCollateralFailed {});
        }

        let msg = to_binary(&MarketExecuteMsg::SwapWithdrawFrom {
            account: sender.to_string(),
            sell_limit: max_collateral.amount,
            buy: amount_to_repay.clone(),
        })?;
        let swap_withdraw_from_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: collateral_market.to_string(),
            msg,
            funds: vec![],
        });

        let msg = to_binary(&MarketExecuteMsg::RepayTo {
            account: sender.to_string(),
            amount: amount_to_repay.amount,
        })?;
        let repay_to_msg = SubMsg::new(WasmMsg::Execute {
            contract_addr: debt_market.to_string(),
            msg,
            funds: vec![],
        });

        Ok(Response::new()
            .add_submessage(swap_withdraw_from_msg)
            .add_submessage(repay_to_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    let res = match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?)?,
        Market { market_token } => to_binary(&query::market(
            deps,
            market_token
                .native()
                .ok_or(ContractError::Cw20TokensNotSupported)?,
        )?)?,
        ListMarkets { start_after, limit } => to_binary(&query::list_markets(
            deps,
            start_after
                .map(|sa| sa.native().ok_or(ContractError::Cw20TokensNotSupported))
                .transpose()?,
            limit,
        )?)?,
        TotalCreditLine { account } => to_binary(&query::total_credit_line(deps, account)?)?,
        ListEnteredMarkets {
            account,
            start_after,
            limit,
        } => to_binary(&query::entered_markets(deps, account, start_after, limit)?)?,
        IsOnMarket { account, market } => to_binary(&query::is_on_market(deps, account, market)?)?,
    };

    Ok(res)
}

mod query {
    use cosmwasm_std::{Order, StdResult};
    use cw_storage_plus::Bound;
    use isotonic_market::msg::QueryMsg as MarketQueryMsg;
    use utils::credit_line::{CreditLineResponse, CreditLineValues};

    use crate::{
        msg::{
            IsOnMarketResponse, ListEnteredMarketsResponse, ListMarketsResponse, MarketResponse,
        },
        state::{ENTERED_MARKETS, MARKETS},
    };

    use super::*;

    pub fn market(deps: Deps, market_token: String) -> Result<MarketResponse, ContractError> {
        let state = MARKETS
            .may_load(deps.storage, &market_token)?
            .ok_or_else(|| ContractError::NoMarket(market_token.clone()))?;

        let addr = state
            .to_addr()
            .ok_or_else(|| ContractError::MarketCreating(market_token.clone()))?;

        Ok(MarketResponse {
            market_token: Token::Native(market_token),
            market: addr,
        })
    }

    // settings for pagination
    const MAX_LIMIT: u32 = 30;
    const DEFAULT_LIMIT: u32 = 10;

    pub fn list_markets(
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<ListMarketsResponse, ContractError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after
            .as_ref()
            .map(|addr| Bound::exclusive(addr.as_str()));

        let markets: StdResult<Vec<_>> = MARKETS
            .range(deps.storage, start, None, Order::Ascending)
            .map(|m| {
                let (market_token, market) = m?;

                let result = market.to_addr().map(|addr| MarketResponse {
                    market_token: Token::Native(market_token),
                    market: addr,
                });

                Ok(result)
            })
            .filter_map(|m| m.transpose())
            .take(limit)
            .collect();

        Ok(ListMarketsResponse { markets: markets? })
    }

    /// Handler for `QueryMsg::TotalCreditLine`
    pub fn total_credit_line(
        deps: Deps,
        account: String,
    ) -> Result<CreditLineResponse, ContractError> {
        let common_token = CONFIG.load(deps.storage)?.common_token;
        let markets = ENTERED_MARKETS
            .may_load(deps.storage, &Addr::unchecked(&account))?
            .unwrap_or_default();

        let total_credit_line: CreditLineValues = markets
            .into_iter()
            .map(|market| {
                let price_response: CreditLineResponse = deps.querier.query_wasm_smart(
                    market,
                    &MarketQueryMsg::CreditLine {
                        account: account.clone(),
                    },
                )?;
                let price_response =
                    price_response.validate(&Token::Native(common_token.clone()))?;
                Ok(price_response)
            })
            .collect::<Result<Vec<CreditLineValues>, ContractError>>()?
            .iter()
            .sum();
        Ok(total_credit_line.make_response(Token::Native(common_token)))
    }

    pub fn entered_markets(
        deps: Deps,
        account: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<ListEnteredMarketsResponse, ContractError> {
        let account = Addr::unchecked(&account);
        let markets = ENTERED_MARKETS
            .may_load(deps.storage, &account)?
            .unwrap_or_default()
            .into_iter();

        let markets = if let Some(start_after) = &start_after {
            Either::Left(
                markets
                    .skip_while(move |market| market != start_after)
                    .skip(1),
            )
        } else {
            Either::Right(markets)
        };

        let markets = markets.take(limit.unwrap_or(u32::MAX) as usize).collect();

        Ok(ListEnteredMarketsResponse { markets })
    }

    pub fn is_on_market(
        deps: Deps,
        account: String,
        market: String,
    ) -> Result<IsOnMarketResponse, ContractError> {
        let account = Addr::unchecked(&account);
        let market = Addr::unchecked(&market);
        let markets = ENTERED_MARKETS
            .may_load(deps.storage, &account)?
            .unwrap_or_default();

        Ok(IsOnMarketResponse {
            participating: markets.contains(&market),
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    reply::handle_market_instantiation_response(deps, env, msg)
}

mod reply {
    use crate::state::{MarketState, MARKETS, REPLY_IDS};

    use super::*;

    pub fn handle_market_instantiation_response(
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

        let market_token = REPLY_IDS.load(deps.storage, id)?;
        let addr = deps.api.addr_validate(&res.contract_address)?;

        MARKETS.save(
            deps.storage,
            &market_token,
            &MarketState::Ready(addr.clone()),
        )?;

        Ok(Response::new().add_attribute(format!("market_{}", market_token), addr))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    use SudoMsg::*;
    match msg {
        AdjustMarketId { new_market_id } => sudo::adjust_market_id(deps, new_market_id),
        AdjustTokenId { new_token_id } => sudo::adjust_token_id(deps, new_token_id),
        AdjustCommonToken { new_common_token } => sudo::adjust_common_token(
            deps,
            new_common_token
                .native()
                .ok_or(ContractError::Cw20TokensNotSupported)?,
        ),
        MigrateMarket {
            contract,
            migrate_msg,
        } => sudo::migrate_market(deps, contract, migrate_msg),
    }
}

mod sudo {
    use super::*;
    use crate::state::{MarketState, MARKETS};

    use cosmwasm_std::{Order, WasmMsg};

    use isotonic_market::msg::{ExecuteMsg as MarketExecuteMsg, MigrateMsg as MarketMigrateMsg};

    pub fn adjust_market_id(deps: DepsMut, new_market_id: u64) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.isotonic_market_id = new_market_id;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_token_id(deps: DepsMut, new_token_id: u64) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.isotonic_token_id = new_token_id;
        CONFIG.save(deps.storage, &cfg)?;
        Ok(Response::new())
    }

    pub fn adjust_common_token(
        deps: DepsMut,
        new_common_token: String,
    ) -> Result<Response, ContractError> {
        let mut cfg = CONFIG.load(deps.storage)?;
        cfg.common_token = new_common_token.clone();
        CONFIG.save(deps.storage, &cfg)?;

        let msg = to_binary(&MarketExecuteMsg::AdjustCommonToken {
            new_token: Token::Native(new_common_token),
        })?;
        let messages = MARKETS
            .range(deps.storage, None, None, Order::Ascending)
            .filter_map(|m| match m {
                Ok((_, MarketState::Ready(addr))) => Some(SubMsg::new(WasmMsg::Execute {
                    contract_addr: addr.to_string(),
                    msg: msg.clone(),
                    funds: vec![],
                })),
                _ => None,
            })
            .collect::<Vec<SubMsg>>();

        Ok(Response::new().add_submessages(messages))
    }

    fn find_market(deps: Deps, market_addr: &Addr) -> bool {
        let found = MARKETS
            .range(deps.storage, None, None, Order::Ascending)
            .find(|m| match m {
                Ok((_, MarketState::Ready(addr))) => market_addr == addr,
                _ => false,
            });
        found.is_some()
    }

    pub fn migrate_market(
        deps: DepsMut,
        contract_addr: String,
        migrate_msg: MarketMigrateMsg,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        let contract = deps.api.addr_validate(&contract_addr)?;

        if !find_market(deps.as_ref(), &contract) {
            return Err(ContractError::MarketSearchError {
                market: contract_addr,
            });
        }

        Ok(Response::new().add_message(WasmMsg::Migrate {
            contract_addr,
            new_code_id: cfg.isotonic_market_id,
            msg: to_binary(&migrate_msg)?,
        }))
    }
}
