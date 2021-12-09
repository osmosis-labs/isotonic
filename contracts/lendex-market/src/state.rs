use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub ltoken_contract: Addr,
    pub btoken_contract: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
