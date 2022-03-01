use cosmwasm_std::{Coin, Decimal, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use utils::interest::Interest;

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
    /// Native denom for the market tokene
    pub market_token: String,
    /// An optional cap on total number of tokens deposited into the market
    pub market_cap: Option<Uint128>,
    /// Interest rate curve
    pub interest_rate: Interest,
    /// Token which would be distributed via created lendex contracts
    pub distributed_token: String,
    /// Define interest's charged period (in seconds)
    pub interest_charge_period: u64,
    /// Common Token denom that comes from Credit Agency (same for all markets)
    pub common_token: String,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    pub collateral_ratio: Decimal,
    /// Address of contract to query for price
    pub price_oracle: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// X market_token must be sent along with this message. If it matches, X l_token is minted of the sender address.
    /// The underlying market_token is stored in this Market contract
    Deposit {},
    /// This requests to withdraw the amount of L Tokens. More specifically,
    /// the contract will burn amount L Tokens and return that to the lender in base asset.
    Withdraw { amount: Uint128 },
    /// If sent tokens' denom matches market_token, burns tokens from sender's address
    Repay {},
    /// Dispatches two messages, one to mint amount of BToken for this sender,
    /// and the other to send amount base asset to the sender
    Borrow { amount: Uint128 },
    /// Helper to allow repay of debt on given account. Transfers and burns btokens.
    /// Sender must be a Credit Agency
    RepayTo { account: String, amount: Uint128 },
    /// Helper to allow transfering Ltokens from account source to account destination.
    /// Sender must be a Credit Agency
    TransferFrom {
        source: String,
        destination: String,
        amount: Coin,
        liquidation_price: Decimal,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns current configuration
    Configuration {},
    /// Returns TokensBalanceResponse
    TokensBalance { account: String },
    /// Returns TransferableAmountResponse
    TransferableAmount {
        /// Lendex contract address that calls "CanTransfer"
        token: String,
        /// Address that wishes to transfer
        account: String,
    },
    /// Returns current utilisation and interest rates
    Interest {},
    /// Returns PriceRate, structure representing sell/buy ratio for local(market)/common denoms
    PriceMarketLocalPerCommon {},
    /// Returns CreditLineResponse
    CreditLine { account: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryTotalCreditLine {
    TotalCreditLine { account: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InterestResponse {
    pub interest: Decimal,
    pub utilisation: Decimal,
    pub charge_period: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokensBalanceResponse {
    pub ltokens: Uint128,
    pub btokens: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TransferableAmountResponse {
    pub transferable: Uint128,
}
