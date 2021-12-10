use std::convert::TryInto;

use cosmwasm_std::{Binary, Decimal, Fraction, Uint128, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token precision for displaying
    pub decimals: u8,
    /// Controller is contract allowed to ming, burn, rebase, and must be checked with to
    /// enable transfer
    pub controller: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer is a base message to move tokens to another account without triggering actions.
    /// Requires check for transfer possibility by `ControllerQuery::CanTransfer` call to
    /// controller.
    Transfer {
        recipient: String,
        amount: DisplayAmount,
    },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    /// Requires check for transfer possibility by `ControllerQuery::CanTransfer` call to
    /// controller.
    Send {
        contract: String,
        amount: DisplayAmount,
        msg: Binary,
    },
    /// Reserved for controller
    Mint {
        recipient: String,
        amount: DisplayAmount,
    },
    /// Reserved for controller
    Burn { amount: DisplayAmount },
    /// Can only be called by the controller.
    /// multiplier *= ratio
    Rebase { ratio: Decimal },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControllerQuery {
    TransferableAmount {
        /// Lendex contract address that calls "CanTransfer"
        token: String,
        /// Address that wishes to transfer
        account: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TransferableAmountResp {
    pub transferable: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    /// Return type: `BalanceResponse`.
    Balance { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: `TokenInfoResponse`.
    TokenInfo {},
    /// Returns the global multiplier factor.
    Multiplier {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BalanceResponse {
    pub balance: DisplayAmount,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: DisplayAmount,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MultiplierResponse {
    pub multiplier: Decimal,
}

/// Represents the amount of tokens displayed in the API - the contract ensures
/// a 1:1 mapping to the base tokens. The displayed amount is different from the amount
/// actually stored and manipulated by the contract.
///
/// display_amount = stored_amount * multiplier
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Copy)]
#[serde(transparent)]
pub struct DisplayAmount(Uint128);

impl DisplayAmount {
    pub fn from_stored_amount(multiplier: Decimal, amount: impl Into<Uint128>) -> Self {
        Self(amount.into() * multiplier)
    }

    pub const fn zero() -> Self {
        Self(Uint128::zero())
    }

    /// A helper to construct this from a raw value for tests.
    pub fn raw(amount: impl Into<Uint128>) -> Self {
        Self(amount.into())
    }

    pub fn to_stored_amount(self, multiplier: Decimal) -> Uint128 {
        // self.0 / multiplier
        let result256 =
            self.0.full_mul(multiplier.denominator()) / Uint256::from(multiplier.numerator());

        result256.try_into().unwrap()
    }

    /// A helper to get the raw inner value for tests.
    pub fn unpack_raw(self) -> Uint128 {
        self.0
    }
}
