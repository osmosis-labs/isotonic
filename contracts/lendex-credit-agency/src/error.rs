use cosmwasm_std::StdError;
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
}
