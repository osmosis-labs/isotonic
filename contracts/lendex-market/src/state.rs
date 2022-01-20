use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utils::interest::Interest;

use cosmwasm_std::{Addr, Coin, Decimal};
use cw_storage_plus::Item;
use lendex_token::msg::TokenInfoResponse;

use crate::error::PriceError;

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
    /// Interest rate calculation
    pub rates: Interest,
    pub interest_charge_period: u64,
    pub last_charged: u64,
    /// Denom common amongst markets within same Credit Agency
    pub common_token: String,
    pub collateral_ratio: Decimal,
    /// Address of Oracle's contract
    pub price_oracle: String,
    /// Address of Credit Agency
    pub credit_agency: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokensInfo {
    pub ltoken: TokenInfoResponse,
    pub btoken: TokenInfoResponse,
}

// Structure containing price ratio for sell market_token / buy common_token
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Price {
    pub buy_denom: String,
    pub sell_denom: String,
    pub rate: Decimal,
}

// Helper that multiplies coins amount in sell denom times proper price rate
// Returns error, if Coin.denom != Price.sell_denom
pub fn coin_times_price(coin: &Coin, price: &Price) -> Result<Coin, PriceError> {
    if coin.denom == price.sell_denom {
        Ok(Coin {
            amount: coin.amount * price.rate,
            denom: price.buy_denom.clone(),
        })
    } else {
        Err(PriceError::MulPrice {
            incorrect: coin.denom.clone(),
            correct: price.sell_denom.clone(),
        })
    }
}
