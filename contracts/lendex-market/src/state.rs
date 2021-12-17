use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utils::interest::Interest;

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub ltoken_contract: Addr,
    pub btoken_contract: Addr,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub token_id: u64,
    pub base_asset: String,
    /// Interest rate calculation
    pub rates: Interest,
}

pub const CONFIG: Item<Config> = Item::new("config");
