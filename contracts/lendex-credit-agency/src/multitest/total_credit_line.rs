use super::suite::SuiteBuilder;

use lendex_market::msg::CreditLineResponse;

use cosmwasm_std::{coin, Decimal, Uint128};

#[test]
fn basic_query_with_one_market() {
    let lender = "lender";
    let market_denom = "OSMO";
    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(lender, &[coin(1000, market_denom)])
        .build();

    suite
        .create_market_quick("gov", "osmo", market_denom)
        .unwrap();

    // Sets sell/buy rate between market denom/common denom as 2.0,
    // which means selling 1000 market denom will result in 2000 common denom
    suite
        .oracle_set_price_market_per_common(market_denom, Decimal::percent(200))
        .unwrap();

    suite
        .deposit_tokens_on_market(lender, coin(1000, market_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(lender).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(2000),
            // 1000 collateral * 2.0 oracle's price * 0.5 collateral_ratio (default in crate_market_quick)
            credit_line: Uint128::new(1000),
            debt: Uint128::zero()
        }
    );
}

#[test]
fn query_with_two_market() {
    let lender = "lender";
    let first_denom = "OSMO";
    let second_denom = "ETH";
    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(lender, &[coin(1000, first_denom), coin(500, second_denom)])
        .build();

    suite
        .create_market_quick("gov", "osmo", first_denom)
        .unwrap();
    suite
        .create_market_quick("gov", "ethereum", second_denom)
        .unwrap();

    // Sets sell/buy rate between market denom/common denom as 2.0,
    // selling 1000 market denom give in 2000 common denom
    suite
        .oracle_set_price_market_per_common(first_denom, Decimal::percent(200))
        .unwrap();
    // here - selling 500 ETH denom will give 250 common denom
    suite
        .oracle_set_price_market_per_common(second_denom, Decimal::percent(50))
        .unwrap();

    suite
        .deposit_tokens_on_market(lender, coin(1000, first_denom))
        .unwrap();
    suite
        .deposit_tokens_on_market(lender, coin(500, second_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(lender).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            // 1000 deposited * 2.0 oracle's price + 500 deposited * 0.5 oracle's price
            collateral: Uint128::new(2250),
            // 1000 collateral * 2.0 oracle's price * 0.5 default collateral_ratio
            //   + 500 collateral * 0.5 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(1125),
            debt: Uint128::zero()
        }
    );
}
