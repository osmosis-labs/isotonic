use super::suite::SuiteBuilder;
use crate::error::ContractError;

#[test]
fn market_create() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite
        .create_market_quick("gov", "osmo", "OSMO", None, None, None)
        .unwrap();
    suite.assert_market("OSMO");
}

#[test]
fn market_create_multiple() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite
        .create_market_quick("gov", "osmo", "OSMO", None, None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "btc", "BTC", None, None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "atom", "ATOM", None, None, None)
        .unwrap();

    suite.assert_market("OSMO");
    suite.assert_market("BTC");
    suite.assert_market("ATOM");
}

#[test]
fn market_create_unauthorized() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    let err = suite
        .create_market_quick("random_dude", "osmo", "OSMO", None, None, None)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());
}

#[test]
fn market_create_already_exists() {
    let mut suite = SuiteBuilder::new().with_gov("gov").build();

    suite
        .create_market_quick("gov", "osmo", "OSMO", None, None, None)
        .unwrap();
    let err = suite
        .create_market_quick("gov", "osmo", "OSMO", None, None, None)
        .unwrap_err();
    assert_eq!(
        ContractError::MarketAlreadyExists("OSMO".to_owned()),
        err.downcast().unwrap()
    );
}
