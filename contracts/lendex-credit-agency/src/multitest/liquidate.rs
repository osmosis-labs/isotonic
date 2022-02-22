use super::suite::SuiteBuilder;
use crate::error::ContractError;

use lendex_token::error::ContractError as TokenContractError;

use cosmwasm_std::{coin, coins, Decimal, Uint128};
use utils::credit_line::{CreditLineResponse, CreditLineValues};

const YEAR_IN_SECONDS: u64 = 31_556_736;

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
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80), None)
        .unwrap();

    let err = suite
        .liquidate(
            liquidator,
            debtor,
            &[coin(100, denom), coin(100, other_denom)],
            denom.to_owned(),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::LiquidationOnlyOneDenomRequired {},
        err.downcast().unwrap(),
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
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80), None)
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
        CreditLineValues {
            collateral: Uint128::new(500),
            credit_line: Uint128::new(400),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token())
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
        .liquidate(liquidator, debtor, &coins(400, denom), denom.to_owned())
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

    let denom = "OSMO";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(5000, denom))
        .with_funds(debtor, &coins(600, denom))
        .with_liquidation_price(Decimal::percent(92))
        .build();

    suite
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80), None)
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
        CreditLineValues {
            collateral: Uint128::new(500),
            credit_line: Uint128::new(400),
            debt: Uint128::zero()
        }
        .make_response(suite.common_token())
    );

    // debt must be higher then credit line, but debtor can borrow at most 400 tokens
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

    suite.advance_seconds(YEAR_IN_SECONDS);

    // Repay some tokens to trigger interest rate charges
    suite
        .repay_tokens_on_market(debtor, coin(2, denom))
        .unwrap();

    // utilisation is 80% (400/500)
    // default interest rates are 3% with 20% slope which gives 3% + 20% * 80% = 19%
    // after a year debt increases to 473.63 tokens
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(575),
            credit_line: Uint128::new(460),
            debt: Uint128::new(474)
        }
        .make_response(suite.common_token())
    );

    suite
        .liquidate(liquidator, debtor, &coins(474, denom), denom.to_owned())
        .unwrap();

    // Liquidation price is 0.92
    // Repaid value is 474 * 1.0 (oracle's price for same denom) * 0.92 = 515.22
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 575 - 515 = 60
            collateral: Uint128::new(60),
            credit_line: Uint128::new(48),
            debt: Uint128::new(1) // FIXME: Rounding issue
        }
        .make_response(suite.common_token())
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
        .make_response(suite.common_token())
    );
}

#[test]
fn receive_reward_in_different_denom_fails_if_theres_no_reward_market() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";
    let reward_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(5000, denom))
        .with_funds(debtor, &coins(600, denom))
        .with_liquidation_price(Decimal::percent(92))
        .build();

    suite
        .create_market_quick("gov", "osmo", denom, Decimal::percent(80), None)
        .unwrap();

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(500, denom))
        .unwrap();

    suite
        .borrow_tokens_from_market(debtor, coin(400, denom))
        .unwrap();

    suite.advance_seconds(YEAR_IN_SECONDS);

    suite
        .oracle_set_price_market_per_common(reward_denom, Decimal::percent(150))
        .unwrap();

    // Repay some tokens to trigger interest rate charges
    suite
        .repay_tokens_on_market(debtor, coin(2, denom))
        .unwrap();

    let err = suite
        .liquidate(
            liquidator,
            debtor,
            &coins(474, denom),
            reward_denom.to_owned(),
        )
        .unwrap_err();
    assert_eq!(
        ContractError::NoMarket(reward_denom.to_owned()),
        err.downcast().unwrap()
    );
}

#[test]
fn receive_reward_different_denom_fails_if_debtor_has_not_enough_reward_tokens() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";
    let reward_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(5000, denom))
        .with_funds(debtor, &[coin(600, denom), coin(500, reward_denom)])
        .with_liquidation_price(Decimal::percent(92))
        .build();

    // create market with very high interest rates
    suite
        .create_market_quick(
            "gov",
            "osmo",
            denom,
            Decimal::percent(80),
            (Decimal::percent(80), Decimal::percent(45)),
        )
        .unwrap();
    // create reward_denom market
    suite
        .create_market_quick("gov", "eth", reward_denom, Decimal::percent(80), None)
        .unwrap();

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();
    suite
        .oracle_set_price_market_per_common(reward_denom, Decimal::percent(25))
        .unwrap();

    suite
        .deposit_tokens_on_market(debtor, coin(500, denom))
        .unwrap();
    suite
        .borrow_tokens_from_market(debtor, coin(400, denom))
        .unwrap();

    suite.advance_seconds(YEAR_IN_SECONDS);

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();
    suite
        .oracle_set_price_market_per_common(reward_denom, Decimal::percent(150))
        .unwrap();

    // Repay some tokens to trigger interest rate charges
    suite
        .repay_tokens_on_market(debtor, coin(10, denom))
        .unwrap();

    // debtor deposits some tokens in reward_denom market
    suite
        .deposit_tokens_on_market(debtor, coin(50, reward_denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(1038),
            credit_line: Uint128::new(830),
            debt: Uint128::new(855)
        }
        .make_response(suite.common_token())
    );

    let err = suite
        .liquidate(
            liquidator,
            debtor,
            &coins(100, denom),
            reward_denom.to_owned(),
        )
        .unwrap_err();
    // Transferable amount is available balance / collateral ratio
    // balance = credit line - debt / price ratio = 830 - 755 (855 - 100 liquidated) / 1.5 = 50
    assert_eq!(
        TokenContractError::InsufficientTokens {
            available: Uint128::new(50),
            needed: Uint128::new(72)
        },
        err.downcast().unwrap()
    );
}

#[test]
fn receive_reward_in_different_denoms_no_interest_rates() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let atom = "OSMO";
    let ust = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(160000, ust))
        .with_funds(debtor, &coins(5000, atom))
        .with_liquidation_price(Decimal::percent(92))
        .build();

    // create market atom osmo
    suite
        .create_market_quick(
            "gov",
            "atom",
            atom,
            Decimal::percent(50),                        // collateral price
            (Decimal::percent(3), Decimal::percent(20)), // interest rates (base, slope)
        )
        .unwrap();
    // create ust market eth
    suite
        .create_market_quick(
            "gov",
            "ust",
            ust,
            Decimal::percent(60),                        // collateral price
            (Decimal::percent(3), Decimal::percent(20)), // interest rates (base, slope)
        )
        .unwrap();

    suite
        .oracle_set_price_market_per_common(atom, Decimal::percent(400))
        .unwrap();
    suite
        .oracle_set_price_market_per_common(ust, Decimal::percent(10))
        .unwrap();

    // debtor deposits 4000 ust
    suite
        .deposit_tokens_on_market(debtor, coin(4000, atom))
        .unwrap();
    // liquidator deposits 100000 ust
    suite
        .deposit_tokens_on_market(liquidator, coin(100000, ust))
        .unwrap();
    // debtor borrows 75_000 ust
    suite
        .borrow_tokens_from_market(debtor, coin(75000, ust))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(16000), // 4000 deposited * 4.0
            credit_line: Uint128::new(8000), // 16000 collateral * 0.5 collateral price
            debt: Uint128::new(7500)         // 75_000 * 0.1
        }
        .make_response(suite.common_token())
    );

    suite
        .oracle_set_price_market_per_common(atom, Decimal::percent(300))
        .unwrap();
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(12000), // 4000 deposited * 3.0
            credit_line: Uint128::new(6000), // 12000 collateral * 0.5 collateral price
            debt: Uint128::new(7500)         // 75_000 * 0.1
        }
        .make_response(suite.common_token())
    );

    // successful liquidation of 6000 tokens
    suite
        .liquidate(liquidator, debtor, &coins(60_000, ust), atom.to_owned())
        .unwrap();

    // Liquidation price is 0.92
    // Repaid value is 60_000 ust * 0.1 / 3.0 / 0.92 = 2000 / 0.92 ~= 1999 / 0.92 = 2173 LATOM
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // (4000 deposited - 2173 repaid) * 3.0 tokens price = 1827 * 3.0
            collateral: Uint128::new(5481),
            // 5481 * 0.5 collateral price
            credit_line: Uint128::new(2740),
            // 7500 - (60_000 * 0.1)
            debt: Uint128::new(1500),
        }
        .make_response(suite.common_token())
    );
    let balance = suite.query_tokens_balance(ust, debtor).unwrap();
    assert_eq!(balance.btokens, Uint128::new(15000)); // 1500 / 0.1 price
    let balance = suite.query_tokens_balance(atom, debtor).unwrap();
    assert_eq!(balance.ltokens, Uint128::new(1827)); // (4000 deposited - 2173 repaid)

    let total_credit_line = suite.query_total_credit_line(liquidator).unwrap();
    assert!(matches!(
        total_credit_line,
        CreditLineResponse {
            collateral,
            ..
        // deposited 100_000 * 0.1 + repaid 2173 * 3.0 (actually 2172 - FIXME rounding error)
        } if collateral.amount == Uint128::new(16_519)
    ));
    let balance = suite.query_tokens_balance(atom, liquidator).unwrap();
    assert_eq!(balance.ltokens, Uint128::new(2173)); // 2173 repaid
}

#[test]
fn receive_reward_in_different_denoms_with_six_months_interests() {
    let debtor = "debtor";
    let liquidator = "liquidator";
    let others = "others";

    let atom = "OSMO";
    let ust = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(others, &[coin(100_000, ust), coin(10_000, atom)])
        .with_funds(liquidator, &coins(100_000, ust))
        .with_funds(debtor, &coins(5000, atom))
        .with_liquidation_price(Decimal::percent(92))
        .build();

    // create market atom osmo
    suite
        .create_market_quick(
            "gov",
            "atom",
            atom,
            Decimal::percent(50),                        // collateral price
            (Decimal::percent(3), Decimal::percent(20)), // interest rates (base, slope)
        )
        .unwrap();
    // create ust market eth
    suite
        .create_market_quick(
            "gov",
            "ust",
            ust,
            Decimal::percent(60),                        // collateral price
            (Decimal::percent(3), Decimal::percent(20)), // interest rates (base, slope)
        )
        .unwrap();

    suite
        .oracle_set_price_market_per_common(atom, Decimal::percent(400))
        .unwrap();
    suite
        .oracle_set_price_market_per_common(ust, Decimal::percent(10))
        .unwrap();

    // investments from the others
    suite
        .deposit_tokens_on_market(others, coin(10000, atom))
        .unwrap();
    suite
        .deposit_tokens_on_market(others, coin(100_000, ust))
        .unwrap();
    suite
        .borrow_tokens_from_market(others, coin(2000, atom))
        .unwrap();
    suite
        .borrow_tokens_from_market(others, coin(20_000, ust))
        .unwrap();

    // debtor deposits 4000 atom
    suite
        .deposit_tokens_on_market(debtor, coin(4000, atom))
        .unwrap();
    // debtor borrows 75_000 ust
    suite
        .borrow_tokens_from_market(debtor, coin(75000, ust))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(16000), // 4000 deposited * 4.0
            credit_line: Uint128::new(8000), // 16000 collateral * 0.5 collateral price
            debt: Uint128::new(7500)         // 75_000 * 0.1
        }
        .make_response(suite.common_token())
    );

    suite.advance_seconds(YEAR_IN_SECONDS / 2);

    // change ATOM price to 3.0 per common denom
    suite
        .oracle_set_price_market_per_common(atom, Decimal::percent(300))
        .unwrap();

    // current interest rates
    // rates = (base + slope * utilization) / 2 (half year)
    // atom = (3% + 20% * (2000/(10_000 + 4000))) / 2 = (3% + 20% * 14.3%) / 2 = (3% + 2.8%) / 2 = 2.9% ~= 3%
    // ust = (3% + 20% * ((20_000 + 75_000)/100_000)) / 2 = (3% + 20% * 95%) / 2 = (3% + 19%) / 2 = 11%

    // expected numbers before liquidation
    // LATOM = 4000 + (2000 * 0.03 * 4000/14000) = 4017
    // BUST = 75_000 * 1.11 * 0.1 = 8325

    // successful liquidation of 6000 tokens
    suite
        .liquidate(liquidator, debtor, &coins(60_000, ust), atom.to_owned())
        .unwrap();

    // Liquidation price is 0.92
    // Repaid value is 60_000 ust * 0.1 / 3.0 / 0.92 = 2000 / 0.92 ~= 1999 / 0.92 = 2172 LATO

    let balance = suite.query_tokens_balance(ust, debtor).unwrap();
    // 75_000 * 1.11 (interests) - 60_000 (repaid) = 83250 - 60000
    assert_eq!(balance.btokens, Uint128::new(23250));
    let balance = suite.query_tokens_balance(atom, debtor).unwrap();
    // amount left after paying liquidation reward
    // 4017 - 2172 repaid = 1845 FIXME: rounding issue
    assert_eq!(balance.ltokens, Uint128::new(1843));

    let balance = suite.query_tokens_balance(atom, liquidator).unwrap();
    assert_eq!(balance.ltokens, Uint128::new(2172)); // repaid amount as reward

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 1843 * 3 = 5529
            collateral: Uint128::new(5529),
            // 5529 * 0.5 collateral price
            credit_line: Uint128::new(2764),
            // 8375 - (60_000 * 0.1)
            debt: Uint128::new(2325),
        }
        .make_response(suite.common_token())
    );
}
