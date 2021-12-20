use cosmwasm_std::{coin, Coin, StdError, Uint128};

use super::suite::SuiteBuilder;

#[test]
fn deposit_works() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM")])
        .with_market_token("ATOM")
        .build();

    // At first, the lender has no l-token, and the contract has no base asset.
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 0);
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 0);

    suite.deposit(lender, &[Coin::new(100, "ATOM")]).unwrap();

    // After the deposit, the lender has 100 l-token and the contract has 100 base asset.
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 100);
}

#[test]
fn deposit_multiple_denoms_fails() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM"), coin(50, "BTC")])
        .with_market_token("ATOM")
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
        .with_market_token("ATOM")
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
    let mut suite = SuiteBuilder::new().with_market_token("ATOM").build();

    assert_eq!(
        suite.deposit(lender, &[]).unwrap_err().to_string(),
        "No funds sent"
    );
}

#[test]
fn query_transferable_amount() {
    let lender = "lender";
    let market_token = "base";
    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token)
        .with_funds(lender, &[coin(100, market_token)])
        .build();

    let btoken = suite.btoken();
    let resp = suite.query_transferable_amount(btoken, lender).unwrap();
    assert_eq!(Uint128::zero(), resp);

    let ltoken = suite.ltoken();
    let resp = suite
        .query_transferable_amount(ltoken.clone(), lender)
        .unwrap();
    assert_eq!(Uint128::zero(), resp);

    // Deposit base asset and mint some L tokens, then query again
    suite.deposit(lender, &[coin(100, market_token)]).unwrap();
    let resp = suite.query_transferable_amount(ltoken, lender).unwrap();
    assert_eq!(Uint128::new(100), resp);

    let resp = suite
        .query_transferable_amount("xtoken", lender)
        .unwrap_err();
    assert_eq!(
        StdError::generic_err(
            "Querier contract error: Generic error: Unrecognized token: xtoken".to_owned()
        ),
        resp.downcast().unwrap()
    );
}
