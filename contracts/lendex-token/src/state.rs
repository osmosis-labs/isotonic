use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

use crate::i128::Int128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Distribution {
    /// Token distributed by this contract.
    pub denom: String,
    /// How much points is single division of lendex worth at this point.
    pub points_per_token: Uint128,
    /// Points which were not fully distributed on previous distribution, and should be
    /// redistributed.
    pub points_leftover: Uint128,
    /// Total funds distributed by this contract.
    pub distributed_total: Uint128,
    /// Total funds not yet withdrawn.
    pub withdrawable_total: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct WithdrawAdjustment {
    /// How much points should be added/removed from calculated funds while withdrawal.
    pub points_correction: Int128,
    /// How much funds was already withdrawn.
    pub withdrawn_funds: Uint128,
}

/// How much points is the worth of single token in token distribution.
/// The scaling is performed to have better precision of fixed point division.
/// This value is not actually the scaling itself, but how much bits value should be shifted
/// (for way more efficient division).
///
/// `32, to have those 32 bits, but it reduces how much tokens may be handled by this contract
/// (it is now 96-bit integer instead of 128). In original ERC2222 it is handled by 256-bit
/// calculations, but I256 is missing and it is required for this.
pub const POINTS_SHIFT: u8 = 32;

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const TOTAL_SUPPLY: Item<Uint128> = Item::new("total_supply");
pub const CONTROLLER: Item<Addr> = Item::new("controller");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
pub const MULTIPLIER: Item<Decimal> = Item::new("multiplier");
pub const DISTRIBUTION: Item<Distribution> = Item::new("distribution");
pub const WITHDRAW_ADJUSTMENT: Map<&Addr, WithdrawAdjustment> = Map::new("withdraw_adjustment");
