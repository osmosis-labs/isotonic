use super::suite::{SuiteBuilder, COMMON};
use crate::state::SECONDS_IN_YEAR;

use cosmwasm_std::{coin, Coin, Decimal, Uint128};
use isotonic_token::DisplayAmount;

use utils::interest::{Interest, ValidatedInterest};

#[test]
fn adjust_collateral_ratio() {
    let mut suite = SuiteBuilder::new()
        .with_collateral_ratio(Decimal::percent(15))
        .build();

    suite.sudo_adjust_collateral_ratio(30).unwrap();

    assert_eq!(
        Decimal::percent(30),
        suite.query_config().unwrap().collateral_ratio
    );
}

#[test]
fn adjust_reserve_factor() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_charge_period((SECONDS_IN_YEAR) as u64)
        .with_funds(lender, &[coin(4000, market_token)])
        .with_funds(borrower, &[coin(2300, market_token)])
        .with_interest(4, 20)
        .with_reserve_factor(10)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(2000, market_token)])
        .unwrap();

    // Borrow some tokens
    suite.borrow(borrower, 1600).unwrap();

    // Point of test - change reserve factor
    suite.sudo_adjust_reserve_factor(30).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // interests are 20% (4% base + 20% slope * 80% utilization)
    // bMul (btoken_ratio) = 20% after full year
    // charged interests = 20% * 1600 = 320
    // reserve = 30% * charged interests = 96

    // deposit some tokens to trigger charging
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    assert_eq!(
        Decimal::percent(30),
        suite.query_config().unwrap().reserve_factor
    );

    assert_eq!(suite.query_reserve().unwrap(), Uint128::new(96));
}

#[test]
fn adjust_price_oracle() {
    let mut suite = SuiteBuilder::new().build();

    let new_oracle = "some new oracle";
    suite.sudo_adjust_price_oracle(new_oracle).unwrap();

    assert_eq!(new_oracle, suite.query_config().unwrap().price_oracle);
}

#[test]
fn adjust_market_cap() {
    let mut suite = SuiteBuilder::new().with_cap(Uint128::new(100)).build();

    let new_cap = Some(Uint128::new(333));
    suite.sudo_adjust_market_cap(new_cap).unwrap();

    assert_eq!(new_cap, suite.query_config().unwrap().market_cap);

    let new_cap = None;
    suite.sudo_adjust_market_cap(new_cap).unwrap();

    assert_eq!(new_cap, suite.query_config().unwrap().market_cap);
}

#[test]
fn adjust_interest_rates() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_charge_period((SECONDS_IN_YEAR) as u64)
        .with_funds(lender, &[coin(4000, market_token)])
        .with_funds(borrower, &[coin(2300, market_token)])
        .with_interest(4, 20)
        .with_reserve_factor(0)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(2000, market_token)])
        .unwrap();

    // Borrow some tokens
    suite.borrow(borrower, 1000).unwrap();

    let new_interests = Interest::Linear {
        base: Decimal::percent(5),
        slope: Decimal::percent(50),
    };
    suite
        .sudo_adjust_interest_rates(new_interests.clone())
        .unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // interests are 30% (5% base + 50% slope * 50% utilization)
    // bMul (btoken_ratio) = 30% after full year
    // charged interests = 30% * 1000 = 300
    // liquid assets = 1000
    // ltokens supplied = 1000 + 1000 = 2000
    // lMul (ltoken_ratio) = borrowed * bMul / lMul = 1000 * 0.3 / 2000 ~= 0.15
    // that means ltokens 2000 * 1.15 = 2300
    // deposit 1000 -> 3300 left btokens

    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    assert_eq!(
        ValidatedInterest::unchecked(new_interests),
        suite.query_config().unwrap().rates
    );

    // TODO: Rounding issue
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(3299u128)
    );
}
