#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response};
use cw0::parse_reply_instantiate_data;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

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
        ledex_token_id: msg.lendex_token_id,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Execution entry point
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        CreateMarket(market_cfg) => exec::create_market(deps, info, market_cfg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    use QueryMsg::*;

    let res = match msg {
        Configuration {} => to_binary(&CONFIG.load(deps.storage)?)?,
        Market { base_asset } => to_binary(&query::market(deps, env, &base_asset)?)?,
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

    use crate::msg::MarketConfig;

    pub fn create_market(
        _deps: DepsMut,
        _info: MessageInfo,
        _cfg: MarketConfig,
    ) -> Result<Response, ContractError> {
        // TODO: assert caller is gov_contract
        // TODO: create a new unique reply ID
        // TODO: trigger market contract instantiation and ask for a `reply` on success (or always?)
        todo!()
    }
}

mod query {
    use crate::msg::{ListMarketsResponse, MarketResponse};

    use super::*;

    pub fn market(
        _deps: Deps,
        _env: Env,
        _base_asset: &str,
    ) -> Result<MarketResponse, ContractError> {
        todo!()
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
    use super::*;

    pub fn handle_market_instantiation_response(
        _deps: DepsMut,
        _env: Env,
        msg: Reply,
    ) -> Result<Response, ContractError> {
        let id = msg.id;
        let _res =
            parse_reply_instantiate_data(msg).map_err(|err| ContractError::ReplyParseFailure {
                id,
                err: err.to_string(),
            })?;

        // TODO: verify msg.id corresponds to a market we're trying to instantiate
        // TODO: store the market addr on success
        // TODO: store some info about failure in case of, well... failure? if it makes sense?
        todo!()
    }
}
