use super::suite::{SuiteBuilder, COMMON};
use crate::error::ContractError;

use cosmwasm_std::{coin, coins, Decimal, Uint128};
use utils::credit_line::{CreditLineResponse, CreditLineValues};
use utils::token::Token;

const YEAR_IN_SECONDS: u64 = 365 * 24 * 3600;

#[test]
fn account_doesnt_have_debt_bigger_then_credit_line() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(debtor, &coins(500, denom))
        .with_pool(1, (coin(100, COMMON), coin(100, denom)))
        .build();

    suite
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80), None, None)
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(500, denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(500),
            credit_line: Uint128::new(400),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone())
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
        } if debt.amount == Uint128::new(400)));

    let err = suite
        .liquidate(
            liquidator,
            debtor,
            Token::Native(denom.into()),
            coin(400, denom),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::LiquidationNotAllowed {},
        err.downcast().unwrap()
    );
}

#[test]
fn liquidating_whole_debt() {
    let debtor = "debtor";
    let liquidator = "liquidator";
    let some_investor = "investor";

    let osmo = "OSMO";
    let atom = "ATOM";
    let juno = "JUNO";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_common_token(osmo)
        .with_liquidation_fee(Decimal::percent(5))
        .with_liquidation_initiation_fee(Decimal::percent(1))
        .with_funds(debtor, &coins(580, atom))
        .with_funds(some_investor, &coins(500, juno))
        .with_pool(
            1,
            (coin(100_000_000_000, osmo), coin(100_000_000_000, atom)),
        )
        .with_pool(2, (coin(80_000_000_000, osmo), coin(100_000_000_000, juno)))
        .build();

    suite
        .create_market_quick("gov", "atom", atom, Decimal::percent(70), None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "juno", juno, Decimal::percent(70), None, None)
        .unwrap();

    // This is just to make sure the JUNO market has enough liquidity to cover the loan below.
    suite
        .deposit_tokens_on_market(some_investor, coin(500, juno))
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(580, atom))
        .unwrap();
    suite
        .borrow_tokens_from_market(debtor, coin(500, juno))
        .unwrap();

    // Prices change. The debtor is now underwater.
    suite
        .set_pool(&[(
            2,
            (coin(100_000_000_000, osmo), coin(100_000_000_000, juno)),
        )])
        .unwrap();

    suite
        .liquidate(
            liquidator,
            debtor,
            Token::Native(atom.into()),
            coin(500, juno),
        )
        .unwrap();

    // reset pools
    suite
        .set_pool(&[(
            1,
            (coin(100_000_000_000, osmo), coin(100_000_000_000, atom)),
        )])
        .unwrap();
    suite
        .set_pool(&[(
            2,
            (coin(100_000_000_000, osmo), coin(100_000_000_000, juno)),
        )])
        .unwrap();

    let liquidation_price = 500 // actual debt worth with 1:1 liquidity pools
        + 25 // 5% liquidation fees
        + 5  // 1% liquidation initiation fee
        + 4; // 3% swap fees, paid twice (swap through two pools), rounded up
    let crl = suite
        .query_total_credit_line(debtor)
        .unwrap()
        .validate(&Token::Native(osmo.to_string()))
        .unwrap();
    assert_eq!(crl.collateral, Uint128::new(580 - liquidation_price));
    assert!(crl.debt.is_zero());
}

/// Prerequisite: https://github.com/confio/isotonic/issues/142
#[test]
#[ignore]
fn liquidating_whole_debt_collateral_and_debt_in_same_token() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let osmo = "OSMO";

    // TODO:

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(5000, osmo))
        .with_funds(debtor, &coins(600, osmo))
        .with_liquidation_fee(Decimal::percent(92))
        .with_pool(1, (coin(10_000, COMMON), coin(10_000, osmo)))
        .build();

    suite
        .create_market_quick("gov", "osmo", osmo, Decimal::percent(80), None, None)
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(500, osmo))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(500),
            credit_line: Uint128::new(400),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone())
    );

    // debt must be higher then credit line, but debtor can borrow at most 400 tokens
    suite
        .borrow_tokens_from_market(debtor, coin(400, osmo))
        .unwrap();
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert!(matches!(
        total_credit_line,
        CreditLineResponse {
            debt,
            ..
        } if debt.amount == Uint128::new(400)));

    suite.advance_seconds(YEAR_IN_SECONDS);

    // Repay some tokens to trigger interest rate charges
    suite.repay_tokens_on_market(debtor, coin(2, osmo)).unwrap();

    // utilisation is 80% (400/500)
    // default interest rates are 3% with 20% slope which gives 3% + 20% * 80% = 19%
    // after a year debt increases to 473.63 tokens
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(576),
            credit_line: Uint128::new(460),
            debt: Uint128::new(474)
        }
        .make_response(suite.common_token().clone())
    );

    suite
        .liquidate(
            liquidator,
            debtor,
            Token::Native(osmo.into()),
            coin(474, osmo),
        )
        .unwrap();

    // Liquidation price is 0.92
    // Repaid value is 474 * 1.0 (oracle's price for same denom) * 0.92 = 515.22
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 575 - 515 = 60
            collateral: Uint128::new(61),
            credit_line: Uint128::new(48),
            debt: Uint128::new(0)
        }
        .make_response(suite.common_token().clone())
    );

    let total_credit_line = suite.query_total_credit_line(liquidator).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 515 tokens transferred as reward from debtor
            collateral: Uint128::new(514), // FIXME: Rounding issue? Message debug shows 515 transferred
            credit_line: Uint128::new(411),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token().clone())
    );
}

/// Prerequisite: https://github.com/confio/isotonic/issues/142
#[test]
#[ignore]
fn liquidate_when_debt_is_in_common_token() {
    todo!()
}

/// Prerequisite: https://github.com/confio/isotonic/issues/142
#[test]
#[ignore]
fn liquidate_when_collateral_is_in_common_token() {
    todo!()
}

#[test]
fn liquidation_fails_if_no_collateral_market() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";
    let reward_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(5000, denom))
        .with_funds(debtor, &coins(600, denom))
        .with_liquidation_fee(Decimal::percent(92))
        .with_pool(1, (coin(100, COMMON), coin(100, denom)))
        .with_pool(2, (coin(100, COMMON), coin(150, reward_denom)))
        .build();

    suite
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80), None, None)
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(500, denom))
        .unwrap();

    suite
        .borrow_tokens_from_market(debtor, coin(400, denom))
        .unwrap();

    suite.advance_seconds(YEAR_IN_SECONDS);

    // Repay some tokens to trigger interest rate charges
    suite
        .repay_tokens_on_market(debtor, coin(2, denom))
        .unwrap();

    let err = suite
        .liquidate(
            liquidator,
            debtor,
            Token::Native(reward_denom.to_owned()),
            coin(474, denom),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::NoMarket(reward_denom.to_owned()),
        err.downcast().unwrap()
    );
}

#[test]
fn receive_reward_fails_when_insufficient_collateral() {
    let debtor = "debtor";
    let liquidator = "liquidator";
    let some_investor = "investor";

    let osmo = "OSMO";
    let atom = "ATOM";
    let juno = "JUNO";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_common_token(osmo)
        .with_liquidation_fee(Decimal::percent(5))
        .with_liquidation_initiation_fee(Decimal::percent(1))
        .with_funds(debtor, &coins(580, atom))
        .with_funds(some_investor, &coins(500, juno))
        .with_pool(
            1,
            (coin(100_000_000_000, osmo), coin(100_000_000_000, atom)),
        )
        .with_pool(2, (coin(80_000_000_000, osmo), coin(100_000_000_000, juno)))
        .build();

    suite
        .create_market_quick("gov", "atom", atom, Decimal::percent(70), None, None)
        .unwrap();
    suite
        .create_market_quick("gov", "juno", juno, Decimal::percent(70), None, None)
        .unwrap();

    // This is just to make sure the JUNO market has enough liquidity to cover the loan below.
    suite
        .deposit_tokens_on_market(some_investor, coin(500, juno))
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(580, atom))
        .unwrap();
    suite
        .borrow_tokens_from_market(debtor, coin(500, juno))
        .unwrap();

    // Prices change. The debtor is now seriously underwater.
    suite
        .set_pool(&[(
            2,
            (coin(200_000_000_000, osmo), coin(100_000_000_000, juno)),
        )])
        .unwrap();

    suite
        .liquidate(
            liquidator,
            debtor,
            Token::Native(atom.into()),
            coin(500, juno),
        )
        .unwrap_err();
}
