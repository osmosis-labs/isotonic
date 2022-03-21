use crate::token::Token;
use cosmwasm_std::Uint128;
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
}
