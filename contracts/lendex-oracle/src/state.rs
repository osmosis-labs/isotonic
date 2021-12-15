use cosmwasm_std::{Addr, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};
use utils::time::{Duration, Expiration};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub oracle: Addr,
    pub maximum_age: Duration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PriceRecord {
    pub rate: Decimal,
    pub expires: Expiration,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const PRICES: Map<(&str, &str), PriceRecord> = Map::new("prices");
