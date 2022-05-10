use cosmwasm_std::{Decimal, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use utils::interest::Interest;
use utils::{coin::Coin, token::Token};

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
    /// Market token
    pub market_token: Token,
    /// An optional cap on total number of tokens deposited into the market
    pub market_cap: Option<Uint128>,
    /// Interest rate curve
    pub interest_rate: Interest,
    /// Token which would be distributed via created isotonic contracts
    pub distributed_token: Token,
    /// Define interest's charged period (in seconds)
    pub interest_charge_period: u64,
    /// Common Token denom that comes from Credit Agency (same for all markets)
    pub common_token: Token,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    pub collateral_ratio: Decimal,
    /// Address of contract to query for price
    pub price_oracle: String,
    /// Defines the portion of borrower interest that is converted into reserves (0 <= x <= 1)
    pub reserve_factor: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// X market_token must be sent along with this message. If it matches, X l_token is minted of the sender address.
    /// The underlying market_token is stored in this Market contract
    Deposit {},
    /// Similar to `Deposit`, but allows the sender to deposit to someone else's account.
    DepositTo {
        account: String,
    },
    /// This requests to withdraw the amount of L Tokens. More specifically,
    /// the contract will burn amount L Tokens and return that to the lender in base asset.
    Withdraw {
        amount: Uint128,
    },
    /// If sent tokens' denom matches market_token, burns tokens from sender's address
    Repay {},
    /// Dispatches two messages, one to mint amount of BToken for this sender,
    /// and the other to send amount base asset to the sender
    Borrow {
        amount: Uint128,
    },
    /// Helper to allow repay of debt on given account. Transfers and burns btokens.
    /// Sender must be a Credit Agency
    RepayTo {
        account: String,
        amount: Uint128,
    },
    /// Helper to allow transfering Ltokens from account source to account destination.
    /// Sender must be a Credit Agency
    TransferFrom {
        source: String,
        destination: String,
        amount: Uint128,
        liquidation_price: Decimal,
    },
    AdjustCommonToken {
        new_token: Token,
    },
    /// Withdraw some base asset, by burning L Tokens and swapping it for `buy` amount.
    /// The bought tokens are transferred to the sender.
    /// Only callable by the credit agency. Skips the credit line check.
    SwapWithdrawFrom {
        account: String,
        sell_limit: Uint128,
        buy: Coin,
    },
    /// Deposits the market currency sent with this message and distributes the L Tokens to all existing lenders.
    /// Only callable by the credit agency.
    DistributeAsLTokens {},
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
        /// Isotonic contract address that calls "CanTransfer"
        token: String,
        /// Address that wishes to transfer
        account: String,
    },
    /// Returns the amount that the given account can withdraw
    Withdrawable { account: String },
    /// Returns the amount that the given account can borrow
    Borrowable { account: String },
    /// Returns current utilisation and interest rates
    Interest {},
    /// Returns PriceRate, structure representing sell/buy ratio for local(market)/common denoms
    PriceMarketLocalPerCommon {},
    /// Returns CreditLineResponse
    CreditLine { account: String },
    /// Returns ReserveResponse
    Reserve {},
    /// APY Query
    Apy {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    AdjustCollateralRatio { new_ratio: Decimal },
    AdjustReserveFactor { new_factor: Decimal },
    AdjustPriceOracle { new_oracle: String },
    AdjustMarketCap { new_cap: Option<Uint128> },
    AdjustInterestRates { new_interest_rates: Interest },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {
    pub isotonic_token_id: Option<u64>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ReserveResponse {
    pub reserve: Uint128,
}

// TODO: should this be defined elsewhere?
// This is here so we can call CA entrypoints without adding credit agency as a dependency.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CreditAgencyExecuteMsg {
    /// Ensures a given account has entered a market. Meant to be called by a specific
    /// market contract - so the sender of the msg would be the market
    EnterMarket { account: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ApyResponse {
    /// How much % interest will a borrower have to pay
    pub borrower: Decimal,
    /// How much % interest will a lender earn
    pub lender: Decimal,
}
