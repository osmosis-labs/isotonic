use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utils::interest::ValidatedInterest;

use cosmwasm_std::{Addr, Decimal, Uint128};
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
    /// Denom for current market
    pub market_token: String,
    /// An optional cap on total number of tokens deposited into the market
    pub market_cap: Option<Uint128>,
    /// Interest rate calculation
    pub rates: ValidatedInterest,
    pub interest_charge_period: u64,
    pub last_charged: u64,
    /// Denom common amongst markets within same Credit Agency
    pub common_token: String,
    pub collateral_ratio: Decimal,
    /// Address of Oracle's contract
    pub price_oracle: String,
    /// Address of Credit Agency
    pub credit_agency: Addr,
    pub reserve_factor: Decimal,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const RESERVE: Item<Uint128> = Item::new("reserve");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokensInfo {
    pub ltoken: TokenInfoResponse,
    pub btoken: TokenInfoResponse,
}
