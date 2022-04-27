use cosmwasm_std::{Uint128, coin};
use utils::coin::{Coin, coin_native};

use super::suite::{SuiteBuilder, COMMON};
use crate::error::ContractError;

#[test]
fn sender_not_ca() {
    let mut suite = SuiteBuilder::new()
        .build();

    let err = suite.swap_withdraw_from("any sender", "account", Uint128::zero(), coin_native(100, "denom")).unwrap_err();
    assert_eq!(ContractError::RequiresCreditAgency {}, err.downcast().unwrap());
}

#[test]
fn two_denoms() {
    let debtor = "debtor";
    let market_token = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    }
