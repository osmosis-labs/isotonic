use cosmwasm_std::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use utils::time::Expiration;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PriceRecord {
    pub rate: Decimal,
    pub expires: Expiration,
}
