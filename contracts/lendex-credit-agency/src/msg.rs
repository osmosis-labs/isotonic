use cosmwasm_std::{Addr, Decimal, Uint128};
use utils::interest::Interest;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    /// Common Token denom (same for all markets)
    pub common_token: String,
    /// Price for collateral in exchange for paying debt during liquidation
    pub liquidation_price: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateMarket(MarketConfig),
    /// Tries to perform liquidation on passed account using collateral's denom
    Liquidate {
        account: String,
        collateral_denom: String,
    },
    /// Ensures a given account has entered a market. Meant to be called by a specific
    /// market contract - so the sender of the msg would be the market
    EnterMarket {
        account: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketConfig {
    /// Name used to create the sub-tokens `Lent ${name}` and `Borrowed ${name}`
    pub name: String,
    /// Symbol used to create the sub-tokens `L${symbol}` and `B${symbol}`
    pub symbol: String,
    /// Decimals for sub-tokens `L` and `B`
    pub decimals: u8,
    /// Native denom for the market token
    pub market_token: String,
    /// An optional cap on total number of tokens deposited into the market
    pub market_cap: Option<Uint128>,
    /// Interest rate curve
    pub interest_rate: Interest,
    /// Define interest's charged period (in seconds)
    pub interest_charge_period: u64,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    pub collateral_ratio: Decimal,
    /// Address of contract to query for price
    pub price_oracle: String,
    /// Defines the portion of borrower interest that is converted into reserves (0 <= x <= 1)
    pub reserve_factor: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns current configuration
    Configuration {},
    /// Queries a market address by market token
    Market { market_token: String },
    /// List all base assets and the addresses of markets handling them.
    /// Pagination by base asset
    ListMarkets {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Queries all markets for credit lines for particular account
    /// and returns sum of all of them.
    /// Returns CreditLineResponse
    TotalCreditLine { account: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    AdjustMarketId { new_market_id: u64 },
    AdjustTokenId { new_token_id: u64 },
    AdjustCommonToken { new_common_token: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MarketResponse {
    pub market_token: String,
    pub market: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ListMarketsResponse {
    pub markets: Vec<MarketResponse>,
}
