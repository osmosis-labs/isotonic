use cosmwasm_std::{Binary, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use cw20::Cw20ReceiveMsg;

use crate::display_amount::DisplayAmount;

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
