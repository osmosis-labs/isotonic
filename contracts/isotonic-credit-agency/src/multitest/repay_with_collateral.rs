use super::suite::{SuiteBuilder, COMMON};

use cosmwasm_std::{coin, Decimal, Uint128};
use utils::credit_line::CreditLineValues;

#[test]
fn on_two_markets() {
    let deposit_one = "deposit1";
    let deposit_two = "deposit2";
    let user = "user";

    let osmo_denom = "OSMO";
    let eth_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(deposit_one, &[coin(10_000, osmo_denom)])
        .with_funds(deposit_two, &[coin(10_000, eth_denom)])
        .with_funds(user, &[coin(5000, osmo_denom)])
        // COMMON denom has same value as osmo_denom
        .with_pool(1, (coin(20000, COMMON), coin(10000, osmo_denom)))
        .with_pool(2, (coin(5000, COMMON), coin(10000, eth_denom)))
        .build();

    suite
        .create_market_quick(
            "gov",
            "osmo",
            osmo_denom,
            None,
            (Decimal::zero(), Decimal::zero()),
            None,
        )
        .unwrap();
    suite
        .create_market_quick(
            "gov",
            "ethereum",
            eth_denom,
            None,
            (Decimal::zero(), Decimal::zero()),
            None,
        )
        .unwrap();

    suite
        .deposit_tokens_on_market(deposit_one, coin(10_000, osmo_denom))
        .unwrap();
    suite
        .deposit_tokens_on_market(deposit_two, coin(10_000, eth_denom))
        .unwrap();

    // User creates a credit line through collateral
    suite
        .deposit_tokens_on_market(user, coin(1000, osmo_denom))
        .unwrap();
    // User goes into debt, but is still liquid
    suite
        .borrow_tokens_from_market(user, coin(1000, eth_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(user).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 1000 OSMO deposited * 2.0 oracle's price
            collateral: Uint128::new(2000),
            // 1000 OSMO collateral * 2.0 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(1000),
            // 1000 ETH borrowed * 0.5 oracle's price
            debt: Uint128::new(500)
        }
        .make_response(suite.common_token().clone())
    );

    suite
        .repay_with_collateral(user, coin(1000, osmo_denom), coin(1000, eth_denom))
        .unwrap();
    let total_credit_line = suite.query_total_credit_line(user).unwrap();
    dbg!(suite.query_tokens_balance(osmo_denom, user).unwrap());
    dbg!(suite.query_tokens_balance(eth_denom, user).unwrap());
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // In this test case, osmosis returns estimate collateral needed 591 tokens
            // 1000 OSMO set as maximum usage - 287 estimated = 713
            collateral: Uint128::new(1347),
            credit_line: Uint128::new(628),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone())
    );
}
