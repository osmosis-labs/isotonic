use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utils::interest::Interest;

use cosmwasm_std::{Addr, Decimal};
use cw_storage_plus::Item;
use lendex_token::msg::TokenInfoResponse;

pub const SECONDS_IN_YEAR: u128 = 31_556_736;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub ltoken_contract: Addr,
    pub btoken_contract: Addr,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub token_id: u64,
    pub market_token: String,
    /// Interest rate calculation
    pub rates: Interest,
    pub interest_charge_period: u64,
    pub last_charged: u64,
    pub common_token: String,
    pub collateral_ratio: Decimal,
    pub price_oracle: String,
    pub credit_agency: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokensInfo {
    pub ltoken: TokenInfoResponse,
    pub btoken: TokenInfoResponse,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Price {
    pub buy_denom: String,
    pub sell_denom: String,
    pub rate: Decimal,
}
