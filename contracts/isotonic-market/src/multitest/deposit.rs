use cosmwasm_std::{coin, Decimal, StdError, Uint128};
use utils::credit_line::CreditLineValues;

use super::suite::{SuiteBuilder, COMMON};
use crate::error::ContractError;

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

    suite.deposit(lender, &[coin(100, "ATOM")]).unwrap();

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

    let err = suite
        .deposit(lender, &[coin(100, "ATOM"), coin(50, "BTC")])
        .unwrap_err();
    assert_eq!(
        ContractError::ExtraDenoms("ATOM".to_owned()),
        err.downcast().unwrap()
    );
}

#[test]
fn deposit_wrong_denom_fails() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(50, "BTC")])
        .with_market_token("ATOM")
        .build();

    let err = suite.deposit(lender, &[coin(50, "BTC")]).unwrap_err();
    assert_eq!(
        ContractError::InvalidDenom("ATOM".to_owned()),
        err.downcast().unwrap()
    );
}

#[test]
fn deposit_nothing_fails() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new().with_market_token("ATOM").build();

    let err = suite.deposit(lender, &[]).unwrap_err();
    assert_eq!(ContractError::NoFundsSent {}, err.downcast().unwrap());
}

#[test]
fn query_transferable_amount() {
    let lender = "lender";
    let market_token = "base";
    let mut suite = SuiteBuilder::new()
        .with_market_token(market_token)
        .with_funds(lender, &[coin(100, market_token)])
        .with_collateral_ratio(Decimal::percent(80))
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    // Set zero credit line in mock
    suite
        .set_credit_line(lender, CreditLineValues::zero())
        .unwrap();

    let btoken = suite.btoken();
    let resp = suite.query_transferable_amount(btoken, lender).unwrap();
    assert_eq!(Uint128::zero(), resp.transferable);

    let ltoken = suite.ltoken();
    let resp = suite
        .query_transferable_amount(ltoken.clone(), lender)
        .unwrap();
    assert_eq!(Uint128::zero(), resp.transferable);

    // Deposit base asset and mint some L tokens, then query again
    suite.deposit(lender, &[coin(100, market_token)]).unwrap();

    // Set appropriate credit line
    suite
        .set_credit_line(
            lender,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.8 collateral ratio
                credit_line: Uint128::new(80),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    // Transferable amount is equal to collateral
    let resp = suite
        .query_transferable_amount(ltoken.clone(), lender)
        .unwrap();
    assert_eq!(Uint128::new(100), resp.transferable);

    // Set credit line with debt
    suite
        .set_credit_line(
            lender,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.8 collateral ratio
                credit_line: Uint128::new(80),
                debt: Uint128::new(50),
            },
        )
        .unwrap();

    // Transferable amount is equal to collateral / (credit_line - debt)
    let resp = suite.query_transferable_amount(ltoken, lender).unwrap();
    assert_eq!(Uint128::new(37), resp.transferable);

    let err = suite
        .query_transferable_amount("xtoken", lender)
        .unwrap_err();
    assert_eq!(
        StdError::generic_err("Querier contract error: Unrecognised token: xtoken".to_owned()),
        err.downcast().unwrap()
    );
}

#[test]
fn cannot_deposit_over_cap() {
    let mut suite = SuiteBuilder::new()
        .with_funds("alice", &[coin(100, "ATOM")])
        .with_market_token("ATOM")
        .with_cap(90u128)
        .build();

    // This is okay.
    suite.deposit("alice", &[coin(80, "ATOM")]).unwrap();

    // This one pushes things over the cap.
    let err = suite.deposit("alice", &[coin(20, "ATOM")]).unwrap_err();
    assert_eq!(
        ContractError::DepositOverCap {
            attempted_deposit: Uint128::from(20u128),
            ltoken_supply: Uint128::from(80u128),
            cap: Uint128::from(90u128)
        },
        err.downcast().unwrap()
    );
}
