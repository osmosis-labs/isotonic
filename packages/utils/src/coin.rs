use cosmwasm_std::{Decimal, Uint128, Coin as StdCoin, coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Mul;
use thiserror::Error;

use crate::token::Token;

/// Universal coin type which is either a native coin, or cw20 coin
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord)]
pub struct Coin {
    pub denom: Token,
    pub amount: Uint128,
}

impl Coin {
    pub fn new(amount: u128, denom: Token) -> Self {
        Coin {
            amount: Uint128::new(amount),
            denom,
        }
    }

    pub fn new_native(amount: u128, denom: &str) -> Self {
        Self::new(amount, Token::new_native(denom))
    }

    pub fn new_cw20(amount: u128, addr: &str) -> Self {
        Self::new(amount, Token::new_cw20(addr))
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, CoinError> {
        if self.denom == rhs.denom {
            Ok(Self { amount: self.amount + rhs.amount, denom: self.denom })
        } else {
            Err(CoinError::IncorrectDenoms { operation: "addition".to_owned(), denom1: self.denom, denom2: rhs.denom })
        }
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, CoinError> {
        if self.denom == rhs.denom {
            Ok(Self { amount: self.amount - rhs.amount, denom: self.denom })
        } else {
            Err(CoinError::IncorrectDenoms { operation: "subtraction".to_owned(), denom1: self.denom, denom2: rhs.denom })
        }
    }

    pub fn into_std_coin(&self) -> Result<StdCoin, CoinError> {
        match &self.denom {
            Token::Native(denom) => Ok(coin(self.amount.u128(), denom)),
            _ => Err(CoinError::ConvertBadToken {})
        }
    }
}

impl Mul<Decimal> for Coin {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self { denom: self.denom, amount: self.amount * rhs }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum CoinError {
    #[error("Operation {operation} is not allowed, because denoms does not match: {denom1} {denom2}")]
    IncorrectDenoms { operation: String, denom1: Token, denom2: Token },

    #[error("cosmwasm_std::Coin type cannot be created from Cw20 coin")]
    ConvertBadToken {},
}

