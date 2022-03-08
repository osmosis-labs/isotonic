use cosmwasm_std::{coin, coins, Addr, Decimal};

use super::suite::SuiteBuilder;
use crate::error::ContractError;

use utils::token::Token;

#[test]
fn enter_market() {
    // This test does not test full flow in which market would be entered on first market operation - it
    // just tests if the market contract can properly introduce account to it.

    let gov = "gov";
    let denom = "OSMO";
    let actor = "actor";

    let mut suite = SuiteBuilder::new().with_gov(gov).build();

    suite
        .create_market_quick(gov, "osmo", denom, None, None, None)
        .unwrap();

    let market = suite.query_market(denom).unwrap().market;

    assert!(!suite.is_on_market(actor, market.as_str()).unwrap());

    let markets = suite.list_all_entered_markets(actor).unwrap();
    assert!(markets.is_empty());

    suite.enter_market(market.as_str(), actor).unwrap();

    assert!(suite.is_on_market(actor, market.as_str()).unwrap());

    let markets = suite.list_all_entered_markets(actor).unwrap();
    assert_eq!(markets, vec![market]);
}

#[test]
fn enter_market_by_deposit() {
    let gov = "gov";
    let denom = "OSMO";
    let actor = "actor";

    let mut suite = SuiteBuilder::new()
        .with_gov(gov)
        .with_funds(actor, &coins(500, denom))
        .build();

    suite
        .create_market_quick(gov, "osmo", denom, None, None, None)
        .unwrap();

    suite
        .deposit_tokens_on_market(actor, coin(500, denom))
        .unwrap();

    let market = suite.query_market(denom).unwrap().market;

    assert!(suite.is_on_market(actor, market.as_str()).unwrap());

    let markets = suite.list_all_entered_markets(actor).unwrap();
    assert_eq!(markets, vec![market]);
}

#[test]
fn enter_market_by_borrow() {
    let gov = "gov";
    let denom1 = "OSMO";
    let denom2 = "ETH";
    let actor1 = "actor1";
    let actor2 = "actor2";

    let mut suite = SuiteBuilder::new()
        .with_gov(gov)
        .with_funds(actor1, &coins(500, denom1))
        .with_funds(actor2, &coins(500, denom2))
        .build();

    suite
        .create_market_quick(gov, "osmo", denom1, None, None, None)
        .unwrap();

    suite
        .create_market_quick(gov, "eth", denom2, None, None, None)
        .unwrap();

    suite
        .oracle_set_price_market_per_common(Token::Native(denom1.to_owned()), Decimal::one())
        .unwrap();
    suite
        .oracle_set_price_market_per_common(Token::Native(denom2.to_owned()), Decimal::one())
        .unwrap();

    // Creating some liquidity on actor1
    suite
        .deposit_tokens_on_market(actor1, coin(500, denom1))
        .unwrap();

    // Need some tokens on market2
    suite
        .deposit_tokens_on_market(actor2, coin(500, denom2))
        .unwrap();

    suite
        .borrow_tokens_from_market(actor1, coin(200, denom2))
        .unwrap();

    let market1 = suite.query_market(denom1).unwrap().market;
    let market2 = suite.query_market(denom2).unwrap().market;

    assert!(suite.is_on_market(actor1, market2.as_str()).unwrap());

    let mut markets = suite.list_all_entered_markets(actor1).unwrap();
    markets.sort();
    assert_eq!(markets, vec![market1, market2]);
}

#[test]
fn exit_market() {
    let gov = "gov";
    let denom = "OSMO";
    let actor = "actor";

    let mut suite = SuiteBuilder::new().with_gov(gov).build();

    suite
        .create_market_quick(gov, "osmo", denom, None, None, None)
        .unwrap();

    let market = suite.query_market(denom).unwrap().market;
    suite.enter_market(market.as_str(), actor).unwrap();

    suite.exit_market(actor, market.as_str()).unwrap();

    assert!(!suite.is_on_market(actor, market.as_str()).unwrap());
    assert!(suite.list_all_entered_markets(actor).unwrap().is_empty());
}

#[test]
fn cant_exit_market_not_being_part_of() {
    let gov = "gov";
    let denom = "OSMO";
    let actor = "actor";

    let mut suite = SuiteBuilder::new().with_gov(gov).build();

    suite
        .create_market_quick(gov, "osmo", denom, None, None, None)
        .unwrap();

    let market = suite.query_market(denom).unwrap().market;

    let err = suite.exit_market(actor, market.as_str()).unwrap_err();

    assert_eq!(
        ContractError::NotOnMarket {
            address: Addr::unchecked(actor),
            market: market.clone()
        },
        err.downcast().unwrap()
    );

    assert!(!suite.is_on_market(actor, market.as_str()).unwrap());
    assert!(suite.list_all_entered_markets(actor).unwrap().is_empty());
}

#[test]
fn cent_exit_market_with_borrowed_tokens() {
    // Use case:
    // 1. Actor1 deposits tokens on denom1 to have some collateral
    // 2. Actor2 deposits tokens on denom2 so there is something to borrow
    // 3. Actor1 borrows denom2 tokens
    // 4. Actor1 tries to exit denom2 market, which fails as he has tokens borrowed there

    let gov = "gov";
    let denom1 = "OSMO";
    let denom2 = "ETH";
    let actor1 = "actor1";
    let actor2 = "actor2";

    let mut suite = SuiteBuilder::new()
        .with_gov(gov)
        .with_funds(actor1, &coins(500, denom1))
        .with_funds(actor2, &coins(500, denom2))
        .build();

    suite
        .create_market_quick(gov, "osmo", denom1, None, None, None)
        .unwrap();

    suite
        .create_market_quick(gov, "eth", denom2, None, None, None)
        .unwrap();

    suite
        .oracle_set_price_market_per_common(Token::Native(denom1.to_owned()), Decimal::one())
        .unwrap();
    suite
        .oracle_set_price_market_per_common(Token::Native(denom2.to_owned()), Decimal::one())
        .unwrap();

    suite
        .deposit_tokens_on_market(actor1, coin(500, denom1))
        .unwrap();

    suite
        .deposit_tokens_on_market(actor2, coin(500, denom2))
        .unwrap();

    suite
        .borrow_tokens_from_market(actor1, coin(200, denom2))
        .unwrap();

    let market1 = suite.query_market(denom1).unwrap().market;
    let market2 = suite.query_market(denom2).unwrap().market;

    // actor1 still have borrowed tokens on market2
    let err = suite.exit_market(actor1, market2.as_str()).unwrap_err();
    assert_eq!(
        ContractError::DebtOnMarket {
            address: Addr::unchecked(actor1),
            market: market2.clone(),
            debt: coin(200, &suite.common_token().clone().native().unwrap()),
        },
        err.downcast().unwrap()
    );

    let mut markets = suite.list_all_entered_markets(actor1).unwrap();
    markets.sort();

    assert_eq!(markets, vec![market1, market2]);
}

#[test]
fn cent_exit_market_with_not_enough_liquidity() {
    // Use case:
    // 1. Actor1 deposits tokens on denom1 to have some collateral
    // 2. Actor2 deposits tokens on denom2 so there is something to borrow
    // 3. Actor1 borrows denom2 tokens
    // 4. Actor1 tries to exit denom1 market, which fails as after that he would not have enough
    //    collateral to cover denom2 debt

    let gov = "gov";
    let denom1 = "OSMO";
    let denom2 = "ETH";
    let actor1 = "actor1";
    let actor2 = "actor2";

    let mut suite = SuiteBuilder::new()
        .with_gov(gov)
        .with_funds(actor1, &coins(500, denom1))
        .with_funds(actor2, &coins(500, denom2))
        .build();

    suite
        .create_market_quick(gov, "osmo", denom1, None, None, None)
        .unwrap();

    suite
        .create_market_quick(gov, "eth", denom2, None, None, None)
        .unwrap();

    suite
        .oracle_set_price_market_per_common(Token::Native(denom1.to_owned()), Decimal::one())
        .unwrap();
    suite
        .oracle_set_price_market_per_common(Token::Native(denom2.to_owned()), Decimal::one())
        .unwrap();

    suite
        .deposit_tokens_on_market(actor1, coin(500, denom1))
        .unwrap();

    suite
        .deposit_tokens_on_market(actor2, coin(500, denom2))
        .unwrap();

    suite
        .borrow_tokens_from_market(actor1, coin(200, denom2))
        .unwrap();

    let market1 = suite.query_market(denom1).unwrap().market;
    let market2 = suite.query_market(denom2).unwrap().market;

    // actor1 needs tokens from market1 to have enough liquidity for market2 debt
    let err = suite.exit_market(actor1, market1.as_str()).unwrap_err();
    assert_eq!(
        ContractError::NotEnoughCollat {
            credit_line: 0u128.into(),
            collateral: 0u128.into(),
            debt: 200u128.into(),
        },
        err.downcast().unwrap()
    );

    let mut markets = suite.list_all_entered_markets(actor1).unwrap();
    markets.sort();

    assert_eq!(markets, vec![market1, market2]);
}

#[test]
fn exit_market_with_ltokens() {
    // Use case:
    // 1. Actor1 deposits tokens on denom1 to have some collateral
    // 2. Actor2 deposits tokens on denom3 just because he can
    // 3. Actor2 deposits tokens on denom2 so there is something to borrow
    // 4. Actor1 borrows denom2 tokens
    // 5. Actor1 exits denom3 market - he covers denom2 debt with denom1 ltokens

    let gov = "gov";
    let denom1 = "OSMO";
    let denom2 = "ETH";
    let denom3 = "USDC";
    let actor1 = "actor1";
    let actor2 = "actor2";

    let mut suite = SuiteBuilder::new()
        .with_gov(gov)
        .with_funds(actor1, &[coin(500, denom1), coin(200, denom3)])
        .with_funds(actor2, &coins(500, denom2))
        .build();

    suite
        .create_market_quick(gov, "osmo", denom1, None, None, None)
        .unwrap();

    suite
        .create_market_quick(gov, "eth", denom2, None, None, None)
        .unwrap();

    suite
        .create_market_quick(gov, "usdc", denom3, None, None, None)
        .unwrap();

    suite
        .oracle_set_price_market_per_common(Token::Native(denom1.to_owned()), Decimal::one())
        .unwrap();
    suite
        .oracle_set_price_market_per_common(Token::Native(denom2.to_owned()), Decimal::one())
        .unwrap();
    suite
        .oracle_set_price_market_per_common(Token::Native(denom3.to_owned()), Decimal::one())
        .unwrap();

    suite
        .deposit_tokens_on_market(actor1, coin(500, denom1))
        .unwrap();

    suite
        .deposit_tokens_on_market(actor2, coin(500, denom2))
        .unwrap();

    suite
        .deposit_tokens_on_market(actor1, coin(200, denom3))
        .unwrap();

    suite
        .borrow_tokens_from_market(actor1, coin(200, denom2))
        .unwrap();

    let market1 = suite.query_market(denom1).unwrap().market;
    let market2 = suite.query_market(denom2).unwrap().market;
    let market3 = suite.query_market(denom3).unwrap().market;

    suite.exit_market(actor1, market3.as_str()).unwrap();

    let mut markets = suite.list_all_entered_markets(actor1).unwrap();
    markets.sort();

    assert_eq!(markets, vec![market1, market2]);
}
