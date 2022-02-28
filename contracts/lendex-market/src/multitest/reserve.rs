use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, Coin, Uint128};
use lendex_token::DisplayAmount;

use crate::state::SECONDS_IN_YEAR;

#[test]
fn reserve_factor_after_full_year() {
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
    // reserve = 10% * borrowed = 160
    // bMul (btoken_ratio) = 20% after full year
    // liquid assets = 400
    // ltokens supplied = 2000 + 400 - 160 = 2240
    // lMul (ltoken_ratio) = borrowed * bMul / lMul = 1600 * 0.2 / 2240 ~= 0.174
    // that means ltokens 2000 * 1.174 = 2348
    // deposit 1000 -> 3348 left btokens
    suite
        .deposit(lender, &[Coin::new(1000, market_token)])
        .unwrap();

    // TODO: rounding error
    assert_eq!(
        suite.query_ltoken_info().unwrap().total_supply,
        DisplayAmount::raw(3346u128)
    );

    assert_eq!(suite.query_reserve().unwrap(), Uint128::new(160));
}
