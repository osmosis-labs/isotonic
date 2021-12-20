use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, Coin, Decimal, StdError, Uint128};

use crate::msg::CreditLineResponse;

#[test]
fn oracle_price_not_set() {
    let lender = "lender";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_market_token(market_token)
        .build();

    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    let err = suite.query_credit_line(lender).unwrap_err();
    assert_eq!(
        StdError::generic_err(
            "Querier contract error: Generic error: \
            Querier contract error: There is no info about the prices for this trading pair"
        ),
        err.downcast().unwrap(),
    );
}

#[test]
fn lender_deposits_money() {
    let lender = "lender";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        // collateral ratio is 0.5
        .with_collateral_ratio(Decimal::percent(50))
        .with_market_token(market_token)
        .build();

    // sell/buy ratio between common_token and market_token is 0.5
    suite.oracle_set_price(Decimal::percent(50)).unwrap();

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    // After the deposit, the lender has 1000 l-token
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 1000);

    let credit_line = suite.query_credit_line(lender).unwrap();
    assert_eq!(
        credit_line,
        CreditLineResponse {
            // 1000 collateral * 0.5 oracle's price
            collateral: Uint128::new(500),
            // 1000 collateral * 0.5 oracle's price * 0.5 collateral_ratio
            credit_line: Uint128::new(250),
            // no debt because of lack of btokens
            debt: Uint128::zero(),
        }
    );
}

#[test]
fn deposits_and_borrows_money() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_funds(borrower, &[coin(100, market_token)])
        // collateral ratio is 0.5
        .with_collateral_ratio(Decimal::percent(50))
        .with_market_token(market_token)
        .build();

    // sell/buy ratio between common_token and market_token is 0.5
    suite.oracle_set_price(Decimal::percent(50)).unwrap();

    // Lender deposits coints
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();
    // Now borrower borrows it
    suite.borrow(borrower, 1000).unwrap();

    // and deposits all he currently has
    suite
        .deposit(borrower, &[Coin::new(1100, market_token)])
        .unwrap();

    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 1000);
    assert_eq!(suite.query_ltoken_balance(borrower).unwrap().u128(), 1100);

    let credit_line = suite.query_credit_line(lender).unwrap();
    assert_eq!(
        credit_line,
        CreditLineResponse {
            // 1000 collateral * 0.5 oracle's price
            collateral: Uint128::new(500),
            // 1000 collateral * 0.5 oracle's price * 0.5 collateral_ratio
            credit_line: Uint128::new(250),
            // no debt because of lack of btokens
            debt: Uint128::zero(),
        }
    );
    let credit_line = suite.query_credit_line(borrower).unwrap();
    assert_eq!(
        credit_line,
        CreditLineResponse {
            // 1100 collateral (deposited) * 0.5 oracle's price
            collateral: Uint128::new(550),
            // 1100 collateral * 0.5 oracle's price * 0.5 collateral_ratio
            credit_line: Uint128::new(275),
            // 1000 borrowed * 0.5 oracle's price
            debt: Uint128::new(500),
        }
    );
}
