use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::iter::Sum;

use cosmwasm_std::{coin, Coin, Decimal, Timestamp, Uint128};

use utils::interest::Interest;

use crate::ContractError;

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

/// The Credit Line response with the common token denom included. Used in the API.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CreditLineResponse {
    /// Total value of L-Tokens in common_token
    pub collateral: Coin,
    /// collateral * collateral_ratio
    pub credit_line: Coin,
    /// Total value of B-Tokens in common_token
    pub debt: Coin,
}

impl CreditLineResponse {
    pub fn validate(&self, expected_denom: &str) -> Result<CreditLineValues, ContractError> {
        for actual in [
            &self.collateral.denom,
            &self.credit_line.denom,
            &self.debt.denom,
        ] {
            if actual != expected_denom {
                return Err(ContractError::InvalidCommonTokenDenom {
                    expected: expected_denom.to_string(),
                    actual: actual.to_string(),
                });
            }
        }

        Ok(CreditLineValues {
            collateral: self.collateral.amount,
            credit_line: self.credit_line.amount,
            debt: self.debt.amount,
        })
    }
}

/// The Credit Line with just the values and no denom included, used for internal calculations.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditLineValues {
    /// Total value of L-Tokens in common_token
    pub collateral: Uint128,
    /// collateral * collateral_ratio
    pub credit_line: Uint128,
    /// Total value of B-Tokens in common_token
    pub debt: Uint128,
}

impl CreditLineValues {
    pub fn zero() -> Self {
        CreditLineValues {
            collateral: Uint128::zero(),
            credit_line: Uint128::zero(),
            debt: Uint128::zero(),
        }
    }

    pub fn new(
        collateral: impl Into<Uint128>,
        credit_line: impl Into<Uint128>,
        debt: impl Into<Uint128>,
    ) -> Self {
        CreditLineValues {
            collateral: collateral.into(),
            credit_line: credit_line.into(),
            debt: debt.into(),
        }
    }

    pub fn make_response(self, denom: impl Into<String>) -> CreditLineResponse {
        let denom = denom.into();

        CreditLineResponse {
            collateral: coin(self.collateral.u128(), denom.clone()),
            credit_line: coin(self.credit_line.u128(), denom.clone()),
            debt: coin(self.debt.u128(), denom),
        }
    }
}

impl<'a> Sum<&'a Self> for CreditLineValues {
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
            CreditLineValues {
                collateral: Uint128::new(500),
                credit_line: Uint128::new(300),
                debt: Uint128::new(200),
            },
            CreditLineValues {
                collateral: Uint128::new(1800),
                credit_line: Uint128::new(200),
                debt: Uint128::new(50),
            },
            CreditLineValues::zero(),
        ];

        let sum: CreditLineValues = responses.iter().sum();
        assert_eq!(
            sum,
            CreditLineValues {
                collateral: Uint128::new(2300),
                credit_line: Uint128::new(500),
                debt: Uint128::new(250),
            },
        );
    }
}
