use cosmwasm_std::{coin, Decimal, StdError};

use super::suite::SuiteBuilder;

#[test]
fn query_pool_id_both_ways() {
    let suite = SuiteBuilder::new()
        .with_pool(2, (coin(100, "ATOM"), coin(200, "OSMO")))
        .build();

    assert_eq!(suite.query_pool_id("ATOM", "OSMO").unwrap(), 2);
    assert_eq!(suite.query_pool_id("OSMO", "ATOM").unwrap(), 2);
}

#[test]
fn query_price_both_ways() {
    let suite = SuiteBuilder::new()
        .with_pool(2, (coin(100, "ATOM"), coin(200, "OSMO")))
        .build();

    assert_eq!(
        suite.query_price("ATOM", "OSMO").unwrap(),
        Decimal::percent(200)
    );
    assert_eq!(
        suite.query_price("OSMO", "ATOM").unwrap(),
        Decimal::percent(50)
    );
}

#[test]
fn query_unknown_pool() {
    let suite = SuiteBuilder::new().build();

    let err = suite.query_pool_id("ATOM", "OSMO").unwrap_err();
    assert_eq!(
        StdError::generic_err(
            "Querier contract error: There is no info about the prices for this trading pair: ATOM, OSMO"
        ),
        err.downcast().unwrap()
    );
}
