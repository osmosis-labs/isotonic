use cosmwasm_std::{coin, Decimal};
use std::str::FromStr;

use super::suite::{SuiteBuilder, COMMON};

#[test]
fn nothing_on_market() {
    let market_token = "atom";

    let suite = SuiteBuilder::new()
        .with_market_token(market_token)
        // sell/buy ratio between common_token and market_token is 2.0
        // which means borrowing (buying) 1000 market btokens will get
        // debt of 2000 common tokens
        .with_pool(1, (coin(200, COMMON), coin(100, market_token)))
        .build();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        Decimal::from_str("0.030454529542178457").unwrap()
    );
    assert_eq!(apy.lender, Decimal::zero());
}

#[test]
fn nothing_borrowed() {
    let lender = "lender";
    let market_token = "atom";

    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_market_token(market_token)
        .with_pool(1, (coin(200, COMMON), coin(100, market_token)))
        .build();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        Decimal::from_str("0.030454529542178457").unwrap()
    );
    assert_eq!(apy.lender, Decimal::zero());
}

#[test]
fn half_borrowed() {
    let borrower = "borrower";
    let lender = "lender";
    let market_token = "atom";

    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_market_token(market_token)
        .with_pool(1, (coin(200, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    suite.borrow(borrower, 500).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        Decimal::from_str("0.138828291780615352").unwrap()
    );
    assert_eq!(
        apy.lender,
        Decimal::from_str("0.069414145890307676").unwrap()
    );
}

#[test]
fn whole_borrowed() {
    let borrower = "borrower";
    let lender = "lender";
    let market_token = "atom";

    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_market_token(market_token)
        .with_pool(1, (coin(200, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    suite.borrow(borrower, 1000).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        Decimal::from_str("0.258599693244403384").unwrap()
    );
    assert_eq!(
        apy.lender,
        Decimal::from_str("0.258599693244403384").unwrap()
    );
}

#[test]
fn with_reserve_factor() {
    let borrower = "borrower";
    let lender = "lender";
    let market_token = "atom";

    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_market_token(market_token)
        .with_reserve_factor(20)
        .with_pool(1, (coin(200, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    suite.borrow(borrower, 500).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(
        apy.borrower,
        Decimal::from_str("0.138828291780615352").unwrap()
    );
    assert_eq!(
        apy.lender,
        Decimal::from_str("0.05553131671224614").unwrap()
    );
}
