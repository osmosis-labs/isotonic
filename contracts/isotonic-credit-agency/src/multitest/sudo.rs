use super::suite::{contract_market, SuiteBuilder};
use crate::error::ContractError;

use isotonic_market::msg::MigrateMsg as MarketMigrateMsg;

#[test]
fn adjust_market_id() {
    let mut suite = SuiteBuilder::new().build();

    suite.sudo_adjust_market_id(30).unwrap();
    assert_eq!(30, suite.query_config().unwrap().isotonic_market_id);
}

#[test]
fn adjust_token_id() {
    let mut suite = SuiteBuilder::new().build();

    suite.sudo_adjust_token_id(30).unwrap();
    assert_eq!(30, suite.query_config().unwrap().isotonic_token_id);
}

#[test]
fn adjust_common_token() {
    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_common_token("common")
        .build();

    suite
        .create_market_quick("gov", "osmo", "OSMO", None, None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "btc", "BTC", None, None, None)
        .unwrap();

    let new_common = "new";
    suite.sudo_adjust_common_token(new_common).unwrap();
    assert_eq!(new_common, suite.query_config().unwrap().common_token);
    assert_eq!(
        new_common,
        suite.query_market_config("OSMO").unwrap().common_token
    );
    assert_eq!(
        new_common,
        suite.query_market_config("BTC").unwrap().common_token
    );
}

#[test]
fn migrate_market() {
    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_common_token("common")
        .build();

    suite
        .create_market_quick("gov", "osmo", "OSMO", None, None, None)
        .unwrap();

    assert_eq!(2, suite.query_contract_code_id("OSMO").unwrap());
    assert_eq!(3, suite.query_market_config("OSMO").unwrap().token_id);

    let new_market_id = suite.app().store_code(contract_market());
    assert_ne!(new_market_id, suite.query_contract_code_id("OSMO").unwrap());

    suite.sudo_adjust_market_id(new_market_id).unwrap();

    let market = suite.query_market("OSMO").unwrap();
    suite
        .sudo_migrate_market(
            market.market.as_str(),
            MarketMigrateMsg {
                isotonic_token_id: Some(50),
            },
        )
        .unwrap();

    assert_eq!(new_market_id, suite.query_contract_code_id("OSMO").unwrap());
    assert_eq!(50, suite.query_market_config("OSMO").unwrap().token_id);
}

#[test]
fn migrate_non_existing_market() {
    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_common_token("common")
        .build();

    let err = suite
        .sudo_migrate_market(
            "OSMO",
            MarketMigrateMsg {
                isotonic_token_id: None,
            },
        )
        .unwrap_err();

    assert_eq!(
        ContractError::MarketSearchError {
            market: "OSMO".to_owned(),
        },
        err.downcast().unwrap()
    );
}
