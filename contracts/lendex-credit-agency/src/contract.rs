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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    let res = match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?)?,
        Market { base_asset } => to_binary(&query::market(deps, env, base_asset)?)?,
        ListMarkets { start_after, limit } => {
            to_binary(&query::list_markets(deps, env, start_after, limit)?)?
        }
    };

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    reply::handle_market_instantiation_response(deps, env, msg)
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

        if let Some(state) = MARKETS.may_load(deps.storage, &market_cfg.base_asset)? {
            use MarketState::*;

            let err = match state {
                Instantiating => ContractError::MarketCreating(market_cfg.base_asset),
                Ready(_) => ContractError::MarketAlreadyExists(market_cfg.base_asset),
            };
            return Err(err);
        }
        MARKETS.save(
            deps.storage,
            &market_cfg.base_asset,
            &MarketState::Instantiating,
        )?;

        let reply_id =
            NEXT_REPLY_ID.update(deps.storage, |id| -> Result<_, StdError> { Ok(id + 1) })?;
        REPLY_IDS.save(deps.storage, reply_id, &market_cfg.base_asset)?;

        let market_msg = lendex_market::msg::InstantiateMsg {
            name: market_cfg.name,
            symbol: market_cfg.symbol,
            decimals: market_cfg.decimals,
            token_id: cfg.lendex_token_id,
            base_asset: market_cfg.base_asset,
            interest_rate: market_cfg.interest_rate,
            distributed_token: market_cfg.distributed_token,
        };
        let market_instantiate = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: cfg.lendex_market_id,
            msg: to_binary(&market_msg)?,
            funds: vec![],
            label: format!("market_contract_{}", env.contract.address),
        };

        Ok(Response::new()
            .add_attribute("action", "create_market")
            .add_attribute("sender", info.sender)
            .add_submessage(SubMsg::reply_on_success(market_instantiate, reply_id)))
    }
}

mod query {
    use crate::{
        msg::{ListMarketsResponse, MarketResponse},
        state::MARKETS,
    };

    use super::*;

    pub fn market(
        deps: Deps,
        _env: Env,
        base_asset: String,
    ) -> Result<MarketResponse, ContractError> {
        // TODO: check expiration

        let state = MARKETS
            .may_load(deps.storage, &base_asset)?
            .ok_or_else(|| ContractError::NoMarket(base_asset.clone()))?;

        let addr = state
            .to_addr()
            .ok_or_else(|| ContractError::MarketCreating(base_asset.clone()))?;

        Ok(MarketResponse {
            base_asset,
            market: addr,
        })
    }

    pub fn list_markets(
        _deps: Deps,
        _env: Env,
        _start_after: Option<String>,
        _limit: Option<u32>,
    ) -> Result<ListMarketsResponse, ContractError> {
        todo!()
    }
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

        let base_asset = REPLY_IDS.load(deps.storage, id)?;
        let addr = deps.api.addr_validate(&res.contract_address)?;

        MARKETS.save(deps.storage, &base_asset, &MarketState::Ready(addr.clone()))?;

        Ok(Response::new().add_attribute(format!("market_{}", base_asset), addr))
    }
}
