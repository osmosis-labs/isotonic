use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot process zero tokens")]
    InvalidZeroAmount,

    #[error("Cannot transfer tokens - controller refuses to transfer more than {max_transferable} tokens")]
    CannotTransfer { max_transferable: Uint128 },
}
