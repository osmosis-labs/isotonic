use crate::state::SECONDS_IN_YEAR;

use super::suite::{SuiteBuilder, COMMON};

use cosmwasm_std::{coin, Decimal, StdError, Uint128};
use utils::credit_line::CreditLineValues;

#[test]
fn oracle_price_not_set() {
    let lender = "lender";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_market_token(market_token)
        .build();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();

    let err = suite.query_credit_line(lender).unwrap_err();
    assert_eq!(
        StdError::generic_err(
            "Querier contract error: Generic error: \
            Querier contract error: There is no info about the prices for this trading pair: atom, COMMON"
        ),
        err.downcast().unwrap(),
    );
}

#[test]
fn zero_credit_line() {
    let lender = "lender";
    let market_token = "atom";
    let suite = SuiteBuilder::new()
        .with_market_token(market_token)
        .with_pool(1, (coin(50, COMMON), coin(100, market_token)))
        .build();

    // No tokens were deposited nor borrowed, so credit line is zero
    let credit_line = suite.query_credit_line(lender).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues::zero().make_response(suite.common_token())
    );
}

#[test]
fn borrower_borrows_tokens() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        // collateral ratio is 0.7
        .with_collateral_ratio(Decimal::percent(70))
        .with_market_token(market_token)
        // sell/buy ratio between common_token and market_token is 2.0
        // which means borrowing (buying) 1000 market btokens will get
        // debt of 2000 common tokens
        .with_pool(1, (coin(200, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();

    // Lender deposits coins
    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    // Now borrower borrows it
    suite.borrow(borrower, 1000).unwrap();

    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 1000);

    let credit_line = suite.query_credit_line(borrower).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            collateral: Uint128::zero(),
            credit_line: Uint128::zero(),
            // 1000 borrowed * 2.0 oracle's price
            debt: Uint128::new(2000),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn lender_deposits_tokens() {
    let lender = "lender";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        // collateral ratio is 0.7
        .with_collateral_ratio(Decimal::percent(70))
        .with_market_token(market_token)
        // sell/buy ratio between common_token and market_token is 2.0
        // so 1000 market tokens will get you 2000 common tokens collateral
        .with_pool(1, (coin(200, COMMON), coin(100, market_token)))
        .build();

    // Deposit some tokens
    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();

    // After the deposit, the lender has 1000 l-token
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 1000);

    let credit_line = suite.query_credit_line(lender).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 2.0 oracle's price
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1400),
            // no debt because of lack of btokens
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn deposits_and_borrows_tokens() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_funds(borrower, &[coin(100, market_token)])
        // collateral ratio is 0.7
        .with_collateral_ratio(Decimal::percent(70))
        .with_market_token(market_token)
        // sell/buy ratio between common_token and market_token is 2.0
        .with_pool(1, (coin(200, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    // Lender deposits coins
    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    // Now borrower borrows it
    suite.borrow(borrower, 1000).unwrap();

    // and deposits all he currently has
    suite
        .deposit(borrower, &[coin(1100, market_token)])
        .unwrap();

    assert_eq!(suite.query_btoken_balance(borrower).unwrap().u128(), 1000);
    assert_eq!(suite.query_ltoken_balance(borrower).unwrap().u128(), 1100);

    let credit_line = suite.query_credit_line(lender).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 2.0 oracle's price
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1400),
            // no debt because of lack of btokens
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
    let credit_line = suite.query_credit_line(borrower).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1100 collateral (deposited) * 2.0 oracle's price
            collateral: Uint128::new(2200),
            // 1100 collateral * 2.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(1540),
            // 1000 borrowed * 2.0 oracle's price
            debt: Uint128::new(2000),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn deposits_and_borrows_tokens_market_common_matches_denoms() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(1000, market_token)])
        .with_funds(borrower, &[coin(100, market_token)])
        .with_collateral_ratio(Decimal::percent(70))
        .with_market_token(market_token)
        .with_common_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();

    suite.deposit(lender, &[coin(1000, market_token)]).unwrap();
    suite.borrow(borrower, 1000).unwrap();
    suite
        .deposit(borrower, &[coin(1100, market_token)])
        .unwrap();

    let credit_line = suite.query_credit_line(lender).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1000 collateral * 1.0 oracle's price (no common_token denom)
            collateral: Uint128::new(1000),
            // 1000 collateral * 0.5 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(700),
            // no debt because of lack of btokens
            debt: Uint128::zero(),
        }
        .make_response(suite.common_token())
    );
    let credit_line = suite.query_credit_line(borrower).unwrap();
    assert_eq!(
        credit_line,
        CreditLineValues {
            // 1100 collateral (deposited) * 1.0 oracle's price
            collateral: Uint128::new(1100),
            // 1100 collateral * 1.0 oracle's price * 0.7 collateral_ratio
            credit_line: Uint128::new(770),
            // 1000 borrowed * 1.0 oracle's price
            debt: Uint128::new(1000),
        }
        .make_response(suite.common_token())
    );
}

#[test]
fn query_credit_line_with_uncharged_interest() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(5000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_charge_period((SECONDS_IN_YEAR) as u64)
        .with_interest(10, 0)
        .with_reserve_factor(0)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    suite.deposit(lender, &[coin(2000, market_token)]).unwrap();

    suite.borrow(borrower, 1000).unwrap();

    suite.assert_debt("borrower", 1000);
    suite.assert_collateral("lender", 2000);

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // we want to make sure the queries return the amount with interest charged
    // even if there was no call to `charge_interest`

    suite.assert_debt("borrower", 1100);
    suite.assert_collateral("lender", 2100);
}
