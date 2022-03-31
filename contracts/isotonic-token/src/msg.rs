use cosmwasm_std::{Binary, Coin, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::display_amount::DisplayAmount;
use utils::token::Token;

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
    /// Token which will be distributed via this contract by cw2222 interface
    pub distributed_token: Token,
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
    /// TransferFrom allows to order transfer of tokens from source to destination.
    /// Proper authentication is in place - can be called only be controller.
    /// Requires check for transfer possibility by `ControllerQuery::CanTransfer` call to
    /// controller.
    TransferFrom {
        sender: String,
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
    BurnFrom {
        owner: String,
        amount: DisplayAmount,
    },
    /// Can only be called by the controller.
    /// multiplier *= ratio
    Rebase { ratio: Decimal },
    /// Distributed tokens using cw2222 mechanism. Tokens send with this message as distributed
    /// alongside with all tokens send until now which are not yet distributed.
    Distribute {
        /// Just for informational purposes - would overwrite message sender in generated event.
        sender: Option<String>,
    },
    /// Withdraw tokens distributed before
    WithdrawFunds {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControllerQuery {
    TransferableAmount {
        /// Isotonic contract address that calls "CanTransfer"
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
    /// Funds distributed by this contract. Returns `FundsResponse`.
    DistributedFunds {},
    /// FUnds send to this contact but not yet distributed. Returns `FundsResponse`.
    UndistributedFunds {},
    /// Queries for funds distributed but not yet withdrawn by owner
    WithdrawableFunds { owner: String },
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

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct FundsResponse {
    pub funds: Coin,
}
