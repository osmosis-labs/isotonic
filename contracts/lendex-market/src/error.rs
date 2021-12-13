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

    #[error("No funds sent")]
    NoFundsSent {},

    #[error("Sent too many denoms, must deposit only '{0}' in the lending pool")]
    ExtraDenoms(String),

    #[error("Sent unsupported token, must deposit '{0}' in the lending pool")]
    InvalidDenom(String),

    #[error("Cannot borrow amount {amount} for {account}")]
    CannowBorrow { amount: Uint128, account: String },

    #[error("Address {account} cannot withdraw {amount}")]
    CannotWithdraw { account: String, amount: Uint128 },
}
