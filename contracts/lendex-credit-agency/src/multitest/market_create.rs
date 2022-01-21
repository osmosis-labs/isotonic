use super::suite::SuiteBuilder;

use cosmwasm_std::Decimal;

#[test]
fn market_create() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite
        .create_market_quick("gov", "osmo", "OSMO", None, None)
        .unwrap();
    suite.assert_market("OSMO");
}

#[test]
fn market_create_multiple() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite
        .create_market_quick("gov", "osmo", "OSMO", None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "btc", "BTC", None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "atom", "ATOM", None, None)
        .unwrap();

    suite.assert_market("OSMO");
    suite.assert_market("BTC");
    suite.assert_market("ATOM");
}

#[test]
fn market_create_unauthorized() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    assert_eq!(
        suite
            .create_market_quick("random_dude", "osmo", "OSMO", None, None)
            .unwrap_err()
            .to_string(),
        "Unauthorized"
    );
}

#[test]
fn market_create_already_exists() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite
        .create_market_quick("gov", "osmo", "OSMO", None, None)
        .unwrap();
    let err = suite
        .create_market_quick("gov", "osmo", "OSMO", None, None)
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "A market for base asset OSMO already exists"
    );
}

#[test]
fn collateral_ratio_higher_then_liquidation_price() {
    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_liquidation_price(Decimal::percent(92))
        .build();

    assert_eq!(
        suite
            .create_market_quick("gov", "osmo", "OSMO", Decimal::percent(92), None)
            .unwrap_err()
            .to_string(),
        "Creating Market failure - collateral ratio must be lower than liquidation price"
    );

    assert_eq!(
        suite
            .create_market_quick("gov", "osmo", "OSMO", Decimal::percent(93), None)
            .unwrap_err()
            .to_string(),
        "Creating Market failure - collateral ratio must be lower than liquidation price"
    );
}
