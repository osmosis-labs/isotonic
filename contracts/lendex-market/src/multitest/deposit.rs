use cosmwasm_std::{coin, Coin};

use super::suite::SuiteBuilder;

#[test]
fn deposit_works() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM")])
        .with_base_asset("ATOM")
        .build();

    // At first, the lender has no l-token, and the contract has no base asset.
    assert_eq!(suite.query_asset_balance().unwrap(), 0);
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 0);

    suite.deposit(lender, &[Coin::new(100, "ATOM")]).unwrap();

    // After the deposit, the lender has 100 l-token and the contract has 100 base asset.
    assert_eq!(suite.query_asset_balance().unwrap(), 100);
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 100);
}

#[test]
fn deposit_multiple_denoms_fails() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM"), coin(50, "BTC")])
        .with_base_asset("ATOM")
        .build();

    assert_eq!(
        suite
            .deposit(lender, &[Coin::new(100, "ATOM"), Coin::new(50, "BTC")])
            .unwrap_err()
            .to_string(),
        "Sent too many denoms, must deposit only 'ATOM' in the lending pool"
    );
}

#[test]
fn deposit_wrong_denom_fails() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(50, "BTC")])
        .with_base_asset("ATOM")
        .build();

    assert_eq!(
        suite
            .deposit(lender, &[Coin::new(50, "BTC")])
            .unwrap_err()
            .to_string(),
        "Sent unsupported token, must deposit 'ATOM' in the lending pool"
    );
}

#[test]
fn deposit_nothing_fails() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new().with_base_asset("ATOM").build();

    assert_eq!(
        suite.deposit(lender, &[]).unwrap_err().to_string(),
        "No funds sent"
    );
}
