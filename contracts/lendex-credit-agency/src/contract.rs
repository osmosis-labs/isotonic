#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};
use cw0::parse_reply_instantiate_data;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, NEXT_REPLY_ID};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lendex-credit-agency";
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
        lendex_market_id: msg.lendex_market_id,
        lendex_token_id: msg.lendex_token_id,
        reward_token: msg.reward_token,
        common_token: msg.common_token,
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
        CreateMarket(market_cfg) => exec::create_market(deps, env, info, market_cfg),
    }
}

mod exec {
    use super::*;

    use cosmwasm_std::{ensure_eq, StdError, SubMsg, WasmMsg};

    use crate::{
        msg::MarketConfig,
        state::{MarketState, MARKETS, REPLY_IDS},
    };

    pub fn create_market(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        market_cfg: MarketConfig,
    ) -> Result<Response, ContractError> {
        let cfg = CONFIG.load(deps.storage)?;
        ensure_eq!(
            info.sender,
            cfg.gov_contract,
            ContractError::Unauthorized {}
        );

        if let Some(state) = MARKETS.may_load(deps.storage, &market_cfg.market_token)? {
            use MarketState::*;

            let err = match state {
                Instantiating => ContractError::MarketCreating(market_cfg.market_token),
                Ready(_) => ContractError::MarketAlreadyExists(market_cfg.market_token),
            };
            return Err(err);
        }
        MARKETS.save(
            deps.storage,
            &market_cfg.market_token,
            &MarketState::Instantiating,
        )?;

        let reply_id =
            NEXT_REPLY_ID.update(deps.storage, |id| -> Result<_, StdError> { Ok(id + 1) })?;
        REPLY_IDS.save(deps.storage, reply_id, &market_cfg.market_token)?;

        let market_msg = lendex_market::msg::InstantiateMsg {
            name: market_cfg.name,
            symbol: market_cfg.symbol,
            decimals: market_cfg.decimals,
            token_id: cfg.lendex_token_id,
            market_token: market_cfg.market_token.clone(),
            interest_rate: market_cfg.interest_rate,
            distributed_token: cfg.reward_token,
            interest_charge_period: market_cfg.interest_charge_period,
            common_token: cfg.common_token,
            collateral_ratio: market_cfg.collateral_ratio,
            price_oracle: market_cfg.price_oracle,
        };
        let market_instantiate = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: cfg.lendex_market_id,
            msg: to_binary(&market_msg)?,
            funds: vec![],
            label: format!("market_contract_{}", market_cfg.market_token),
        };

        Ok(Response::new()
            .add_attribute("action", "create_market")
            .add_attribute("sender", info.sender)
            .add_submessage(SubMsg::reply_on_success(market_instantiate, reply_id)))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    let res = match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?)?,
        Market { market_token } => to_binary(&query::market(deps, market_token)?)?,
        ListMarkets { start_after, limit } => {
            to_binary(&query::list_markets(deps, start_after, limit)?)?
        }
        TotalCreditLine { account } => to_binary(&query::total_credit_line(deps, account)?)?,
    };

    Ok(res)
}

mod query {
    use cosmwasm_std::{Order, StdResult};
    use cw_storage_plus::Bound;
    use lendex_market::msg::{CreditLineResponse, QueryMsg as MarketQueryMsg};

    use crate::{
        msg::{ListMarketsResponse, MarketResponse},
        state::MARKETS,
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
            market_token,
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
        let start = start_after.map(|addr| Bound::exclusive(addr.as_str()));

        let markets: StdResult<Vec<_>> = MARKETS
            .range_de(deps.storage, start, None, Order::Ascending)
            .map(|m| {
                let (market_token, market) = m?;

                let result = market.to_addr().map(|addr| MarketResponse {
                    market_token,
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
        let total_credit_line = list_markets(deps, None, None)?
            .markets
            .iter()
            .map(|market| {
                let price_response: CreditLineResponse = deps.querier.query_wasm_smart(
                    market.market.clone(),
                    &MarketQueryMsg::CreditLine {
                        account: account.clone(),
                    },
                )?;
                Ok(price_response)
            })
            .collect::<Result<Vec<CreditLineResponse>, ContractError>>()?
            .iter()
            .sum();
        Ok(total_credit_line)
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
