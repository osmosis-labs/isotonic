use cosmwasm_std::Decimal;

// Once there's Decimal::pow in cosmwasm_std, we can get rid of this trait

pub trait DecimalExt {
    fn pow(self, exp: usize) -> Decimal;
}

impl DecimalExt for Decimal {
    fn pow(self, exp: usize) -> Self {
        // TODO: naive algorithm, should improve this
        std::iter::repeat(self)
            .take(exp)
            .fold(Decimal::one(), std::ops::Mul::mul)
    }
}
