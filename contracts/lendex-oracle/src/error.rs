use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("There is no info about the prices for this trading pair")]
    NoInfo {},

    #[error("The prices for this trading pair are outdated")]
    OutdatedOracle {},
}
