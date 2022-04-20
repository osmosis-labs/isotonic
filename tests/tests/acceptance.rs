//! https://confio.slab.com/posts/acceptance-tests-q7uuk5oi

use cosmwasm_std::{coin, Decimal, Uint128};

use tests::{MarketBuilder, SuiteBuilder};
use utils::{credit_line::CreditLineValues, token::Token};

#[test]
#[ignore]
fn paying_back_loan_using_collateral() {
    let mut suite = SuiteBuilder::new()
        .with_market(MarketBuilder::new("A").with_collateral_ratio(Decimal::percent(65)))
        .with_funds("alice", &[coin(100_000_000, "A")])
        .with_common_token("C")
        .with_pool(1, (coin(100_000_000, "A"), coin(100_000_000, "C")))
        .build();

    suite.deposit("alice", coin(100_000_000, "A")).unwrap();

    suite.borrow("alice", coin(65_000_000, "A")).unwrap();
    suite.deposit("alice", coin(65_000_000, "A")).unwrap();

    suite.borrow("alice", coin(42_250_000, "A")).unwrap();
    suite.deposit("alice", coin(42_250_000, "A")).unwrap();

    suite.borrow("alice", coin(27_462_500, "A")).unwrap();
    suite.burn("alice", coin(27_462_500, "A")).unwrap(); // Alice buys coffee ;)

    assert_eq!(
        suite
            .query_total_credit_line("alice")
            .unwrap()
            .validate(&Token::new_native("C"))
            .unwrap(),
        CreditLineValues {
            collateral: Uint128::new(207_250_000),
            credit_line: Uint128::new(134_712_500),
            debt: Uint128::new(134_712_500),
        }
    );
    assert_eq!(suite.query_balance("alice", "A").unwrap(), 0);

    suite
        .repay_with_collateral("alice", coin(207_250_000, "A"), coin(133_365_375, "A"))
        .unwrap();
    suite.reset_pools().unwrap();

    assert_eq!(
        suite
            .query_total_credit_line("alice")
            .unwrap()
            .validate(&Token::new_native("C"))
            .unwrap(),
        CreditLineValues {
            collateral: Uint128::new(73_884_625),
            credit_line: Uint128::new(48_025_006),
            debt: Uint128::new(1_347_125),
        }
    );
    assert_eq!(suite.query_balance("alice", "A").unwrap(), 0);
}
