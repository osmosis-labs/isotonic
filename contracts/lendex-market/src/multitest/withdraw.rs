use cosmwasm_std::{coin, Coin};

use super::suite::SuiteBuilder;

#[test]
fn withdraw_works() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM")])
        .with_base_asset("ATOM")
        .build();

    // Deposit some tokens so we have something to withdraw.
    suite.deposit(lender, &[Coin::new(100, "ATOM")]).unwrap();

    // After the deposit, the lender has 100 l-token and the contract has 100 base asset.
    // The lender should be able to withdraw 40 tokens.
    suite.withdraw(lender, 40).unwrap();

    assert_eq!(suite.query_asset_balance(lender).unwrap(), 40);
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 60);
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 60);
}

#[test]
fn withdraw_overflow_is_handled() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM")])
        .with_base_asset("ATOM")
        .build();

    // Deposit some tokens so we have something to withdraw.
    suite.deposit(lender, &[Coin::new(100, "ATOM")]).unwrap();

    // After the deposit, the lender has 100 l-token and the contract has 100 base asset.
    // The lender should be able to withdraw 40 tokens.
    assert_eq!(
        suite.withdraw(lender, 150).unwrap_err().to_string(),
        "Performing operation while there is not enough tokens, 100 tokens available, 150 needed"
    );
}
