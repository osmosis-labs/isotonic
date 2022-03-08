use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, Decimal, Uint128};
use utils::credit_line::CreditLineValues;
use utils::token::Token;

#[test]
fn lender_on_one_market() {
    let lender = "lender";
    let market_denom = "OSMO";
    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(lender, &[coin(1000, market_denom)])
        .build();

    suite
        .create_market_quick("gov", "osmo", market_denom, None, None, None)
        .unwrap();

    // Sets sell/buy rate between market denom/common denom as 2.0,
    // which means selling 1000 market denom will result in 2000 common denom
    suite
        .oracle_set_price_market_per_common(
            Token::Native(market_denom.to_owned()),
            Decimal::percent(200),
        )
        .unwrap();

    suite
        .deposit_tokens_on_market(lender, coin(1000, market_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(lender).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.5 collateral_ratio (default in crate_market_quick)
            credit_line: Uint128::new(1000),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone().native().unwrap())
    );
}

#[test]
fn lender_on_three_markets() {
    let lender = "lender";
    let first_denom = "OSMO";
    let second_denom = "ETH";
    let third_denom = "BTC";
    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(
            lender,
            &[
                coin(1000, first_denom),
                coin(500, second_denom),
                coin(7, third_denom),
            ],
        )
        .build();

    suite
        .create_market_quick("gov", "osmo", first_denom, None, None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "ethereum", second_denom, None, None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "bitcoin", third_denom, None, None, None)
        .unwrap();

    // Sets sell/buy rate between market denom/common denom as 2.0,
    // selling 1000 OSMO denom gives 2000 common denom
    suite
        .oracle_set_price_market_per_common(
            Token::Native(first_denom.to_owned()),
            Decimal::percent(200),
        )
        .unwrap();
    // here - selling 500 ETH denom gives 250 common denom
    suite
        .oracle_set_price_market_per_common(
            Token::Native(second_denom.to_owned()),
            Decimal::percent(50),
        )
        .unwrap();
    // here - selling 7 BTC denom gives 7000 common denom
    suite
        .oracle_set_price_market_per_common(
            Token::Native(third_denom.to_owned()),
            Decimal::percent(100_000),
        )
        .unwrap();

    suite
        .deposit_tokens_on_market(lender, coin(1000, first_denom))
        .unwrap();
    suite
        .deposit_tokens_on_market(lender, coin(500, second_denom))
        .unwrap();
    suite
        .deposit_tokens_on_market(lender, coin(7, third_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(lender).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 1000 deposited * 2.0 oracle's price + 500 deposited * 0.5 oracle's price
            //   + 7 * 1000.0 oracle's price
            collateral: Uint128::new(9250),
            // 1000 collateral * 2.0 oracle's price * 0.5 default collateral_ratio
            //   + 500 collateral * 0.5 oracle's price * 0.5 default collateral_ratio
            //   + 7 collateral * 1000.0 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(4625),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone().native().unwrap())
    );
}

#[test]
fn lender_on_two_markets_with_two_borrowers() {
    let lender = "lender";
    let borrower_one = "borrower1";
    let borrower_two = "borrower2";

    let first_denom = "OSMO";
    let second_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(lender, &[coin(100, first_denom), coin(500, second_denom)])
        .with_funds(borrower_one, &[coin(1000, first_denom)])
        .with_funds(borrower_two, &[coin(1500, second_denom)])
        .build();

    suite
        .create_market_quick("gov", "osmo", first_denom, None, None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "ethereum", second_denom, None, None, None)
        .unwrap();

    // Sets sell/buy rate between market denom/common denom as 2.0,
    // selling 1000 market denom gives 2000 common denom
    suite
        .oracle_set_price_market_per_common(
            Token::Native(first_denom.to_owned()),
            Decimal::percent(200),
        )
        .unwrap();
    // here - selling 500 ETH denom gives 250 common denom
    suite
        .oracle_set_price_market_per_common(
            Token::Native(second_denom.to_owned()),
            Decimal::percent(50),
        )
        .unwrap();

    // Lender deposits all his money
    suite
        .deposit_tokens_on_market(lender, coin(100, first_denom))
        .unwrap();
    suite
        .deposit_tokens_on_market(lender, coin(500, second_denom))
        .unwrap();

    // First borrower deposits 1000 owned tokens and then borrows
    suite
        .deposit_tokens_on_market(borrower_one, coin(1000, first_denom))
        .unwrap();
    suite
        .borrow_tokens_from_market(borrower_one, coin(500, second_denom))
        .unwrap();

    // Second borrower deposits 1500 owned tokens and then borrows
    suite
        .deposit_tokens_on_market(borrower_two, coin(1500, second_denom))
        .unwrap();
    suite
        .borrow_tokens_from_market(borrower_two, coin(100, first_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(lender).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 100 deposited * 2.0 oracle's price + 500 deposited * 0.5 oracle's price
            collateral: Uint128::new(450),
            // 100 collateral * 2.0 oracle's price * 0.5 default collateral_ratio
            //   + 500 collateral * 0.5 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(225),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone().native().unwrap())
    );

    let total_credit_line = suite.query_total_credit_line(borrower_one).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 1000 deposited * 2.0 oracle's price
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(1000),
            // 500 borrowed * 0.5 oracle's price (second denom)
            debt: Uint128::new(250)
        }
        .make_response(suite.common_token().clone().native().unwrap())
    );

    let total_credit_line = suite.query_total_credit_line(borrower_two).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 1500 deposited * 0.5 oracle's price
            collateral: Uint128::new(750),
            // 1500 collateral * 0.5 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(375),
            // 100 borrowed * 2.0 oracle's price (first denom)
            debt: Uint128::new(200)
        }
        .make_response(suite.common_token().clone().native().unwrap())
    );
}

#[test]
fn two_lenders_with_borrower_on_two_markets() {
    let lender_one = "lender1";
    let lender_two = "lender2";
    let borrower = "borrower";

    let first_denom = "OSMO";
    let second_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(lender_one, &[coin(500, first_denom)])
        .with_funds(lender_two, &[coin(300, second_denom)])
        .with_funds(borrower, &[coin(3000, first_denom)])
        .build();

    suite
        .create_market_quick("gov", "osmo", first_denom, None, None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "ethereum", second_denom, None, None, None)
        .unwrap();

    // Sets sell/buy rate between market denom/common denom as 1.5,
    // selling 500 market denom gives 750 common denom
    suite
        .oracle_set_price_market_per_common(
            Token::Native(first_denom.to_owned()),
            Decimal::percent(150),
        )
        .unwrap();
    // here - selling 300 ETH denom gives 150 common denom
    suite
        .oracle_set_price_market_per_common(
            Token::Native(second_denom.to_owned()),
            Decimal::percent(50),
        )
        .unwrap();

    // Lenders deposits all his money
    suite
        .deposit_tokens_on_market(lender_one, coin(500, first_denom))
        .unwrap();
    suite
        .deposit_tokens_on_market(lender_two, coin(300, second_denom))
        .unwrap();

    // Borrower deposits his tokens on first market, then borrows from first and second market
    suite
        .deposit_tokens_on_market(borrower, coin(3000, first_denom))
        .unwrap();
    suite
        .borrow_tokens_from_market(borrower, coin(500, first_denom))
        .unwrap();
    suite
        .borrow_tokens_from_market(borrower, coin(300, second_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(lender_one).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 500 deposited * 1.5 oracle's price
            collateral: Uint128::new(750),
            // 500 collateral * 1.5 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(375),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone().native().unwrap())
    );

    let total_credit_line = suite.query_total_credit_line(lender_two).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 300 deposited * 0.5 oracle's price
            collateral: Uint128::new(150),
            // 300 collateral * 0.5 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(75),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone().native().unwrap())
    );

    let total_credit_line = suite.query_total_credit_line(borrower).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 3000 deposited * 1.5 oracle's price
            collateral: Uint128::new(4500),
            // 3000 collateral * 1.5 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(2250),
            // 500 borrowed * 1.5 oracle's price + 300 borrowed * 0.5 oracle's price
            debt: Uint128::new(900)
        }
        .make_response(suite.common_token().clone().native().unwrap())
    );
}
