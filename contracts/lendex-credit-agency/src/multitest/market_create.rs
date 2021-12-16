use super::suite::SuiteBuilder;

#[test]
fn market_create() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite.create_market_quick("gov", "osmo", "OSMO").unwrap();
    suite.assert_market("OSMO");
}

#[test]
fn market_create_multiple() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite.create_market_quick("gov", "osmo", "OSMO").unwrap();
    suite.create_market_quick("gov", "btc", "BTC").unwrap();
    suite.create_market_quick("gov", "atom", "ATOM").unwrap();

    suite.assert_market("OSMO");
    suite.assert_market("BTC");
    suite.assert_market("ATOM");
}

#[test]
fn market_create_unauthorized() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    assert_eq!(
        suite
            .create_market_quick("random_dude", "osmo", "OSMO")
            .unwrap_err()
            .to_string(),
        "Unauthorized"
    );
}

#[test]
#[ignore]
fn market_create_already_exists() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite.create_market_quick("gov", "osmo", "OSMO").unwrap();
    assert_eq!(
        suite
            .create_market_quick("random_dude", "osmo", "OSMO")
            .unwrap_err()
            .to_string(),
        "A market for base asset OSMO already exists"
    );
}
