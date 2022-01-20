use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, Decimal};

use crate::error::PriceError;

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
