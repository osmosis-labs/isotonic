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
