use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, Coin, Decimal, Timestamp};
use lendex_token::DisplayAmount;

use crate::msg::InterestResponse;
use crate::state::SECONDS_IN_YEAR;

#[test]
fn query_interest() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(150, market_token)])
        .with_market_token(market_token)
        .build();

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_token_ratio_one().unwrap();
    suite.set_high_credit_line(borrower).unwrap();

    // At first, the lender has no l-token, and the contract has no base asset.
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 0);
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 0);

    // And, we are at base interest, with no utilisation
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            interest: Decimal::percent(3),
            utilisation: Decimal::zero(),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(100, market_token)])
        .unwrap();

    // After the deposit, the lender has 100 l-token and the contract has 100 base asset.
    assert_eq!(suite.query_ltoken_balance(lender).unwrap().u128(), 100);
    assert_eq!(suite.query_contract_asset_balance().unwrap(), 100);

    // We still are at base interest, with no utilisation
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            interest: Decimal::percent(3),
            utilisation: Decimal::zero(),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Borrow some tokens
    suite.borrow(borrower, 10).unwrap();

    // Now utilisation is 10% (10/100),
    // The interest changed according to the linear formula: 3% + 20% * 10% = 3% + 2% = 5%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(10),
            interest: Decimal::percent(3) + Decimal::percent(2),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Repay some tokens
    suite.repay(borrower, Coin::new(5, market_token)).unwrap();

    // Utilisation is now 5% ((10-5)/100).
    // The interest changed according to the linear formula: 3% + 20% * 5% = 3% + 1% = 4%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(5),
            interest: Decimal::percent(3) + Decimal::percent(1),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    // Lend some more
    suite
        .deposit(lender, &[Coin::new(50, market_token)])
        .unwrap();

    // Utilisation is now ~3.33% ((10-5)/(100+50)).
    // The interest changed according to the linear formula: 3% + 20% * 3.33% = 3% + 0.67% = 3.67%.
    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::from_ratio(1u8, 30u8),
            interest: Decimal::percent(3) + Decimal::from_ratio(1u8, 150u8),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );
}

#[test]
fn charge_interest_borrow() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(2000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_interest(4, 20)
        .with_market_token(market_token)
        .build();

    // Set arbitrary market/common exchange ratio and credit line (not part of this test)
    suite.set_token_ratio_one().unwrap();
    suite.set_high_credit_line(borrower).unwrap();

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(2000, market_token)])
        .unwrap();

    // Borrow some tokens
    suite.borrow(borrower, 1600).unwrap();

    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(80),
            interest: Decimal::percent(20),
            charge_period: Timestamp::from_seconds(300),
        },
        resp
    );

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // Repay some tokens
    // interest is 20%
    // that means btoken 1600 + 320
    // repay 800 -> 1120 left btokens
    suite.repay(borrower, coin(800, market_token)).unwrap();

    assert_eq!(
        suite.query_btoken_info().unwrap().total_supply,
        DisplayAmount::raw(1120u128)
    );
    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // Repay some tokens
    // Utilisation is 48.3%
    // interest is 13.66%
    // btoken 1120 + 13.66% - 800 = 472.992
    suite.repay(borrower, coin(800, market_token)).unwrap();

    // TODO: rounding error
    assert_eq!(
        suite.query_btoken_info().unwrap().total_supply,
        DisplayAmount::raw(472u128)
    );

    // Repay the rest of debt (borrower had extra 500 tokens)
    suite.repay(borrower, coin(474, market_token)).unwrap();
    // TODO: rounding error
    assert_eq!(
        suite.query_btoken_info().unwrap().total_supply,
        DisplayAmount::raw(1u128)
    );
}

#[test]
fn charge_interest_deposit() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(4000, market_token)])
        .with_funds(borrower, &[coin(2300, market_token)])
        .with_interest(4, 20)
        .with_market_token(market_token)
        .build();

    // Set arbitrary market/common exchange ratio and credit lines (not part of this test)
    suite.set_token_ratio_one().unwrap();
    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(2000, market_token)])
        .unwrap();

    // Borrow some tokens
    suite.borrow(borrower, 1600).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // Deposit some tokens
    // interest is 20% (4% base + 20% slope * 80% utilization)
    // that means ltoken 2000 + 1600*20% = 2320
    // deposit 1000 -> 3320 left btokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    // TODO: rounding error
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(3318u128)
    );

    suite.advance_seconds((SECONDS_IN_YEAR) as u64);

    // Repay some tokens
    // Now utilisation is 57.84%,
    // interest rate 15.57%
    // amount of btokens - 1600 + 20% interests = 1920
    // 1920 * 15.57% = 298.94 ltokens interests are made
    // ltokens should go up to 3616.94
    // 3616.94 + 1000 = 4616.94 ltokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    // TODO: rounding error
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(4616u128)
    );

    // Borrower pays all of his debt
    suite.repay(borrower, coin(2219, market_token)).unwrap();
    assert_eq!(
        suite.query_btoken_info().unwrap().total_supply,
        // TODO: rounding error
        DisplayAmount::raw(1u128)
    );

    // ...which allows to withdraw all tokens with interests
    suite.withdraw(lender, 4616).unwrap();
    assert_eq!(suite.query_asset_balance(lender).unwrap(), 4616);
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        // TODO: rounding error
        DisplayAmount::raw(1u128)
    );
}
