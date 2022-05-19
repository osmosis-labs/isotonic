use cosmwasm_std::{Addr, Decimal, Uint128};
use isotonic_market::msg::MigrateMsg as MarketMigrateMsg;

use utils::{coin::Coin, interest::Interest, token::Token};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The address that controls the credit agency and can set up markets
    pub gov_contract: String,
    /// The CodeId of the isotonic-market contract
    pub isotonic_market_id: u64,
    /// The CodeId of the isotonic-token contract
    pub isotonic_token_id: u64,
    /// Token denom which would be distributed as reward token to isotonic token holders.
    /// This is `distributed_token` in the market contract.
    pub reward_token: Token,
    /// Common Token denom (same for all markets)
    pub common_token: Token,
    /// The liquidation fee to be paid out to all lenders in the debt market
    pub liquidation_fee: Decimal,
    /// The liquidation triggering fee to be paid out to the person who "clicked the button"
    pub liquidation_initiation_fee: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateMarket(MarketConfig),
    /// Tries to perform liquidation on passed account using a specific kind of collateral
    Liquidate {
        account: String,
        collateral_denom: Token,
        amount_to_repay: Coin,
    },
    /// Ensures a given account has entered a market. Meant to be called by a specific
    /// market contract - so the sender of the msg would be the market
    EnterMarket {
        account: String,
    },
    /// Exits market if:
    /// * Sender have no BTokens in the market
    /// * Sender have no LTokens in the market, or collateral provided by owned LTokens
    ///   is not affecting liquidity of sender
    ExitMarket {
        market: String,
    },
    /// Repay a loan by using some indicated collateral.
    /// The collateral is traded on the Osmosis AMM.
    RepayWithCollateral {
        /// The maximum amount of collateral to use
        max_collateral: Coin,
        /// How much of the loan is trying to be repaid
        amount_to_repay: Coin,
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
    pub market_token: Token,
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
    Market { market_token: Token },
    /// List all base assets and the addresses of markets handling them.
    /// Pagination by base asset
    ListMarkets {
        start_after: Option<Token>,
        limit: Option<u32>,
    },
    /// Queries all markets for credit lines for particular account
    /// and returns sum of all of them.
    /// Returns CreditLineResponse
    TotalCreditLine { account: String },
    /// Lists all markets which address entered. Pagination by market contract address. Mostly for
    /// verification purposes, but may be useful to verify if there are some obsolete markets to
    /// leave.
    /// Returns `ListEnteredMarketsResponse`
    ListEnteredMarkets {
        account: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Checks if account is a member of particular market. Useful to ensure if the account is
    /// included in market before leaving it (to not waste tokens on obsolete call).
    /// Returns `IsOnMarketResponse`
    IsOnMarket { account: String, market: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    AdjustMarketId {
        new_market_id: u64,
    },
    AdjustTokenId {
        new_token_id: u64,
    },
    /// Sets common_token parameter in configuration and sends AdjustCommonToken
    /// message to all affiliated markets
    AdjustCommonToken {
        new_common_token: Token,
    },
    MigrateMarket {
        contract: String,
        migrate_msg: MarketMigrateMsg,
    },
    AdjustLiquidation {
        liquidation_fee: Option<Decimal>,
        liquidation_initiation_fee: Option<Decimal>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MarketResponse {
    pub market_token: Token,
    pub market: Addr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ListMarketsResponse {
    pub markets: Vec<MarketResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ListEnteredMarketsResponse {
    pub markets: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct IsOnMarketResponse {
    pub participating: bool,
}
