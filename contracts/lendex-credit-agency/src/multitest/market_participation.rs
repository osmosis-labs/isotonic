use super::suite::SuiteBuilder;

// This test does not test full flow in which market would be entered on first market operation - it
// just tests if the market contract can properly introduce account to it.
#[test]
fn enter_market() {
    let gov = "gov";
    let market_token = "OSMO";
    let actor = "actor";

    let mut suite = SuiteBuilder::new().with_gov(gov).build();

    suite
        .create_market_quick(gov, "osmo", market_token, None, None, None)
        .unwrap();

    let market = suite.query_market(market_token).unwrap().market;

    suite.enter_market(market.as_str(), actor).unwrap();
}
