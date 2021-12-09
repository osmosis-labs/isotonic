use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const TOTAL_SUPPLY: Item<Uint128> = Item::new("total_supply");
pub const CONTROLLER: Item<Addr> = Item::new("controller");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
pub const MULTIPLIER: Item<Decimal> = Item::new("multiplier");
