use cosmwasm_std::{OverflowError, StdError, Uint128};
use thiserror::Error;
use utils::interest::InterestError;

use utils::credit_line::InvalidCommonTokenDenom;
use utils::price::PriceError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Price(#[from] PriceError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

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

    #[error("{0}")]
    InvalidCommonTokenDenom(#[from] InvalidCommonTokenDenom),

    #[error("{0}")]
    InterestError(#[from] InterestError),

    #[error("Cannot deposit {attempted_deposit} tokens - market cap is {cap} and there are already {ltoken_supply} tokens present")]
    DepositOverCap {
        attempted_deposit: Uint128,
        ltoken_supply: Uint128,
        cap: Uint128,
    },

    #[error("Cw20 tokens are not supported yet")]
    Cw20TokensNotSupported,
}
