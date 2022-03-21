use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use utils::time::Expiration;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub controller: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PriceRecord {
    pub rate: Decimal,
    pub expires: Expiration,
}

pub const CONFIG: Item<Config> = Item::new("config");
/// The list of all pools the oracle is aware of. The denoms are expected to be given in ascending order
pub const POOLS: Map<(&str, &str), u64> = Map::new("prices");
