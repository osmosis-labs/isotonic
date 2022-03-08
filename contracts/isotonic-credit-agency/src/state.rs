use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: Addr,
    /// The CodeId of the isotonic-market contract
    pub isotonic_market_id: u64,
    /// The CodeId of the isotonic-token contract
    pub isotonic_token_id: u64,
    /// Token denom which would be distributed as reward token to isotonic token holders.
    /// This is `distributed_token` in the market contract.
    pub reward_token: String,
    /// Common Token denom (same for all markets)
    pub common_token: String,
    /// Price for collateral in exchange for paying debt during liquidation
    pub liquidation_price: Decimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum MarketState {
    Instantiating,
    Ready(Addr),
}

impl MarketState {
    pub fn to_addr(self) -> Option<Addr> {
        match self {
            MarketState::Instantiating => None,
            MarketState::Ready(addr) => Some(addr),
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
/// A map of reply_id -> market_token, used to tell which base asset
/// a given instantiating contract will handle
pub const REPLY_IDS: Map<u64, String> = Map::new("reply_ids");
/// The next unused reply ID
pub const NEXT_REPLY_ID: Item<u64> = Item::new("next_reply_id");
/// A map of base asset -> market contract address
pub const MARKETS: Map<&str, MarketState> = Map::new("market");
/// A set of "entered markets" for each account, as in markets in which the account is
/// actively participating.
pub const ENTERED_MARKETS: Map<&Addr, HashSet<Addr>> = Map::new("entered_martkets");
