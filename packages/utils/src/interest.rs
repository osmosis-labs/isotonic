use cosmwasm_std::{Decimal, Fraction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Interest {
    Linear {
        /// Base percentage, charged at 0% utilisation
        base: Decimal,
        /// Utilisation multiplier
        slope: Decimal,
    },
    /// A piecewise linear composed of two pieces. The breakpoint is meant
    /// to be the "optimal" utilisation after which there's a much steeper
    /// slope.
    /// https://docs.aave.com/risk/liquidity-risk/borrow-interest-rate
    PiecewiseLinear {
        /// Base percentage, charged at 0% utilisation.
        /// *R0* in the Aave docs.
        base: Decimal,
        /// Rate charged on top of `base` at the breakpoint.
        /// *Rslope1* in the Aave docs.
        slope1: Decimal,
        /// Determines the max interest rate charged. At 100% utilisation,
        /// `base + slope1 + slope2` is the max interest rate.
        /// *Rslope2* in the Aave docs.
        slope2: Decimal,
        /// The optimal utilisation and the breakpoint between the two segments.
        /// *Uoptimal* in the Aave docs.
        optimal_utilisation: Decimal,
    },
}

impl Interest {
    pub fn calculate_interest_rate(&self, utilisation: Decimal) -> Decimal {
        match self {
            Interest::Linear { base, slope } => *base + *slope * utilisation,
            Interest::PiecewiseLinear {
                base,
                slope1,
                slope2,
                optimal_utilisation,
            } => {
                if utilisation < *optimal_utilisation || *optimal_utilisation == Decimal::one() {
                    // unwrapping should be okay here - if `optimal_utilisation == 0`,
                    // this branch will never be reached
                    *base + *slope1 * (utilisation * optimal_utilisation.inv().unwrap())
                } else {
                    // unwrapping should be okay here - if `optimal_utilisation == 1`,
                    // this branch will never be reached
                    *base
                        + *slope1
                        + *slope2
                            * ((utilisation - *optimal_utilisation)
                                * (Decimal::one() - *optimal_utilisation).inv().unwrap())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_interest_rate() {
        let interest = Interest::Linear {
            base: Decimal::percent(10),
            slope: Decimal::percent(90),
        };

        assert_eq!(
            interest.calculate_interest_rate(Decimal::zero()),
            Decimal::percent(10)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::percent(10)),
            Decimal::percent(19)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::one()),
            Decimal::one()
        );
    }

    #[test]
    fn piecewise_linear_interest_rate() {
        let interest = Interest::PiecewiseLinear {
            base: Decimal::percent(10),
            slope1: Decimal::percent(10),
            slope2: Decimal::percent(100),
            optimal_utilisation: Decimal::percent(50),
        };

        assert_eq!(
            interest.calculate_interest_rate(Decimal::zero()),
            Decimal::percent(10)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::percent(10)),
            Decimal::percent(12)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::percent(40)),
            Decimal::percent(18)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::percent(50)),
            Decimal::percent(20)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::percent(60)),
            Decimal::percent(40)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::percent(90)),
            Decimal::percent(100)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::one()),
            Decimal::percent(120)
        );
    }

    #[test]
    fn piecewise_linear_interest_rate_zero_optimal_utilisation() {
        let interest = Interest::PiecewiseLinear {
            base: Decimal::percent(10),
            slope1: Decimal::percent(10),
            slope2: Decimal::percent(100),
            optimal_utilisation: Decimal::zero(),
        };

        assert_eq!(
            interest.calculate_interest_rate(Decimal::zero()),
            Decimal::percent(20)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::percent(50)),
            Decimal::percent(70)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::one()),
            Decimal::percent(120)
        );
    }

    #[test]
    fn piecewise_linear_interest_rate_one_optimal_utilisation() {
        let interest = Interest::PiecewiseLinear {
            base: Decimal::percent(10),
            slope1: Decimal::percent(10),
            slope2: Decimal::percent(100),
            optimal_utilisation: Decimal::one(),
        };

        assert_eq!(
            interest.calculate_interest_rate(Decimal::zero()),
            Decimal::percent(10)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::percent(50)),
            Decimal::percent(15)
        );
        assert_eq!(
            interest.calculate_interest_rate(Decimal::one()),
            Decimal::percent(20)
        );
    }
}
