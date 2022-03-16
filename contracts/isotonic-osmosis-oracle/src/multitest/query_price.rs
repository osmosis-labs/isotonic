use cosmwasm_std::{coin, Decimal};

use crate::ContractError;

use super::suite::SuiteBuilder;

#[test]
fn query_price_both_ways() {
    let suite = SuiteBuilder::new()
        .with_pool(2, (coin(100, "ATOM"), coin(200, "OSMO")))
        .build();

    assert_eq!(suite.query_price("ATOM", "OSMO"), Ok(Decimal::percent(200)));
    assert_eq!(suite.query_price("OSMO", "ATOM"), Ok(Decimal::percent(50)));
}

#[test]
fn query_unknown_price() {
    let suite = SuiteBuilder::new().build();

    let err = suite.query_price("ATOM", "OSMO").unwrap_err();
    let expected_err_msg = ContractError::NoInfo {}.to_string();
    assert!(err.to_string().contains(&expected_err_msg));
}
