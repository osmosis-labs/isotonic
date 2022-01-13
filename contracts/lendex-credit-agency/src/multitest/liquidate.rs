use super::suite::SuiteBuilder;

use lendex_market::msg::CreditLineResponse;

use cosmwasm_std::{coin, coins, Decimal, Uint128};

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
        .liquidate(liquidator, debtor, &coins(400, denom), denom.to_owned())
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
        CreditLineResponse {
            collateral: Uint128::new(500),
            credit_line: Uint128::new(400),
            debt: Uint128::zero()
        }
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
        } if debt == Uint128::new(400)));

    suite.advance_seconds(YEAR_IN_SECONDS);

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();

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
        CreditLineResponse {
            collateral: Uint128::new(575),
            credit_line: Uint128::new(460),
            debt: Uint128::new(474)
        }
    );

    suite
        .liquidate(liquidator, debtor, &coins(474, denom), denom.to_owned())
        .unwrap();

    // Liquidation price is 0.92
    // Repaid value is 474 * 1.0 (oracle's price for same denom) * 0.92 = 515.22
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            // 575 - 515 = 60
            collateral: Uint128::new(60),
            credit_line: Uint128::new(48),
            debt: Uint128::new(1) // FIXME: Rounding issue
        }
    );

    let total_credit_line = suite.query_total_credit_line(liquidator).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            // 515 tokens transferred as reward from debtor
            collateral: Uint128::new(514), // FIXME: Rounding issue? Message debug shows 515 transferred
            credit_line: Uint128::new(411),
            debt: Uint128::zero()
        }
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
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();
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
        err.to_string(),
        "No market set up for base asset ".to_owned() + reward_denom
    );
}

#[test]
fn receive_rewarddifferent_denom_fails_if_debtor_has_not_enough_reward_tokens() {
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
        CreditLineResponse {
            collateral: Uint128::new(1038),
            credit_line: Uint128::new(830),
            debt: Uint128::new(855)
        }
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
    // transferable amount = balance / collateral ratio = 49 (rounding issue?) / 0.8 = 61.25
    assert_eq!(
        err.to_string(),
        "Cannot transfer tokens - controller refuses to transfer more than 61 tokens"
    );
}

#[test]
fn receive_reward_in_different_denoms() {
    let debtor = "debtor";
    let liquidator = "liquidator";

    let denom = "OSMO";
    let reward_denom = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(liquidator, &coins(1200, denom))
        .with_funds(debtor, &coins(1000, reward_denom))
        .with_liquidation_price(Decimal::percent(92))
        .build();

    // create market denom osmo
    suite
        .create_market_quick(
            "gov",
            "osmo",
            denom,
            Decimal::percent(80),                        // collateral price
            (Decimal::percent(5), Decimal::percent(20)), // interest rates (base, slope)
        )
        .unwrap();
    // create reward_denom market eth
    suite
        .create_market_quick(
            "gov",
            "eth",
            reward_denom,
            Decimal::percent(80),
            (Decimal::percent(2), Decimal::percent(25)), // interest rates (base, slope)
        )
        .unwrap();

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();
    suite
        .oracle_set_price_market_per_common(reward_denom, Decimal::percent(200))
        .unwrap();

    // debtor deposits 1000 eth
    suite
        .deposit_tokens_on_market(debtor, coin(1000, reward_denom))
        .unwrap();
    // liquidator deposits 1000 osmo and borrows 400 eth
    suite
        .deposit_tokens_on_market(liquidator, coin(1000, denom))
        .unwrap();
    suite
        .borrow_tokens_from_market(liquidator, coin(400, reward_denom))
        .unwrap();
    // debtor borrows 1000 osmo
    suite
        .borrow_tokens_from_market(debtor, coin(1000, denom))
        .unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(2000),  // 1000 deposited * 2.0
            credit_line: Uint128::new(1600), // 2000 collateral * 0.8 collateral price
            debt: Uint128::new(1000)         // 1000 * 1.0 price
        }
    );
    let total_credit_line = suite.query_total_credit_line(liquidator).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(1000), // 1000 deposited * 1.0
            credit_line: Uint128::new(800), // 1000 collateral * 0.8 collateral price
            debt: Uint128::new(800)         // 400 borrowed * 2.0 price
        }
    );

    suite.advance_seconds(YEAR_IN_SECONDS);

    suite
        .oracle_set_price_market_per_common(denom, Decimal::percent(100))
        .unwrap();
    suite
        .oracle_set_price_market_per_common(reward_denom, Decimal::percent(110))
        .unwrap();

    // Repay some tokens to trigger interest rate charges
    suite
        .repay_tokens_on_market(debtor, coin(20, denom))
        .unwrap();

    // current interest rates
    // rates = base + slope * utilization
    // denom = 5% + 20% * (1000/1000) = 5% + 20% = 25%
    // reward denom = 2% + 25% * (800/2000) = 2% + 25% * 40% = 2% + 0.1% = 12%

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(1100),
            credit_line: Uint128::new(880),
            debt: Uint128::new(1229) // 1000 * 25% interest - 20 repaid = 1250 - 20 = 1230 ~= 1229 (FIXME rounding error?)
        }
    );

    // successful liquidation of 185 tokens
    suite
        .liquidate(
            liquidator,
            debtor,
            &coins(30, denom),
            reward_denom.to_owned(),
        )
        .unwrap();

    // Liquidation price is 0.92
    // Repaid value is 130 * 2.0 (oracle's price for reward denom) * 0.92 = 239.2 leth
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(1451),
            credit_line: Uint128::new(1160),
            // 1337 - 185 = 1152
            debt: Uint128::new(1152),
        }
    );

    let total_credit_line = suite.query_total_credit_line(liquidator).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineResponse {
            collateral: Uint128::new(185),
            credit_line: Uint128::new(0),
            debt: Uint128::zero()
        }
    );
}

// #[test]
// fn receive_reward_in_different_denomxxx() {
//     let debtor = "debtor";
//     let liquidator = "liquidator";
//
//     let denom = "OSMO";
//     let reward_denom = "ETH";
//
//     let mut suite = SuiteBuilder::new()
//         .with_gov("gov")
//         .with_funds(liquidator, &coins(5000, denom))
//         .with_funds(debtor, &[coin(600, denom), coin(500, reward_denom)])
//         .with_liquidation_price(Decimal::percent(92))
//         .build();
//
//     // create market with very high interest rates
//     suite
//         .create_market_quick(
//             "gov",
//             "osmo",
//             denom,
//             Decimal::percent(80),
//             (Decimal::percent(200), Decimal::percent(45)),
//         )
//         .unwrap();
//     // create reward_denom market
//     suite
//         .create_market_quick("gov", "eth", reward_denom, Decimal::percent(80), None)
//         .unwrap();
//
//     suite
//         .oracle_set_price_market_per_common(denom, Decimal::percent(100))
//         .unwrap();
//     suite
//         .oracle_set_price_market_per_common(reward_denom, Decimal::percent(10))
//         .unwrap();
//
//     suite
//         .deposit_tokens_on_market(debtor, coin(500, denom))
//         .unwrap();
//     // debtor deposits some tokens in reward_denom market
//     suite
//         .deposit_tokens_on_market(debtor, coin(100, reward_denom))
//         .unwrap();
//
//     suite
//         .borrow_tokens_from_market(debtor, coin(400, denom))
//         .unwrap();
//
//     suite.advance_seconds(YEAR_IN_SECONDS);
//
//     suite
//         .oracle_set_price_market_per_common(denom, Decimal::percent(100))
//         .unwrap();
//     suite
//         .oracle_set_price_market_per_common(reward_denom, Decimal::percent(10))
//         .unwrap();
//
//     // Repay some tokens to trigger interest rate charges
//     suite
//         .repay_tokens_on_market(debtor, coin(3, denom))
//         .unwrap();
//
//     let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
//     // just for status
//     assert_eq!(
//         total_credit_line,
//         CreditLineResponse {
//             collateral: Uint128::new(1453),
//             credit_line: Uint128::new(1162),
//             debt: Uint128::new(1337)
//         }
//     );
//
//     // successful liquidation of 185 tokens
//     suite
//         .liquidate(
//             liquidator,
//             debtor,
//             &coins(185, denom),
//             reward_denom.to_owned(),
//         )
//         .unwrap();
//
//     // Liquidation price is 0.92
//     // Repaid value is 185 * 0.1 (oracle's price for reward denom) * 0.92 = 5
//     let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
//     assert_eq!(
//         total_credit_line,
//         CreditLineResponse {
//             collateral: Uint128::new(1451),
//             credit_line: Uint128::new(1160),
//             // 1337 - 185 = 1152
//             debt: Uint128::new(1152),
//         }
//     );
//
//     let total_credit_line = suite.query_total_credit_line(liquidator).unwrap();
//     assert_eq!(
//         total_credit_line,
//         CreditLineResponse {
//             collateral: Uint128::new(185),
//             credit_line: Uint128::new(0),
//             debt: Uint128::zero()
//         }
//     );
// }
