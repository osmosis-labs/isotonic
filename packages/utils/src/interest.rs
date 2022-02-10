use cosmwasm_std::Decimal;
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
}

impl Interest {
    pub fn calculate_interest_rate(&self, utilisation: Decimal) -> Decimal {
        match self {
            Interest::Linear { base, slope } => *base + *slope * utilisation,
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
}
