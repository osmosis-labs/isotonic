use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Universal token type which is either a native token, or cw20 token
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Hash,
)]
pub enum Token {
    /// Native token of given name
    Native(String),
    /// Cw20 token with its cw20 contract address
    Cw20(String),
}

impl Token {
    pub fn new_native(denom: &str) -> Self {
        Self::Native(denom.to_owned())
    }

    pub fn new_cw20(denom: &str) -> Self {
        Self::Cw20(denom.to_owned())
    }

    /// Return native token name or `None`
    pub fn native(self) -> Option<String> {
        match self {
            Token::Native(token) => Some(token),
            _ => None,
        }
    }

    /// Returns cw20 token address or `None`
    pub fn cw20(self) -> Option<String> {
        match self {
            Token::Cw20(addr) => Some(addr),
            _ => None,
        }
    }

    /// Checks is token is native
    pub fn is_native(&self) -> bool {
        matches!(self, Token::Native(_))
    }

    /// Checks is token is cw20
    pub fn is_cw20(&self) -> bool {
        matches!(self, Token::Cw20(_))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Native(denom) => write!(f, "Native({})", denom),
            Token::Cw20(denom) => write!(f, "Cw20({})", denom),
        }
    }
}
