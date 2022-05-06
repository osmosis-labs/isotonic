use super::suite::{SuiteBuilder, COMMON};
use crate::error::ContractError;

use cosmwasm_std::{coin, Addr, Decimal, Uint128};
use utils::{coin::coin_native, credit_line::CreditLineValues};

#[test]
fn not_on_market() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    let user = "user";
    let osmo = "OSMO";
    let ust = "UST";
    suite
        .create_market_quick(
            "gov",
            "osmo",
            osmo,
            None,
            (Decimal::zero(), Decimal::zero()),
            None,
        )
        .unwrap();
    let osmo_market = suite.query_market(osmo).unwrap().market;

    suite
        .create_market_quick(
            "gov",
            "ust",
            ust,
            None,
            (Decimal::zero(), Decimal::zero()),
            None,
        )
        .unwrap();
    let ust_market = suite.query_market(ust).unwrap().market;

    let err = suite
        .repay_with_collateral(
            user,
            coin_native(1_000_000, osmo),
            coin_native(1_000_000, "UST"),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::NotOnMarket {
            address: Addr::unchecked(user),
            market: osmo_market.clone()
        },
        err.downcast().unwrap()
    );

    suite.enter_market(osmo_market.as_str(), user).unwrap();
    let err = suite
        .repay_with_collateral(
            user,
            coin_native(1_000_000, osmo),
            coin_native(1_000_000, ust),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::NotOnMarket {
            address: Addr::unchecked(user),
            market: ust_market
        },
        err.downcast().unwrap()
    );
}

#[test]
fn on_two_markets() {
    let deposit_one = "deposit1";
    let deposit_two = "deposit2";
    let user = "user";

    let osmo_denom = "OSMO";
    let eth_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(deposit_one, &[coin(10_000_000, osmo_denom)])
        .with_funds(deposit_two, &[coin(10_000_000, eth_denom)])
        .with_funds(user, &[coin(5_000_000, osmo_denom)])
        .with_pool(1, (coin(20_000_000, COMMON), coin(10_000_000, osmo_denom))) // 2.0
        .with_pool(2, (coin(5_000_000, COMMON), coin(10_000_000, eth_denom))) // 0.5
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
        .deposit_tokens_on_market(deposit_one, coin(10_000_000, osmo_denom))
        .unwrap();
    suite
        .deposit_tokens_on_market(deposit_two, coin(10_000_000, eth_denom))
        .unwrap();

    // User creates a credit line through collateral
    suite
        .deposit_tokens_on_market(user, coin(1_000_000, osmo_denom))
        .unwrap();
    // User goes into debt, but is still liquid
    suite
        .borrow_tokens_from_market(user, coin(1_000_000, eth_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(user).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 1_000_000 OSMO deposited * 2.0 oracle's price
            collateral: Uint128::new(2_000_000),
            // 1000_000 OSMO collateral * 2.0 oracle's price * 0.5 default collateral_ratio
            credit_line: Uint128::new(1_000_000),
            // 1_000_000 ETH borrowed * 0.5 oracle's price
            debt: Uint128::new(500_000)
        }
        .make_response(suite.common_token().clone())
    );

    suite
        .repay_with_collateral(
            user,
            coin_native(1_000_000, osmo_denom),
            coin_native(1_000_000, eth_denom),
        )
        .unwrap();
    let total_credit_line = suite.query_total_credit_line(user).unwrap();
    suite.reset_pools().unwrap();

    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(1_346_662),
            credit_line: Uint128::new(673_331),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone())
    );
}
