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

/// Helper that multiplies coins amount in sell denom times proper price rate
/// Returns error, if Coin.denom != Price.sell_denom
/// Inverted price can't be just returned, because price is a weighted average
pub fn coin_times_price_rate(coin: &Coin, price: &PriceRate) -> Result<Coin, PriceError> {
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
    #[error("Calculating price failed because incorrect denom was used: {incorrect} instead of {correct}")]
    MulPrice { incorrect: String, correct: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::coin;

    #[test]
    fn price_rate_correct_denom() {
        let price_rate = PriceRate {
            buy_denom: "USD".to_owned(),
            sell_denom: "EUR".to_owned(),
            rate_sell_per_buy: Decimal::percent(110),
        };
        let eur_coin = coin(100, "EUR");
        let usd_coin = coin_times_price_rate(&eur_coin, &price_rate).unwrap();
        assert_eq!(usd_coin, coin(110, "USD"));
    }

    #[test]
    fn price_rate_wrong_buy_denom() {
        let price_rate = PriceRate {
            buy_denom: "USD".to_owned(),
            sell_denom: "EUR".to_owned(),
            rate_sell_per_buy: Decimal::percent(110),
        };
        let usd_coin = coin(100, "USD");
        let err = coin_times_price_rate(&usd_coin, &price_rate).unwrap_err();
        assert_eq!(
            PriceError::MulPrice {
                incorrect: "USD".to_owned(),
                correct: "EUR".to_owned()
            },
            err
        );
    }

    #[test]
    fn price_rate_incorrect_denom() {
        let price_rate = PriceRate {
            buy_denom: "USD".to_owned(),
            sell_denom: "EUR".to_owned(),
            rate_sell_per_buy: Decimal::percent(110),
        };
        let pln_coin = coin(100, "PLN");
        let err = coin_times_price_rate(&pln_coin, &price_rate).unwrap_err();
        assert_eq!(
            PriceError::MulPrice {
                incorrect: "PLN".to_owned(),
                correct: "EUR".to_owned()
            },
            err
        );
    }
}
