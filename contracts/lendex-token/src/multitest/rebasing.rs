use super::suite::{Suite, SuiteBuilder};
use crate::msg::DisplayAmount;
use crate::multitest::receiver::Cw20ExecMsg;
use cosmwasm_std::{to_binary, Decimal, Uint128};

#[test]
fn queries() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_transferable(lender, Uint128::new(100))
        .build();
    let controller = suite.controller();
    let controller = controller.as_str();

    // Preparation to have anything to query
    suite.mint(controller, lender, Uint128::new(100)).unwrap();

    // Before rebase the multiplier is at 1.0 and we have 100 tokens.
    assert_eq!(
        suite.query_balance(lender).unwrap(),
        DisplayAmount::raw(100u128)
    );
    assert_eq!(
        suite.query_token_info().unwrap().total_supply,
        DisplayAmount::raw(100u128)
    );

    // Rebase by 1.2. The "displayed" tokens are now at 120. The multiplier is at 1.2.
    suite.rebase(controller, Decimal::percent(120)).unwrap();
    assert_eq!(suite.query_multiplier().unwrap(), Decimal::percent(120));
    assert_eq!(
        suite.query_balance(lender).unwrap(),
        DisplayAmount::raw(120u128)
    );
    assert_eq!(
        suite.query_token_info().unwrap().total_supply,
        DisplayAmount::raw(120u128)
    );

    // Another rebase by 1.2. The "displayed" tokens are now at 144. The multiplier is at 1.44.
    suite.rebase(controller, Decimal::percent(120)).unwrap();
    assert_eq!(suite.query_multiplier().unwrap(), Decimal::percent(144));
    assert_eq!(
        suite.query_balance(lender).unwrap(),
        DisplayAmount::raw(144u128)
    );
    assert_eq!(
        suite.query_token_info().unwrap().total_supply,
        DisplayAmount::raw(144u128)
    );
}

#[test]
fn mint() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_transferable(lender, Uint128::new(100))
        .build();
    let controller = suite.controller();
    let controller = controller.as_str();

    suite.mint(controller, lender, Uint128::new(100)).unwrap();

    // Rebase by 1.25. The "displayed" tokens are now at 125. The multiplier is at 1.25.
    suite.rebase(controller, Decimal::percent(125)).unwrap();
    assert_eq!(suite.query_multiplier().unwrap(), Decimal::percent(125));
    assert_eq!(
        suite.query_balance(lender).unwrap(),
        DisplayAmount::raw(125u128)
    );

    // Mint 20 with the multiplier at 1.25. The actual stored amount is 16.
    suite.mint(controller, lender, Uint128::new(20)).unwrap();

    // Reverse the rebase so that the multiplier is back at 1.0
    suite.rebase(controller, Decimal::percent(80)).unwrap();
    assert_eq!(suite.query_multiplier().unwrap(), Decimal::percent(100));
    assert_eq!(
        suite.query_balance(lender).unwrap(),
        DisplayAmount::raw(116u128)
    );
}

#[test]
fn transfer() {
    let lender = "lender";
    let receiver = "receiver";
    let mut suite = SuiteBuilder::new()
        .with_transferable(lender, Uint128::new(100))
        .build();
    let controller = suite.controller();
    let controller = controller.as_str();

    // Preparation to have anything to transfer
    suite.mint(controller, lender, Uint128::new(100)).unwrap();

    // Rebase by 1.20
    suite.rebase(controller, Decimal::percent(120)).unwrap();

    suite.transfer(lender, receiver, Uint128::new(24)).unwrap();

    assert_eq!(
        suite.query_balance(lender).unwrap(),
        DisplayAmount::raw(96u128)
    );
    assert_eq!(
        suite.query_balance(receiver).unwrap(),
        DisplayAmount::raw(24u128)
    );
    assert_eq!(
        suite.query_balance(controller).unwrap(),
        DisplayAmount::zero()
    );
}

#[test]
fn send() {
    let lender = "lender";
    let mut suite = SuiteBuilder::new()
        .with_transferable(lender, Uint128::new(100))
        .build();
    let controller = suite.controller();
    let controller = controller.as_str();
    let receiver = suite.receiver();
    let receiver = receiver.as_str();

    // Preparation to have anything to send
    suite.mint(controller, lender, Uint128::new(100)).unwrap();

    // Rebase by 1.2, the "displayed" tokens are now at 120.
    suite.rebase(controller, Decimal::percent(120)).unwrap();

    let exec = to_binary(&Cw20ExecMsg::Valid {}).unwrap();

    suite
        .send(lender, receiver, Uint128::new(24), exec)
        .unwrap();

    assert_eq!(suite.query_receiver().unwrap(), 1);
    assert_eq!(
        suite.query_balance(lender).unwrap(),
        DisplayAmount::raw(96u128)
    );
    assert_eq!(
        suite.query_balance(receiver).unwrap(),
        DisplayAmount::raw(24u128)
    );
    assert_eq!(
        suite.query_balance(controller).unwrap(),
        DisplayAmount::zero()
    );
}

#[test]
fn burn() {
    let mut suite = Suite::new();
    let controller = suite.controller();
    let controller = controller.as_str();

    // Preparation to have anything to burnground
    suite
        .mint(controller, controller, Uint128::new(100))
        .unwrap();

    // Rebase by 1.25, the "displayed" tokens are now at 125.
    suite.rebase(controller, Decimal::percent(125)).unwrap();

    suite.burn(controller, Uint128::new(25)).unwrap();
    assert_eq!(
        suite.query_balance(controller).unwrap(),
        DisplayAmount::raw(100u128)
    );

    // Reverse the rebase so that the multiplier is back at 1.0
    suite.rebase(controller, Decimal::percent(80)).unwrap();
    assert_eq!(
        suite.query_balance(controller).unwrap(),
        DisplayAmount::raw(80u128)
    );
}
