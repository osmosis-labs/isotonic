use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("This contract cannot be executed")]
    NoExecute {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("There is no info about the prices for this trading pair: {denom1}, {denom2}")]
    NoInfo { denom1: String, denom2: String },
}
