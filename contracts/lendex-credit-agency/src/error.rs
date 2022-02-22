use cosmwasm_std::{StdError, Uint128};
use utils::{credit_line::InvalidCommonTokenDenom, price::PriceError};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Price(#[from] PriceError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Creating Market failure - collateral ratio must be lower than liquidation price")]
    MarketCfgCollateralFailure {},

    #[error("Unrecognised reply id: {0}")]
    UnrecognisedReply(u64),

    #[error("Invalid reply from submessage {id}, {err}")]
    ReplyParseFailure { id: u64, err: String },

    #[error("No market set up for base asset {0}")]
    NoMarket(String),

    #[error("A market for base asset {0} is still being created")]
    MarketCreating(String),

    #[error("A market for base asset {0} already exists")]
    MarketAlreadyExists(String),

    #[error("Account cannot be liquidated as it does not have more debt then credit line")]
    LiquidationNotAllowed {},

    #[error("Only one denom can be sent for liquidation")]
    LiquidationOnlyOneDenomRequired {},

    #[error("Incorrect denom sent to liquidation: {incorrect} instead of {common}")]
    LiquidationIncorrectDenom { incorrect: String, common: String },

    #[error("Insufficient amount of tokens sent to liquidation: {sent} instead of {required}")]
    LiquidationInsufficientTokens { sent: Uint128, required: Uint128 },

    #[error(
        "Insufficient amount of btokens on account {account}: {btokens} with debt of high {debt}"
    )]
    LiquidationInsufficientBTokens {
        account: String,
        btokens: Uint128,
        debt: Uint128,
    },

    #[error("{0}")]
    InvalidCommonTokenDenom(#[from] InvalidCommonTokenDenom),
}
