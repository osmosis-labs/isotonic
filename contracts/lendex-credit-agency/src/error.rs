use cosmwasm_std::{StdError, Uint128};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

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

    #[error("Cannot liquidate debt if collateral ratio is higher then liquidation price")]
    LiquidationCollateralRatioHigherThenLiquidationPrice {},

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
}
