use cosmwasm_std::coin;

use tests::{MarketBuilder, SuiteBuilder};

/// This can expose rounding issues.
/// https://github.com/confio/isotonic/issues/40
#[test]
#[ignore]
fn withdraw_whole_deposit() {
    let lender = "lender";
    let charge_period = 100;
    let mut suite = SuiteBuilder::new()
        .with_common_token("OSMO")
        .with_funds(lender, &[coin(10 ^ 19, "ATOM")])
        .with_market(MarketBuilder::new("ATOM").with_charge_period(charge_period))
        .with_pool(1, (coin(10 ^ 19, "OSMO"), coin(10 ^ 19, "ATOM")))
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
#[ignore]
fn withdraw_whole_deposit_after_being_repaid() {
    let lender = "lender";
    let borrower = "borrower";
    let mut suite = SuiteBuilder::new()
        .with_common_token("OSMO")
        .with_funds(lender, &[coin(10_000_000_000_000_000_000, "ATOM")])
        .with_pool(
            1,
            (
                coin(10_000_000_000_000_000_000, "OSMO"),
                coin(10_000_000_000_000_000_000, "ATOM"),
            ),
        )
        .with_pool(
            2,
            (
                coin(10_000_000_000_000_000_000, "OSMO"),
                coin(10_000_000_000_000_000_000, "ETH"),
            ),
        )
        .with_market(MarketBuilder::new("ATOM"))
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
        suite.deposit(lender, coin(input, "ATOM")).unwrap();

        suite.assert_borrowable(borrower, coin(input, "ATOM"));
        suite.attempt_borrow_max(borrower, "ATOM").unwrap();

        suite.repay(borrower, coin(input, "ATOM")).unwrap();

        suite.assert_withdrawable(lender, coin(input, "ATOM"));
        suite.attempt_withdraw_max(lender, "ATOM").unwrap();
    }
}
