use cosmwasm_std::{coin, Decimal};

use tests::{MarketBuilder, SuiteBuilder};

// regression: #40
#[test]
fn withdraw_all_with_matching_collateral() {
    let mut suite = SuiteBuilder::new()
        .with_funds("alice", &[coin(1_000_000, "ATOM")])
        .with_common_token("OSMO")
        .with_pool(1, (coin(1_000_000, "OSMO"), coin(1_000_000, "ATOM")))
        .with_market(MarketBuilder::new("ATOM").with_collateral_ratio(Decimal::percent(60)))
        .build();

    suite.deposit("alice", coin(100, "ATOM")).unwrap();

    suite.assert_withdrawable("alice", coin(100, "ATOM"));
    suite.attempt_withdraw_max("alice", "ATOM").unwrap();
}
