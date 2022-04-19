use cosmwasm_std::{coin, Decimal, Uint128};
use utils::credit_line::CreditLineValues;

use super::suite::{SuiteBuilder, COMMON};
use crate::error::ContractError;
use isotonic_token::error::ContractError as TokenContractError;

#[test]
fn withdraw_works() {
    let lender = "lender";
    let market_token = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, market_token)])
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(lender).unwrap();

    // Deposit some tokens so we have something to withdraw.
    suite.deposit(lender, &[coin(100, market_token)]).unwrap();

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
    let market_token = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, market_token)])
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(lender).unwrap();

    // Deposit some tokens so we have something to withdraw.
    suite.deposit(lender, &[coin(100, market_token)]).unwrap();

    // Try to withdraw more base asset than we have deposited - should fail and not
    // affect any balances.
    let err = suite.withdraw(lender, 150).unwrap_err();
    assert_eq!(
        TokenContractError::InsufficientTokens {
            available: Uint128::new(100),
            needed: Uint128::new(150)
        },
        err.downcast().unwrap()
    );
    assert_eq!(suite.query_asset_balance(lender).unwrap(), 0);
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 100);
}

#[test]
fn cant_withdraw_with_debt_higher_then_credit_line() {
    let lender = "lender";
    let market_token = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, market_token)])
        .with_collateral_ratio(Decimal::percent(70))
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.deposit(lender, &[coin(100, market_token)]).unwrap();

    // Set debt higher then credit line
    suite
        .set_credit_line(
            lender,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                debt: Uint128::new(200),
            },
        )
        .unwrap();

    let err = suite.withdraw(lender, 1).unwrap_err();
    assert_eq!(
        ContractError::CannotWithdraw {
            amount: Uint128::new(1),
            account: lender.to_owned()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn can_withdraw_up_to_credit_line() {
    let lender = "lender";
    let market_token = "ATOM";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, market_token)])
        .with_collateral_ratio(Decimal::percent(70))
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.deposit(lender, &[coin(100, market_token)]).unwrap();

    // Set appropriate credit line and collateral
    suite
        .set_credit_line(
            lender,
            CreditLineValues {
                collateral: Uint128::new(100),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(70),
                debt: Uint128::zero(),
            },
        )
        .unwrap();

    // Withdraw more then credit line is
    suite.withdraw(lender, 90).unwrap();
    assert_eq!(suite.query_asset_balance(lender).unwrap(), 90);

    // withdrawing another 20 dollars (10 over limit) will fail
    // adjust mock's response
    suite
        .set_credit_line(
            lender,
            CreditLineValues {
                collateral: Uint128::new(10),
                // 100 * 0.7 collateral ratio
                credit_line: Uint128::new(7),
                debt: Uint128::zero(),
            },
        )
        .unwrap();
    let err = suite.withdraw(lender, 20).unwrap_err();
    assert_eq!(
        ContractError::CannotWithdraw {
            amount: Uint128::new(20),
            account: lender.to_owned()
        },
        err.downcast().unwrap()
    );
}

#[test]
fn query_withdrawable_when_only_lending() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM")])
        .with_pool(1, (coin(100, COMMON), coin(100, "ATOM")))
        .with_market_token("ATOM")
        .build();

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_high_credit_line(lender).unwrap();

    // Deposit some tokens so we have something to withdraw.
    suite.deposit(lender, &[coin(100, "ATOM")]).unwrap();

    suite.assert_withdrawable(lender, 100);

    suite.attempt_withdraw_max(lender).unwrap();
}

#[test]
fn query_withdrawable_up_to_credit_line() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM")])
        .with_pool(1, (coin(100, COMMON), coin(100, "ATOM")))
        .with_market_token("ATOM")
        .with_collateral_ratio(Decimal::percent(50))
        .build();

    suite
        .set_credit_line(
            lender,
            CreditLineValues {
                collateral: Uint128::new(100),
                credit_line: Uint128::new(50),
                debt: Uint128::new(40),
            },
        )
        .unwrap();

    // Deposit some tokens so we have something to withdraw.
    suite.deposit(lender, &[coin(100, "ATOM")]).unwrap();

    suite.assert_withdrawable(lender, 20);

    suite.attempt_withdraw_max(lender).unwrap();
}

#[test]
fn query_withdrawable_not_enough_liquid() {
    let lender = "lender";
    let borrower = "borrower";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(100, "ATOM")])
        .with_pool(1, (coin(100, COMMON), coin(100, "ATOM")))
        .with_market_token("ATOM")
        .with_collateral_ratio(Decimal::percent(50))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    suite.deposit(lender, &[coin(100, "ATOM")]).unwrap();
    suite.borrow(borrower, 40).unwrap();

    // Technically, the lender is allowed to withdraw the whole 100 tokens, but
    // the contract doesn't have enough liquidity to cover that!
    suite.assert_withdrawable(lender, 60);
    suite.attempt_withdraw_max(lender).unwrap();
}

// Below: regression tests for rounding issues when withdrawing deposited money
// this possibly belongs with acceptance/system/whatever tests

#[test]
fn withdraw_whole_deposit() {
    let lender = "lender";
    let charge_period = 100;
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(10_000_000_000_000_000_000, "ATOM")])
        .with_charge_period(charge_period)
        .with_pool(
            1,
            (
                coin(10_000_000_000_000_000_000, COMMON),
                coin(10_000_000_000_000_000_000, "ATOM"),
            ),
        )
        .with_market_token("ATOM")
        .build();

    let inputs = [
        1,
        99,
        100,
        999_999,
        1_000_000,
        999_999_999_999,
        1_000_000_000_000,
        999_999_999_999_999_999,
        1_000_000_000_000_000_000,
        9_999_999_999_999_999_999,
        10_000_000_000_000_000_000,
    ];

    for input in inputs {
        suite.set_high_credit_line(lender).unwrap();
        suite.deposit(lender, &[coin(input, "ATOM")]).unwrap();

        // no interest should be charged/earned here as there are no borrowers,
        // so this shouldn't matter
        suite.advance_seconds(charge_period * 2);

        suite.assert_withdrawable(lender, input);
        suite.attempt_withdraw_max(lender).unwrap();
    }
}

#[test]
fn withdraw_whole_deposit_after_being_repaid() {
    let lender = "lender";
    let borrower = "borrower";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(10_000_000_000_000_000_000, "ATOM")])
        .with_pool(
            1,
            (
                coin(10_000_000_000_000_000_000, COMMON),
                coin(10_000_000_000_000_000_000, "ATOM"),
            ),
        )
        .with_market_token("ATOM")
        .build();

    let inputs = [
        1,
        99,
        100,
        999_999,
        1_000_000,
        999_999_999_999,
        1_000_000_000_000,
        999_999_999_999_999_999,
        1_000_000_000_000_000_000,
    ];

    for input in inputs {
        suite.set_high_credit_line(lender).unwrap();
        suite.set_high_credit_line(borrower).unwrap();

        suite.deposit(lender, &[coin(input, "ATOM")]).unwrap();

        suite.assert_borrowable(borrower, input);
        suite.attempt_borrow_max(borrower).unwrap();

        suite.repay(borrower, coin(input, "ATOM")).unwrap();

        suite.assert_withdrawable(lender, input);
        suite.attempt_withdraw_max(lender).unwrap();
    }
}
