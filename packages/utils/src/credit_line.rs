use std::iter::Sum;

use cosmwasm_std::{coin, Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    pub fn validate(
        &self,
        expected_denom: &str,
    ) -> Result<CreditLineValues, InvalidCommonTokenDenom> {
        for actual in [
            &self.collateral.denom,
            &self.credit_line.denom,
            &self.debt.denom,
        ] {
            if actual != expected_denom {
                return Err(InvalidCommonTokenDenom {
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

/// Used for when CreditLineResponse validation fails
#[derive(Error, Debug, PartialEq)]
#[error("Received invalid common token from another contract, expected: {expected}, got: {actual}")]
pub struct InvalidCommonTokenDenom {
    pub expected: String,
    pub actual: String,
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
