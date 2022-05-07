use cosmwasm_std::{coin, Uint128};
use utils::coin::coin_native;

use super::suite::{SuiteBuilder, COMMON};
use crate::error::ContractError;

#[test]
fn sender_not_credit_agency() {
    let mut suite = SuiteBuilder::new().build();

    let err = suite
        .swap_withdraw_from(
            "any sender",
            "account",
            Uint128::zero(),
            coin_native(100, "denom"),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::RequiresCreditAgency {},
        err.downcast().unwrap()
    );
}

#[test]
fn two_denoms() {
    let user = "user";
    let atom = "ATOM";
    let ust = "UST";
    let mut suite = SuiteBuilder::new()
        .with_market_token(atom)
        .with_funds(user, &[coin(5_000_000, atom)])
        .with_pool(
            1,
            (coin(100_000_000_000, COMMON), coin(100_000_000_000, atom)),
        )
        .with_pool(
            2,
            (coin(100_000_000_000, ust), coin(100_000_000_000, COMMON)),
        )
        .build();

    suite.deposit(user, &[coin(5_000_000, atom)]).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 5_000_000);

    let ca = suite.credit_agency();
    // Buy 4.5M UST, using maximally 5M ATOM tokens for that
    suite
        .swap_withdraw_from(
            ca,
            user,
            Uint128::new(5_000_000),
            coin_native(4_500_000, ust),
        )
        .unwrap();

    // Excluding swap fees, amount left on contract should be less or equal to 0.5M tokens
    assert!(
        matches!(suite.query_contract_asset_balance().unwrap(), x if x > 470_000 && x <= 500_000)
    );
}

#[test]
fn sell_limit_lesser_then_required() {
    let user = "user";
    let atom = "ATOM";
    let ust = "UST";
    let mut suite = SuiteBuilder::new()
        .with_market_token(atom)
        .with_funds(user, &[coin(5_000_000, atom)])
        .with_pool(
            1,
            (coin(100_000_000_000, COMMON), coin(100_000_000_000, atom)),
        )
        .with_pool(
            2,
            (coin(100_000_000_000, ust), coin(100_000_000_000, COMMON)),
        )
        .build();

    suite.deposit(user, &[coin(5_000_000, atom)]).unwrap();
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 5_000_000);

    let ca = suite.credit_agency();
    // Since price ratio is 1:1, sell limit == buy will fail because of fees
    suite
        .swap_withdraw_from(
            ca,
            user,
            Uint128::new(4_500_000),
            coin_native(4_500_000, ust),
        )
        .unwrap_err();
    // TODO: How to assert querier error?
}

#[test]
fn same_denom() {
    let user = "user";
    let atom = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_market_token(atom)
        .with_funds(user, &[coin(5_000_000, atom)])
        .with_pool(
            1,
            (coin(100_000_000_000, COMMON), coin(100_000_000_000, atom)),
        )
        .with_pool(
            2,
            (coin(100_000_000_000, atom), coin(100_000_000_000, COMMON)),
        )
        .build();

    suite.deposit(user, &[coin(5_000_000, atom)]).unwrap();

    let ca = suite.credit_agency();
    // Buy 4.5M UST, using maximally 5M ATOM tokens for that
    suite
        .swap_withdraw_from(
            ca,
            user,
            Uint128::new(4_500_000),
            coin_native(4_500_000, atom),
        )
        .unwrap();

    // Excluding swap fees, amount left on contract should be equal to 0.5M tokens,
    // becase no fees are included
    assert!(
        matches!(suite.query_contract_asset_balance().unwrap(), 500_000)
    );
}
