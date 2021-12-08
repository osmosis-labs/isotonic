use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    Transfer { recipient: String, amount: Uint128 },
    /// Reserved for controller
    Burn { amount: Uint128 },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    /// Requires check for transfer possibility by `ControllerQuery::CanTransfer` call to
    /// controller.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
    /// Reserved for controller
    Mint { recipient: String, amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControllerQuery {
    CanTransfer {
        /// Lendex contract address that calls "CanTransfer"
        token: String,
        /// Address that wishes to transfer
        account: String,
        /// The amount we wish to transfer from theri account
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CanTransferResp {
    /// No tokens can be transferred
    None,
    /// Whole requested amount can be transferred
    Whole,
    /// Can transfer tokens, but only limited amount
    Partial(Uint128),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    /// Return type: BalanceResponse.
    Balance { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    TokenInfo {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}
