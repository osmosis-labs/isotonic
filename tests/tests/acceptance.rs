//! https://confio.slab.com/posts/acceptance-tests-q7uuk5oi
//!
//! These tests assume that 1 unit of a token is represented by 1_000_000.
//! This way we can verify the fractions mentioned in the slab document.
//!
//! User A = Alice
//! User B = Bob
//! User C = Carol

use cosmwasm_std::{coin, Decimal, Uint128};

use tests::{MarketBuilder, SuiteBuilder};
use utils::{coin::coin_native, credit_line::CreditLineValues, token::Token};

#[test]
#[ignore]
fn liquidate_via_amm() {
    let alice = "alice";
    let bob = "bob";
    let carol = "carol";

    let mut suite = SuiteBuilder::new()
        .with_market(MarketBuilder::new("A").with_collateral_ratio(Decimal::percent(65)))
        .with_market(MarketBuilder::new("B").with_collateral_ratio(Decimal::percent(65)))
        .with_funds(alice, &[coin(100_000_000_000, "A")])
        .with_funds(
            bob,
            &[coin(100_000_000_000, "A"), coin(100_000_000_000, "B")],
        )
        .with_common_token("C")
        .with_pool(1, (coin(100_000_000_000, "A"), coin(100_000_000_000, "C")))
        .with_pool(2, (coin(100_000_000_000, "B"), coin(100_000_000_000, "C")))
        .build();

    suite.deposit(alice, coin(100_000_000, "A")).unwrap();
    suite.deposit(bob, coin(100_000_000, "B")).unwrap();

    suite.borrow(alice, coin(65_000_000, "B")).unwrap(); // max loan

    // Bob is in the green, can't be liquidated yet
    suite
        .liquidate(carol, alice, "A", coin(65_000_000, "B"))
        .unwrap_err();

    // Put Bob under water, prime for liquidation
    suite
        .swap_exact_in(bob, coin(1_000_000_000, "A"), "B")
        .unwrap();
    assert_eq!(
        suite
            .query_total_credit_line(alice)
            .unwrap()
            .validate(&Token::new_native("C"))
            .unwrap(),
        CreditLineValues {
            collateral: Uint128::new(98_032_000),
            credit_line: Uint128::new(63_720_000),
            debt: Uint128::new(66_287_000),
        }
    );

    suite
        .liquidate(carol, alice, "A", coin(65_000, "B"))
        .unwrap();

    // Carol earns a "trigger fee" from liquidation.
    assert_eq!(
        suite
            .query_tokens_balance("B", carol)
            .unwrap()
            .ltokens
            .u128(),
        325_000
    );

    // Bob earns interest from liquidation since he's a B token lender.
    assert_eq!(
        suite.query_tokens_balance("B", bob).unwrap().ltokens.u128(),
        100_000_000 + 2_925_000
    );

    // Alice sold some of her initial deposit to pay the debt.
    assert!(
        suite
            .query_tokens_balance("A", carol)
            .unwrap()
            .ltokens
            .u128()
            < 100_000_000
    );
}

#[test]
#[ignore]
fn paying_back_loan_using_collateral_one_market() {
    let alice = "alice";

    let mut suite = SuiteBuilder::new()
        .with_market(MarketBuilder::new("A").with_collateral_ratio(Decimal::percent(65)))
        .with_funds(alice, &[coin(100_000_000, "A")])
        .with_common_token("C")
        .with_pool(1, (coin(100_000_000_000, "A"), coin(100_000_000_000, "C")))
        .build();

    suite.deposit(alice, coin(100_000_000, "A")).unwrap();

    suite.borrow(alice, coin(65_000_000, "A")).unwrap();
    suite.deposit(alice, coin(65_000_000, "A")).unwrap();

    suite.borrow(alice, coin(42_250_000, "A")).unwrap();
    suite.deposit(alice, coin(42_250_000, "A")).unwrap();

    suite.borrow(alice, coin(27_462_500, "A")).unwrap();
    suite.burn(alice, coin(27_462_500, "A")).unwrap(); // Alice buys coffee ;)

    assert_eq!(
        suite
            .query_total_credit_line(alice)
            .unwrap()
            .validate(&Token::new_native("C"))
            .unwrap(),
        CreditLineValues {
            collateral: Uint128::new(207_250_000),
            credit_line: Uint128::new(134_712_500),
            debt: Uint128::new(134_712_500),
        }
    );
    assert_eq!(suite.query_balance(alice, "A").unwrap(), 0);

    suite
        .repay_with_collateral(
            alice,
            coin_native(204_555_750, "A"),
            coin_native(133_365_375, "A"),
        )
        .unwrap();
    suite.reset_pools().unwrap();

    assert_eq!(
        suite
            .query_total_credit_line(alice)
            .unwrap()
            .validate(&Token::new_native("C"))
            .unwrap(),
        CreditLineValues {
            collateral: Uint128::new(73_884_625),
            credit_line: Uint128::new(48_025_006),
            debt: Uint128::new(1_347_125),
        }
    );
    assert_eq!(suite.query_balance(alice, "A").unwrap(), 0);
}
