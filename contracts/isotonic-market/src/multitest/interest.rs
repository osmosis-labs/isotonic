use super::suite::{SuiteBuilder, COMMON};

use cosmwasm_std::{coin, Coin, Decimal, Timestamp};
use isotonic_token::DisplayAmount;

use crate::msg::InterestResponse;
use crate::state::SECONDS_IN_YEAR;

const YEAR: u64 = (SECONDS_IN_YEAR) as u64;
const QUARTER: u64 = YEAR / 4;

#[test]
fn query_interest() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(150, market_token)])
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

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
        .with_charge_period(YEAR)
        .with_funds(lender, &[coin(2000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_interest(4, 20)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

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
            charge_period: Timestamp::from_seconds(YEAR),
        },
        resp
    );

    suite.advance_seconds(YEAR);

    // Repay some tokens
    // interests are 20%
    // that means btoken 1600 + 320
    // repay 800 -> 1120 left btokens
    suite.repay(borrower, coin(800, market_token)).unwrap();

    assert_eq!(
        suite.query_btoken_info().unwrap().total_supply,
        DisplayAmount::raw(1120u128)
    );
    suite.advance_seconds(YEAR);

    // Repay some tokens
    // Utilisation is 48.3%
    // interests are 13.66%
    // btoken 1120 + 13.66% - 800 = 472.992
    suite.repay(borrower, coin(800, market_token)).unwrap();

    assert_eq!(
        suite.query_btoken_info().unwrap().total_supply,
        DisplayAmount::raw(472u128)
    );

    // Repay the rest of debt (borrower had extra 500 tokens)
    suite.repay(borrower, coin(474, market_token)).unwrap();
    assert_eq!(
        suite.query_btoken_info().unwrap().total_supply,
        DisplayAmount::zero()
    );
}

#[test]
fn charge_interest_deposit() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_funds(lender, &[coin(4000, market_token)])
        .with_funds(borrower, &[coin(2300, market_token)])
        .with_interest(4, 20)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(2000, market_token)])
        .unwrap();

    // Borrow some tokens
    suite.borrow(borrower, 1600).unwrap();

    suite.advance_seconds(YEAR);

    // Deposit some tokens
    // interest is 20% (4% base + 20% slope * 80% utilization)
    // that means ltoken 2000 + 1600*20% = 2320
    // deposit 1000 -> 3320 left btokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(3319u128)
    );

    suite.advance_seconds(YEAR);

    // Repay some tokens
    // Now utilisation is 57.85%,
    // interest rate 15.57%
    // amount of btokens - 1600 + 20% interests = 1920
    // 1920 * 15.57% = 298.94 ltokens interests are made
    // ltokens should go up to 3618.14
    // 3618.14 + 1000 = 4618.14 ltokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(4618u128)
    );

    // Borrower pays all of his debt
    suite.repay(borrower, coin(2219, market_token)).unwrap();
    assert_eq!(
        suite.query_btoken_info().unwrap().total_supply,
        DisplayAmount::zero()
    );

    // ...which allows to withdraw all tokens with interests
    suite.withdraw(lender, 4616).unwrap();
    assert_eq!(suite.query_asset_balance(lender).unwrap(), 4616);
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        // TODO: rounding error
        DisplayAmount::raw(2u128)
    );
}

#[test]
fn query_balance_with_uncharged_interest() {
    // We want to make sure if we query for balance with interest that hasn't been charged yet,
    // the query will display the value with interest included.

    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_charge_period(YEAR)
        .with_funds(lender, &[coin(2000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_interest(10, 20)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();

    suite
        .deposit(lender, &[Coin::new(2000, market_token)])
        .unwrap();
    suite.borrow(borrower, 500).unwrap();

    let resp = suite.query_interest().unwrap();
    assert_eq!(
        InterestResponse {
            utilisation: Decimal::percent(25),
            interest: Decimal::percent(15),
            charge_period: Timestamp::from_seconds(YEAR),
        },
        resp
    );

    suite.assert_ltoken_balance("lender", 2000u128);
    suite.assert_btoken_balance("borrower", 500u128);

    suite.advance_seconds(YEAR);

    suite.assert_ltoken_balance("lender", 2075u128);
    suite.assert_btoken_balance("borrower", 575u128);
}

#[test]
fn compounding_interest() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(5000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_charge_period(QUARTER)
        .with_interest(40, 0) // 40% annual, 10% quarterly
        .with_reserve_factor(15)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    suite
        .deposit(lender, &[Coin::new(2000, market_token)])
        .unwrap();
    suite.borrow(borrower, 1000).unwrap();

    suite.assert_btoken_balance("borrower", 1000u128);

    // We're charging interest every quarter.
    // After three quarters pass, the result should be:
    // 1000 * 110% * 110% * 110% = 1331
    suite.advance_seconds(QUARTER * 3);
    suite.assert_btoken_balance("borrower", 1331u128);
}

#[test]
fn compounding_interest_charge_triggered_every_epoch() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(5000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_charge_period(QUARTER)
        .with_interest(40, 0) // 40% annual, 10% quarterly
        .with_reserve_factor(15)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    suite.deposit(lender, &[coin(2000, market_token)]).unwrap();
    suite.borrow(borrower, 1000).unwrap();

    suite.assert_btoken_balance("borrower", 1000u128);

    for _ in 0..3 {
        suite.advance_seconds(QUARTER);
        // Just to trigger an interest charge
        suite.deposit(lender, &[coin(2, market_token)]).unwrap();
    }

    // We're charging interest every quarter.
    // After three quarters pass, the result should be:
    // 1000 * 110% * 110% * 110% = 1331
    suite.assert_btoken_balance("borrower", 1331u128);
}

#[test]
fn query_last_charged_with_uncharged_interest() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(5000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_charge_period(YEAR)
        .with_interest(10, 0)
        .with_reserve_factor(15)
        .with_market_token(market_token)
        .with_pool(1, (coin(100, COMMON), coin(100, market_token)))
        .build();

    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    suite
        .deposit(lender, &[Coin::new(2000, market_token)])
        .unwrap();

    suite.borrow(borrower, 1000).unwrap();

    let next_epoch = suite.query_config().unwrap().last_charged + YEAR;

    suite.advance_seconds(YEAR + 123);

    // we want to make sure the query returns the timestamp as if interest was already charged for this epoch
    // even if there was no call to `charge_interest`

    assert_eq!(next_epoch, suite.query_config().unwrap().last_charged);
}
