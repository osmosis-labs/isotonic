use super::suite::SuiteBuilder;

use lendex_market::msg::CreditLineResponse;

use cosmwasm_std::{coin, coins, Decimal, Uint128};

#[test]
fn send_more_then_one_denom() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";
    let other_denom = "otherOSMO";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &[coin(5000, denom), coin(500, other_denom)])
        .with_liquidation_price(Decimal::percent(92))
        .build();

    suite
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80))
        .unwrap();

    let err = suite
        .liquidate(
            liquidator,
            debtor,
            &[coin(100, denom), coin(100, other_denom)],
        )
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Only one denom can be sent for liquidation"
    );
}

#[test]
fn account_doesnt_have_debt_bigger_then_credit_line() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(5000, denom))
        .with_funds(debtor, &coins(500, denom))
        .with_liquidation_price(Decimal::percent(92))
        .build();

    suite
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80))
        .unwrap();

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(500, denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(500),
            credit_line: Uint128::new(400),
            debt: Uint128::zero()
        }
    );

    // debt must be higher then credit line, so 400 debt with 400 credit line won't allow liquidation
    suite
        .borrow_tokens_from_market(debtor, coin(400, denom))
        .unwrap();
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert!(matches!(
        total_credit_line,
        CreditLineResponse {
            debt,
            ..
        } if debt == Uint128::new(400)));

    let err = suite
        .liquidate(liquidator, debtor, &coins(400, denom))
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Account cannot be liquidated as it does not have more debt then credit line"
    );
}

#[test]
fn liquidating_whole_debt() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(5000, denom))
        .with_funds(debtor, &coins(600, denom))
        .with_liquidation_price(Decimal::percent(92))
        .build();

    suite
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80))
        .unwrap();

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(500, denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(500),
            credit_line: Uint128::new(400),
            debt: Uint128::zero()
        }
    );

    // debt must be higher then credit line, so 400 debt with 400 credit line won't allow liquidation
    suite
        .borrow_tokens_from_market(debtor, coin(400, denom))
        .unwrap();
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert!(matches!(
        total_credit_line,
        CreditLineResponse {
            debt,
            ..
        } if debt == Uint128::new(400)));

    suite.advance_seconds(31_556_736);

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();

    suite
        .repay_tokens_on_market(debtor, coin(2, denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(575),
            credit_line: Uint128::new(460),
            debt: Uint128::new(474)
        }
    );

    suite
        .liquidate(liquidator, debtor, &coins(400, denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(141),
            credit_line: Uint128::new(112),
            debt: Uint128::new(74)
        }
    );

    let total_credit_line = suite.query_total_credit_line(liquidator).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(433),
            credit_line: Uint128::new(346),
            debt: Uint128::zero()
        }
    );

}
