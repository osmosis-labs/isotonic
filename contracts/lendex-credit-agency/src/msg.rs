use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utils::interest::Interest;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: String,
    /// The CodeId of the lendex-market contract
    pub lendex_market_id: u64,
    /// The CodeId of the lendex-token contract
    pub lendex_token_id: u64,
    /// Token denom which would be distributed as reward token to lendex token holders.
    /// This is `distributed_token` in the market contract.
    pub reward_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateMarket(MarketConfig),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketConfig {
    /// Name used to create the sub-tokens `Lent ${name}` and `Borrowed ${name}`
    pub name: String,
    /// Symbol used to create the sub-tokens `L${symbol}` and `B${symbol}`
    pub symbol: String,
    /// Decimals for sub-tokens `L` and `B`
    pub decimals: u8,
    /// Native denom for the base asset
    pub base_asset: String,
    /// Interest rate curve
    pub interest_rate: Interest,
    /// Define interest's charged period (in seconds)
    pub interest_charge_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns current configuration
    Configuration {},
    /// Queries a market address by base asset
    Market { base_asset: String },
    /// List all base assets and the addresses of markets handling them.
    /// Pagination by base asset
    ListMarkets {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MarketResponse {
    pub base_asset: String,
    pub market: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ListMarketsResponse {
    pub markets: Vec<MarketResponse>,
}
