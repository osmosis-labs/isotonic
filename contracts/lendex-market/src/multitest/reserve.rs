use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, Coin, Uint128};
use lendex_token::DisplayAmount;

use crate::state::SECONDS_IN_YEAR;

#[test]
fn after_full_year() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(4000, market_token)])
        .with_funds(borrower, &[coin(2300, market_token)])
        .with_interest(4, 20)
        .with_reserve_factor(10)
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
    // interests are 20% (4% base + 20% slope * 80% utilization)
    // supplied (ltokens) = 2000
    // borrwed (btokens) = 1600
    // bMul (btoken_ratio) = 20% after full year
    // charged interests = 20% * 1600 = 320
    // reserve = 10% * charged interests = 32
    // liquid assets = 400
    // ltokens supplied = 1600 + 400 - 32 = 1968
    // lMul (ltoken_ratio) = borrowed * bMul / lMul = 1600 * 0.2 / 1968 ~= 0.162
    // that means ltokens 2000 * 1.163 = 2324
    // deposit 1000 -> 3324 left btokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    // TODO: rounding error
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(3323u128)
    );

    // TODO: Rounding error
    assert_eq!(suite.query_reserve().unwrap(), Uint128::new(31));
}

#[test]
fn after_half_year() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(5000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_interest(4, 20)
        .with_reserve_factor(20)
        .with_market_token(market_token)
        .build();

    // Set arbitrary market/common exchange ratio and credit lines (not part of this test)
    suite.set_token_ratio_one().unwrap();
    suite.set_high_credit_line(borrower).unwrap();
    suite.set_high_credit_line(lender).unwrap();

    // Deposit some tokens
    suite
        .deposit(lender, &[Coin::new(4000, market_token)])
        .unwrap();

    // Borrow some tokens
    suite.borrow(borrower, 3000).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 2) as u64);

    // Deposit some tokens
    // interests are 19% (4% base + 20% slope * 75% utilization)
    // supplied (ltokens) = 4000
    // borrwed (btokens) = 3000
    // bMul (btoken_ratio) = 9.5% after half year
    // charged interests = 9.5% * 3000 = 285
    // reserve = 20% * charged interests = 57
    // liquid assets = 1000
    // ltokens supplied = 3000 + 1000 - 57 = 3943
    // lMul (ltoken_ratio) = borrowed * bMul / lMul = 3000 * 0.095 / 3400 ~= 0.072
    // that means ltokens 4000 * 1.072 = 4288
    // deposit 1000 -> 5288 left btokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    // TODO: rounding error
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(5288u128)
    );

    assert_eq!(suite.query_reserve().unwrap(), Uint128::new(57));
}

#[test]
fn charged_couple_times() {
    let lender = "lender";
    let borrower = "borrower";
    let market_token = "atom";
    let mut suite = SuiteBuilder::new()
        .with_funds(lender, &[coin(5000, market_token)])
        .with_funds(borrower, &[coin(500, market_token)])
        .with_interest(4, 20)
        .with_reserve_factor(15)
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
    suite.borrow(borrower, 1200).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 4) as u64);

    // Deposit some tokens
    // interests are 16%(4% base + 20% slope * 60% utilization)
    // supplied (ltokens) = 2000
    // borrwed (btokens) = 1200
    // bMul (btoken_ratio) = 4% after 3 months
    // charged interests = 4% * 1200 = 48
    // reserve = 15% * charged interests = 7.2 ~= 7
    // liquid assets = 800
    // ltokens supplied = 1200 + 800 - 7 = 1993
    // lMul (ltoken_ratio) = borrowed * bMul / lMul = 1200 * 0.04 / 1993 ~= 0.024
    // that means ltokens 2000 * 1.024 = 2048
    // deposit 1000 -> 3048 left btokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    // TODO: rounding error
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(3047u128)
    );

    assert_eq!(suite.query_reserve().unwrap(), Uint128::new(7));

    suite.borrow(borrower, 800).unwrap();

    suite.advance_seconds((SECONDS_IN_YEAR / 3) as u64);

    // Deposit some tokens
    // interests are 17.4%(4% base + 20% slope * 67% utilization)
    // supplied (ltokens) = 3047
    // borrwed (btokens) = 2047 (1200 * 1.04 + 1000)
    // bMul (btoken_ratio) = 5.8% after 7 months
    // charged interets = 5.8% * 2047 = 118.7 ~= 118
    // reserve = 15% * charged interests = 7 + (15% * 118) = 24
    // liquid assets =  3047 + 7 (old reserve) - 2047 (borrowed) = 1007
    // ltokens supplied = 2047 + 1007 - 24 = 3030
    // lMul (ltoken_ratio) = borrowed * bMul / lMul = 2047 * 0.058 / 3030 ~= 0.039
    // that means ltokens 3047 * 1.039 = 3165
    // deposit 1000 -> 4165 left btokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(4165u128)
    );

    assert_eq!(suite.query_reserve().unwrap(), Uint128::new(24));
}
