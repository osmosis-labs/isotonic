use super::suite::SuiteBuilder;

use cosmwasm_std::{coin, Decimal};

#[test]
fn liquidation_price_lower_then_collateral_ratio() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(debtor, &[coin(5000, denom)])
        .with_funds(liquidator, &[coin(5000, denom)])
        .with_liquidation_price(Decimal::percent(90))
        .build();

    suite
        .create_market_quick("gov", "osmo", denom, Decimal::percent(95))
        .unwrap();

    let err = suite
        .liquidate(liquidator, debtor, coin(100, denom))
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Cannot liquidate debt if collateral ratio is higher then liquidation price"
    );
}
