use cosmwasm_std::{coin, Decimal};

use super::suite::SuiteBuilder;

#[test]
fn nothing_on_market() {
    let market_token = "atom";

    let mut suite = SuiteBuilder::new().with_market_token(market_token).build();

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_token_ratio_one().unwrap();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market btokens will get
    // debt of 2000 common tokens
    suite
        .oracle_set_price_market_per_common(Decimal::percent(200))
        .unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(apy.borrower, "0.030454529542178457".parse().unwrap());
    assert_eq!(apy.lender, Decimal::zero());
}

#[test]
fn nothing_borrowed() {
    let lender = "lender";
    let market_token = "atom";

    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_market_token(market_token)
        .build();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_token_ratio_one().unwrap();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market btokens will get
    // debt of 2000 common tokens
    suite
        .oracle_set_price_market_per_common(Decimal::percent(200))
        .unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(apy.borrower, "0.030454529542178457".parse().unwrap());
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
        .build();

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_token_ratio_one().unwrap();

    suite.set_high_credit_line(borrower).unwrap();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market btokens will get
    // debt of 2000 common tokens
    suite
        .oracle_set_price_market_per_common(Decimal::percent(200))
        .unwrap();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    suite.borrow(borrower, 500).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(apy.borrower, "0.138828291780615352".parse().unwrap());
    assert_eq!(apy.lender, "0.069414145890307676".parse().unwrap());
}

#[test]
fn whole_borrowed() {
    let borrower = "borrower";
    let lender = "lender";
    let market_token = "atom";

    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_market_token(market_token)
        .build();

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_token_ratio_one().unwrap();

    suite.set_high_credit_line(borrower).unwrap();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market btokens will get
    // debt of 2000 common tokens
    suite
        .oracle_set_price_market_per_common(Decimal::percent(200))
        .unwrap();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    suite.borrow(borrower, 1000).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(apy.borrower, "0.258599693244403384".parse().unwrap());
    assert_eq!(apy.lender, "0.258599693244403384".parse().unwrap());
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
        .build();

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_token_ratio_one().unwrap();

    suite.set_high_credit_line(borrower).unwrap();

    // sell/buy ratio between common_token and market_token is 2.0
    // which means borrowing (buying) 1000 market btokens will get
    // debt of 2000 common tokens
    suite
        .oracle_set_price_market_per_common(Decimal::percent(200))
        .unwrap();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    suite.borrow(borrower, 500).unwrap();

    let apy = suite.query_apy().unwrap();
    assert_eq!(apy.borrower, "0.138828291780615352".parse().unwrap());
    assert_eq!(apy.lender, "0.05553131671224614".parse().unwrap());
}
