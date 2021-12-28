use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::iter::Sum;

use cosmwasm_std::{Decimal, Timestamp, Uint128};
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
    /// Returns CreditLineResponse
    CreditLine { account: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InterestResponse {
    pub interest: Decimal,
    pub utilisation: Decimal,
    pub charge_period: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TransferableAmountResponse {
    pub transferable: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CreditLineResponse {
    /// Total value of L-Tokens in common_token
    pub collateral: Uint128,
    /// collateral * collateral_ratio
    pub credit_line: Uint128,
    /// Total value of B-Tokens in common_token
    pub debt: Uint128,
}

impl CreditLineResponse {
    pub fn zero() -> Self {
        CreditLineResponse {
            collateral: Uint128::zero(),
            credit_line: Uint128::zero(),
            debt: Uint128::zero(),
        }
    }
}

impl<'a> Sum<&'a Self> for CreditLineResponse {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a Self>,
    {
        iter.fold(Self::zero(), |a, b| Self {
            collateral: a.collateral + b.collateral,
            credit_line: a.credit_line + b.credit_line,
            debt: a.debt + b.debt,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_credit_line_response() {
        let responses = vec![
            CreditLineResponse {
                collateral: Uint128::new(500),
                credit_line: Uint128::new(300),
                debt: Uint128::new(200),
            },
            CreditLineResponse {
                collateral: Uint128::new(1800),
                credit_line: Uint128::new(200),
                debt: Uint128::new(50),
            },
            CreditLineResponse::zero(),
        ];

        let sum: CreditLineResponse = responses.iter().sum();
        assert_eq!(
            sum,
            CreditLineResponse {
                collateral: Uint128::new(2300),
                credit_line: Uint128::new(500),
                debt: Uint128::new(250),
            },
        );
    }
}
