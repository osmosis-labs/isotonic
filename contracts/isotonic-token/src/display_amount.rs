use cosmwasm_std::{Decimal, Fraction, Uint128, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the amount of tokens displayed in the API - the contract ensures
/// a 1:1 mapping to the base tokens. The displayed amount is different from the amount
/// actually stored and manipulated by the contract.
///
/// display_amount = stored_amount * multiplier
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Copy)]
#[serde(transparent)]
pub struct DisplayAmount(Uint128);

impl DisplayAmount {
    pub fn from_stored_amount(multiplier: Decimal, amount: impl Into<Uint128>) -> Self {
        Self(amount.into() * multiplier)
    }

    pub const fn zero() -> Self {
        Self(Uint128::zero())
    }

    /// A helper to construct this from a raw value for tests.
    pub fn raw(amount: impl Into<Uint128>) -> Self {
        Self(amount.into())
    }

    pub fn to_stored_amount(self, multiplier: Decimal) -> Uint128 {
        // self.0 / multiplier
        let result256 =
            self.0.full_mul(multiplier.denominator()) / Uint256::from(multiplier.numerator());

        result256.try_into().unwrap()
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn display_amount(&self) -> Uint128 {
        self.0
    }
}
