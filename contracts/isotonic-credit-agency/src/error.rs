use cosmwasm_std::{Addr, DivideByZeroError, OverflowError, StdError, Uint128};
use utils::coin::Coin;
use utils::{coin::CoinError, credit_line::InvalidCommonTokenDenom, price::PriceError};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Price(#[from] PriceError),

    #[error("{0}")]
    Coin(#[from] CoinError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("The liquidation fee needs to be larger than 0")]
    InvalidLiquidationFee {},

    #[error("The liquidation initiation fee needs to be larger than 0")]
    InvalidLiquidationInitiationFee {},

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

    #[error("{market}: Market either does not exist or is not active yet")]
    MarketSearchError { market: String },

    #[error("{address} is not on a market {market}")]
    NotOnMarket { address: Addr, market: Addr },

    #[error("{address} has dept on market {market} of {debt:?}")]
    DebtOnMarket {
        address: Addr,
        market: Addr,
        debt: Coin,
    },

    #[error("Not enough credit line left after operation, total dept: {debt}, total credit line: credit_line, total collateral: {collateral}")]
    NotEnoughCollat {
        debt: Uint128,
        credit_line: Uint128,
        collateral: Uint128,
    },

    #[error("Cw20 tokens are not supported yet")]
    Cw20TokensNotSupported,

    #[error("Repaying loan using collateral not allowed with these values - the account could end up undercollateralized")]
    RepayingLoanUsingCollateralFailed {},

    #[error("{0}")]
    DivisionByZero(#[from] DivideByZeroError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),
}
