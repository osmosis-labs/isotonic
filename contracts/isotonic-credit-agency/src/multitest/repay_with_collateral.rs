use super::suite::{SuiteBuilder, COMMON};

use cosmwasm_std::{coin, Uint128};
use utils::credit_line::CreditLineValues;

#[test]
fn on_two_markets() {
    let deposit_one = "deposit1";
    let deposit_two = "deposit2";
    let user = "user";

    let first_denom = "OSMO";
    let second_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(deposit_one, &[coin(10_000, first_denom)])
        .with_funds(deposit_two, &[coin(10_000, second_denom)])
        .with_funds(user, &[coin(5000, first_denom)])
        .with_pool(1, (coin(150, COMMON), coin(100, first_denom)))
        .with_pool(2, (coin(50, COMMON), coin(100, second_denom)))
        .build();

    suite
        .create_market_quick("gov", "osmo", first_denom, None, None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "ethereum", second_denom, None, None, None)
        .unwrap();

    suite
        .deposit_tokens_on_market(deposit_one, coin(10_000, first_denom))
        .unwrap();
    suite
        .deposit_tokens_on_market(deposit_two, coin(10_000, second_denom))
        .unwrap();

    // User creates a credit line throughj collateral
    suite
        .deposit_tokens_on_market(user, coin(3000, first_denom))
        .unwrap();
    // User goes into debt, but is still liquid
    suite
        .borrow_tokens_from_market(user, coin(1000, second_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(user).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 3_000 deposited * 1.5 oracle's price
            collateral: Uint128::new(4500),
            // 4500 collateral * 1.5 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(2250),
            // 1000 borrowed * 0.5 oracle's price
            debt: Uint128::new(500)
        }
        .make_response(suite.common_token().clone())
    );

    suite
        .repay_with_collateral(user, coin(2000, first_denom), coin(500, second_denom))
        .unwrap();
    let total_credit_line = suite.query_total_credit_line(user).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 3_000 deposited * 1.5 oracle's price
            collateral: Uint128::new(4500),
            // 4500 collateral * 1.5 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(2250),
            // 1000 borrowed * 0.5 oracle's price
            debt: Uint128::new(500)
        }
        .make_response(suite.common_token().clone())
    );
}
