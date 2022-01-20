use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use cosmwasm_std::{Coin, Decimal};

// Structure containing price ratio for sell market_token / buy common_token
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PriceRate {
    pub buy_denom: String,
    pub sell_denom: String,
    pub rate_sell_per_buy: Decimal,
}

// Helper that multiplies coins amount in sell denom times proper price rate
// Returns error, if Coin.denom != Price.sell_denom
pub fn coin_times_price(coin: &Coin, price: &PriceRate) -> Result<Coin, PriceError> {
    if coin.denom == price.sell_denom {
        Ok(Coin {
            amount: coin.amount * price.rate_sell_per_buy,
            denom: price.buy_denom.clone(),
        })
    } else {
        Err(PriceError::MulPrice {
            incorrect: coin.denom.clone(),
            correct: price.sell_denom.clone(),
        })
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum PriceError {
    #[error("Calucating denom failed because incorrect denom was used: {incorrect} instead of {correct}")]
    MulPrice { incorrect: String, correct: String },
}
