use crate::token::Token;
use cosmwasm_std::{Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
}

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CoinError {
    #[error("Operation {operation} is not allowed, because denoms does not match: {denom1} {denom2}")]
    IncorrectDenoms { operation: String, denom1: Token, denom2: Token },
}

use std::ops::Mul;

impl Mul<Decimal> for Coin {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self::Output {
        Self { denom: self.denom, amount: self.amount * rhs }
    }
}
