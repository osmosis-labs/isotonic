use cosmwasm_std::{coin, Decimal};

use tests::{MarketBuilder, SuiteBuilder};

/// This can expose rounding issues.
/// https://github.com/confio/isotonic/issues/40
#[test]
fn withdraw_whole_deposit() {
    let lender = "lender";
    let charge_period = 100;
    let mut suite = SuiteBuilder::new()
        .with_common_token("OSMO")
        .with_funds(lender, &[coin(u128::MAX, "ATOM")])
        .with_market(MarketBuilder::new("ATOM").with_charge_period(charge_period))
        .with_pool(1, (coin(100, "OSMO"), coin(100, "ATOM")))
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
        suite.deposit(lender, coin(input, "ATOM")).unwrap();

        // no interest should be charged/earned here as there are no borrowers,
        // so this shouldn't matter
        suite.advance_seconds(charge_period * 2);

        suite.assert_withdrawable(lender, coin(input, "ATOM"));
        suite.attempt_withdraw_max(lender, "ATOM").unwrap();
    }
}

/// This can expose rounding issues.
/// https://github.com/confio/isotonic/issues/40
#[test]
fn withdraw_whole_deposit_after_being_repaid() {
    let lender = "lender";
    let borrower = "borrower";
    let mut suite = SuiteBuilder::new()
        .with_common_token("OSMO")
        .with_liquidation_price(Decimal::one())
        .with_funds(lender, &[coin(u128::MAX, "ATOM")])
        .with_funds(borrower, &[coin(u128::MAX, "ETH")])
        .with_pool(1, (coin(1, "OSMO"), coin(1, "ATOM")))
        .with_pool(2, (coin(1, "OSMO"), coin(1, "ETH")))
        .with_market(MarketBuilder::new("ATOM"))
        .with_market(MarketBuilder::new("ETH").with_collateral_ratio(Decimal::percent(99)))
        .build();

    suite
        .deposit(borrower, coin(10_u128.pow(20), "ETH"))
        .unwrap();

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
        suite.deposit(lender, coin(input, "ATOM")).unwrap();

        suite.assert_borrowable(borrower, coin(input, "ATOM"));
        suite.attempt_borrow_max(borrower, "ATOM").unwrap();

        suite.repay(borrower, coin(input, "ATOM")).unwrap();

        suite.assert_withdrawable(lender, coin(input, "ATOM"));
        suite.attempt_withdraw_max(lender, "ATOM").unwrap();

        suite.assert_borrowable(borrower, coin(0, "ATOM"));
    }
}
