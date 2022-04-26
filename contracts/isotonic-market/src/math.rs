use cosmwasm_std::{ConversionOverflowError, Decimal, Fraction, OverflowError, Uint128};

// Once there's Decimal::checked_pow in cosmwasm_std, we can get rid of this trait
// https://github.com/CosmWasm/cosmwasm/pull/1239
// We're waiting for a new release after v1.0.0-beta6

pub trait DecimalExt {
    fn checked_mul(self, rhs: Self) -> Result<Decimal, OverflowError>;
    fn checked_pow(self, exp: u32) -> Result<Decimal, OverflowError>;
}

impl DecimalExt for Decimal {
    fn checked_mul(self, other: Self) -> Result<Self, OverflowError> {
        fn inner(lhs: Decimal, rhs: Decimal) -> Result<Decimal, ConversionOverflowError> {
            let numerator: Uint128 = lhs.numerator().full_mul(rhs.numerator()).try_into()?;
            let denominator: Uint128 = lhs.denominator().full_mul(rhs.denominator()).try_into()?;

            Ok(Decimal::from_ratio(numerator, denominator))
        }

        inner(self, other).map_err(|_| OverflowError {
            operation: cosmwasm_std::OverflowOperation::Mul,
            operand1: self.to_string(),
            operand2: other.to_string(),
        })
    }

    fn checked_pow(self, mut n: u32) -> Result<Self, OverflowError> {
        let mut x = self;

        if n == 0 {
            return Ok(Decimal::one());
        }

        let mut y = Decimal::one();

        while n > 1 {
            if n % 2 == 0 {
                x = x.checked_mul(x)?;
                n /= 2;
            } else {
                y = x.checked_mul(y)?;
                x = x.checked_mul(x)?;
                n = (n - 1) / 2;
            }
        }

        Ok(x * y)
    }
}
