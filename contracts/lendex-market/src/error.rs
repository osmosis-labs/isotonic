use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

use utils::price::PriceError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Price(#[from] PriceError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unrecognised reply id: {0}")]
    UnrecognisedReply(u64),

    #[error("Unrecognised token: {0}")]
    UnrecognisedToken(String),

    #[error("Invalid reply from submessage {id}, {err}")]
    ReplyParseFailure { id: u64, err: String },

    #[error("No funds sent")]
    NoFundsSent {},

    #[error("Sent too many denoms, must deposit only '{0}' in the lending pool")]
    ExtraDenoms(String),

    #[error("Sent unsupported token, must deposit '{0}' in the lending pool")]
    InvalidDenom(String),

    #[error("Cannot borrow amount {amount} for {account}")]
    CannotBorrow { amount: Uint128, account: String },

    #[error("Address {account} cannot withdraw {amount}")]
    CannotWithdraw { account: String, amount: Uint128 },

    #[error("Insufficient amount of btokens on account {account}: {btokens} to liquidate debt")]
    LiquidationInsufficientBTokens { account: String, btokens: Uint128 },

    #[error(
        "Unauthorized - Liquidation helpers call requires sender to be a Market's Credit Agency"
    )]
    LiquidationRequiresCreditAgency {},

    #[error("Received invalid invalid common token from another contract, expected: {expected}, got: {actual}")]
    InvalidCommonTokenDenom { expected: String, actual: String },
}
