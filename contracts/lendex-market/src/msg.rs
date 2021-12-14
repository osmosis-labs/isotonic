use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Name used to create the sub-tokens `Lent ${name}` and `Borrowed ${name}`
    pub name: String,
    /// Symbol used to create the sub-tokens `L${symbol}` and `B${symbol}`
    pub symbol: String,
    /// Decimals for sub-tokens `L` and `B`
    pub decimals: u8,
    /// CodeId used to create sub-tokens `L` and `B`
    pub token_id: u64,
    /// Native denom for the base asset
    pub base_asset: String,
    /// Interest rate curve
    pub interest_rate: Interest,
    /// Token which would be distributed via created lendex contracts
    pub distributed_token: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Interest {
    Linear {
        /// Base percentage, charged at 0% utilisation
        base: Decimal,
        /// Utilisation multiplier
        slope: Decimal,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// X base_asset must be sent along with this message. If it matches, X l_token is minted of the sender address.
    /// The underlying base_asset is stored in this Market contract
    Deposit {},
    /// This requests to withdraw the amount of L Tokens. More specifically,
    /// the contract will burn amount L Tokens and return that to the lender in base asset.
    Withdraw { amount: Uint128 },
    /// If sent tokens' denom matches base_asset, burns tokens from sender's address
    Repay {},
    /// Dispatches two messages, one to mint amount of BToken for this sender,
    /// and the other to send amount base asset to the sender
    Borrow { amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns current configuration
    Configuration {},
    /// Returns TransferableAmountResponse
    TransferableAmount {
        /// Lendex contract address that calls "CanTransfer"
        token: String,
        /// Address that wishes to transfer
        account: String,
    },
    /// Returns current utilisation and interest rates
    Interest {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InterestResponse {
    pub interest: Decimal,
    pub utilisation: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TransferableAmountResponse {
    pub transferable: Uint128,
}
