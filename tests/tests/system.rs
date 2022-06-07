use cosmwasm_std::{coin, coins, Decimal, Uint128};

use tests::{MarketBuilder, SuiteBuilder};
use utils::credit_line::CreditLineValues;

const YEAR_IN_SECONDS: u64 = 365 * 24 * 3600;

/// This can expose rounding issues.
/// https://github.com/confio/isotonic/issues/40
#[test]
fn withdraw_whole_deposit() {
    let lender = "lender";
    let charge_period = 100;
    let mut suite = SuiteBuilder::new()
        .with_common_token("OSMO")
        .with_funds(lender, &[coin(u128::MAX, "ATOM")])
        .with_market(MarketBuilder::new("ATOM").with_charge_period(charge_period))
        .with_pool(1, (coin(100, "OSMO"), coin(100, "ATOM")))
        .build();

    let inputs = [
        1,
        99,
        100,
        999_999,
        1_000_000,
        999_999_999_999,
        1_000_000_000_000,
        999_999_999_999_999_999,
        1_000_000_000_000_000_000,
        9_999_999_999_999_999_999,
        10_000_000_000_000_000_000,
    ];

    for input in inputs {
        suite.deposit(lender, coin(input, "ATOM")).unwrap();

        // no interest should be charged/earned here as there are no borrowers,
        // so this shouldn't matter
        suite.advance_seconds(charge_period * 2);

        suite.assert_withdrawable(lender, coin(input, "ATOM"));
        suite.attempt_withdraw_max(lender, "ATOM").unwrap();
    }
}

#[test]
fn withdraw_whole_deposit_after_being_repaid() {
    let lender = "lender";
    let borrower = "borrower";
    let mut suite = SuiteBuilder::new()
        .with_common_token("OSMO")
        .with_funds(lender, &[coin(u128::MAX, "ATOM")])
        .with_funds(borrower, &[coin(u128::MAX, "ETH")])
        .with_pool(1, (coin(1, "OSMO"), coin(1, "ATOM")))
        .with_pool(2, (coin(1, "OSMO"), coin(1, "ETH")))
        .with_market(MarketBuilder::new("ATOM"))
        .with_market(MarketBuilder::new("ETH").with_collateral_ratio(Decimal::percent(99)))
        .build();

    suite
        .deposit(borrower, coin(10_u128.pow(20), "ETH"))
        .unwrap();

    let inputs = [
        1,
        99,
        100,
        999_999,
        1_000_000,
        999_999_999_999,
        1_000_000_000_000,
        999_999_999_999_999_999,
        1_000_000_000_000_000_000,
    ];

    for input in inputs {
        suite.deposit(lender, coin(input, "ATOM")).unwrap();

        suite.assert_borrowable(borrower, coin(input, "ATOM"));
        suite.attempt_borrow_max(borrower, "ATOM").unwrap();

        suite.repay(borrower, coin(input, "ATOM")).unwrap();

        suite.assert_withdrawable(lender, coin(input, "ATOM"));
        suite.attempt_withdraw_max(lender, "ATOM").unwrap();

        suite.assert_borrowable(borrower, coin(0, "ATOM"));
    }
}

#[test]
fn receive_reward_in_different_denoms_no_interest_rates() {
    let debtor = "debtor";
    let liquidator = "liquidator";
    let lender1 = "lender1";
    let lender2 = "lender2";

    let common = "COMMON";
    let osmo = "OSMO";
    let eth = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_common_token(common)
        .with_funds(lender1, &coins(160_000, eth))
        .with_funds(lender2, &coins(160_000, eth))
        .with_funds(debtor, &coins(5_000, osmo))
        .with_liquidation_fee(Decimal::percent(4))
        .with_liquidation_initiation_fee(Decimal::percent(1))
        .with_pool(1, (coin(400_000, common), coin(100_000, osmo)))
        .with_pool(2, (coin(100_000, common), coin(1_000_000, eth)))
        .with_market(
            MarketBuilder::new(osmo)
                .with_linear_interest(Decimal::percent(3), Decimal::percent(20)),
        )
        .with_market(
            MarketBuilder::new(eth).with_linear_interest(Decimal::percent(3), Decimal::percent(20)),
        )
        .build();

    suite.deposit(debtor, coin(4_000, osmo)).unwrap();
    suite.deposit(lender1, coin(75_000, eth)).unwrap();
    suite.deposit(lender2, coin(25_000, eth)).unwrap();
    suite.borrow(debtor, coin(75_000, eth)).unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(16_000), // 4000 deposited * 4.0
            credit_line: Uint128::new(8_000), // 16000 collateral * 0.5 collateral price
            debt: Uint128::new(7_500)         // 75_000 * 0.1
        }
        .make_response(suite.common_token().clone())
    );

    suite
        .set_pool(&[(1, (coin(300_000, common), coin(100_000, osmo)))])
        .unwrap();
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(12_000), // 4000 deposited * 3.0
            credit_line: Uint128::new(6_000), // 12000 collateral * 0.5 collateral price
            debt: Uint128::new(7_500)         // 75_000 * 0.1
        }
        .make_response(suite.common_token().clone())
    );

    suite
        .liquidate(liquidator, debtor, osmo, coin(60_000, eth))
        .unwrap();

    suite.reset_pools().unwrap();
    suite
        .set_pool(&[(1, (coin(300_000, common), coin(100_000, osmo)))])
        .unwrap();

    // 60k ETH + liquidation initiation fee + liquidation fee = 63k ETH
    let expected_swap_price = suite
        .estimate_swap_exact_out(osmo, coin(63_000, eth))
        .unwrap();
    let expected_collateral = Uint128::new(12_000) - (expected_swap_price * Uint128::new(3));

    // Liquidation price is 0.92
    // Repaid value is 60_000 ust * 0.1 / 3.0 / 0.92 = 2000 / 0.92 ~= 1999 / 0.92 = 2173 LATOM
    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: expected_collateral,
            credit_line: expected_collateral / Uint128::new(2),
            // 7500 - (60_000 * 0.1)
            debt: Uint128::new(1_500),
        }
        .make_response(suite.common_token().clone())
    );
    let balance = suite.query_tokens_balance(eth, debtor).unwrap();
    assert_eq!(balance.btokens, Uint128::new(15_000)); // 1500 / 0.1 price
    let balance = suite.query_tokens_balance(osmo, debtor).unwrap();
    assert_eq!(balance.ltokens, Uint128::new(4_000) - expected_swap_price);

    // Liquidation initiator earns 1% of 60k ETH, which is 600 ETH
    let balance = suite.query_tokens_balance(eth, liquidator).unwrap();
    assert_eq!(balance.ltokens, Uint128::new(600));

    // Liquidation fees are 4% of 60k ETH - 2400 ETH split between lenders like so:
    // lender1 - 1800 ETH
    // lender2 -  600 ETH
    let balance = suite.query_tokens_balance(eth, lender1).unwrap();
    assert_eq!(balance.ltokens, Uint128::new(75_000 + 1_800));

    let balance = suite.query_tokens_balance(eth, lender2).unwrap();
    assert_eq!(balance.ltokens, Uint128::new(25_000 + 600));
}

#[test]
#[ignore]
fn receive_reward_in_different_denoms_with_six_months_interests() {
    let debtor = "debtor";
    let liquidator = "liquidator";
    let others = "others";

    let common = "COMMON";
    let osmo = "OSMO";
    let eth = "ETH";

    let mut suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_funds(others, &[coin(100_000, eth), coin(10_000, osmo)])
        .with_funds(debtor, &coins(5_000, osmo))
        .with_liquidation_fee(Decimal::percent(7))
        .with_liquidation_initiation_fee(Decimal::percent(1))
        .with_pool(1, (coin(400_000, common), coin(100_000, osmo)))
        .with_pool(2, (coin(10_000, common), coin(100_000, eth)))
        .with_market(MarketBuilder::new(osmo).with_charge_period(YEAR_IN_SECONDS / 2))
        .with_market(MarketBuilder::new(eth).with_charge_period(YEAR_IN_SECONDS / 2))
        .build();

    // investments from the others
    suite.deposit(others, coin(10_000, osmo)).unwrap();
    suite.deposit(others, coin(100_000, eth)).unwrap();
    suite.borrow(others, coin(2_000, osmo)).unwrap();
    suite.borrow(others, coin(20_000, eth)).unwrap();

    // debtor deposits 4000 atom
    suite.deposit(debtor, coin(4_000, osmo)).unwrap();
    // debtor borrows 75_000 ust
    suite.borrow(debtor, coin(75_000, eth)).unwrap();

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            collateral: Uint128::new(16_000), // 4000 deposited * 4.0
            credit_line: Uint128::new(8_000), // 16000 collateral * 0.5 collateral price
            debt: Uint128::new(7_500)         // 75_000 * 0.1
        }
        .make_response(suite.common_token().clone())
    );

    suite.advance_seconds(YEAR_IN_SECONDS / 2);

    // change ATOM price to 3.0 per common denom
    suite
        .set_pool(&[(1, (coin(300_000, common), coin(100_000, osmo)))])
        .unwrap();

    // current interest rates
    // rates = (base + slope * utilization) / 2 (half year)
    // atom = (3% + 20% * (2000/(10_000 + 4000))) / 2 = (3% + 20% * 14.3%) / 2 = (3% + 2.8%) / 2 = 2.9% ~= 3%
    // ust = (3% + 20% * ((20_000 + 75_000)/100_000)) / 2 = (3% + 20% * 95%) / 2 = (3% + 19%) / 2 = 11%

    // expected numbers before liquidation
    // LATOM = 4000 + (2000 * 0.03 * 4000/14000) = 4017
    // BUST = 75_000 * 1.11 * 0.1 = 8325

    // TODO: same problem as in the above test
    // successful liquidation of 6000 tokens
    suite
        .liquidate(liquidator, debtor, osmo, coin(60_000, eth))
        .unwrap();

    // Liquidation price is 0.92
    // Repaid value is 60_000 ust * 0.1 / 3.0 / 0.92 = 2000 / 0.92 ~= 1999 / 0.92 = 2172 LATO

    let balance = suite.query_tokens_balance(eth, debtor).unwrap();
    // 75_000 * 1.11 (interests) - 60_000 (repaid) = 83250 - 60000
    assert_eq!(balance.btokens, Uint128::new(23_250));
    let balance = suite.query_tokens_balance(osmo, debtor).unwrap();
    // amount left after paying liquidation reward
    // 4017 - 2172 repaid = 1845 FIXME: rounding issue
    assert_eq!(balance.ltokens, Uint128::new(1_843));

    let balance = suite.query_tokens_balance(osmo, liquidator).unwrap();
    assert_eq!(balance.ltokens, Uint128::new(2_172)); // repaid amount as reward

    let total_credit_line = suite.query_total_credit_line(debtor).unwrap();
    assert_eq!(
        total_credit_line,
        CreditLineValues {
            // 1843 * 3 = 5529
            collateral: Uint128::new(5_529),
            // 5529 * 0.5 collateral price
            credit_line: Uint128::new(2_764),
            // 8375 - (60_000 * 0.1)
            debt: Uint128::new(2_325),
        }
        .make_response(suite.common_token().clone())
    );
}
