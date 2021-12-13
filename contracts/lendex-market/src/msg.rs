use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;

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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// X base_asset must be sent along with this message. If it matches, X l_token is minted of the sender address.
    /// The underlying base_asset is stored in this Market contract
    Deposit {},
    /// This requests to burn amount L Tokens and receive that in base_asset.
    /// In the future we will do a check if this is allowed, for now, you can always allow, assuming enough L Token balance
    /// Dispatches two messages, one to burn amount Token from this, and the other to send amount base asset to the sender.
    Withdraw { amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns current configuration
    Configuration {},
    TransferableAmount {
        /// Lendex contract address that calls "CanTransfer"
        token: String,
        /// Address that wishes to transfer
        account: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TransferableAmountResponse {
    pub transferable: Uint128,
}
