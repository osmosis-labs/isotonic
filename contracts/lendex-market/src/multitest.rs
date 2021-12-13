mod borrow_repay;
mod deposit;
mod instantiate;
pub mod suite;
mod withdraw;

use cosmwasm_std::{coin, Addr, Decimal, StdError, Uint128};

use crate::msg::Interest;
use crate::state::Config;
use suite::SuiteBuilder;

#[test]
fn market_instantiate_and_query_config() {
    let suite = SuiteBuilder::new().build();

    assert_eq!(
        Config {
            ltoken_contract: Addr::unchecked("Contract #1"),
            btoken_contract: Addr::unchecked("Contract #2"),
            name: "lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            token_id: 1,
            base_asset: "native_denom".to_owned(),
            rates: Interest::Linear {
                base: Decimal::percent(3),
                slope: Decimal::percent(20)
            }
        },
        suite.query_config().unwrap()
    );
}

#[test]
fn query_transferable_amount() {
    let lender = "lender";
    let base_asset = "base";
    let mut suite = SuiteBuilder::new()
        .with_base_asset(base_asset)
        .with_funds(lender, &[coin(100, base_asset)])
        .build();

    let btoken = suite.btoken();
    let resp = suite.query_transferable_amount(btoken, lender).unwrap();
    assert_eq!(Uint128::zero(), resp);

    let ltoken = suite.ltoken();
    let resp = suite
        .query_transferable_amount(ltoken.clone(), lender)
        .unwrap();
    assert_eq!(Uint128::zero(), resp);

    // Deposit base asset and mint some L tokens, then query again
    suite.deposit(lender, &[coin(100, base_asset)]).unwrap();
    let resp = suite.query_transferable_amount(ltoken, lender).unwrap();
    assert_eq!(Uint128::new(100), resp);

    let resp = suite
        .query_transferable_amount("xtoken", lender)
        .unwrap_err();
    assert_eq!(
        StdError::generic_err(
            "Querier contract error: Generic error: Unrecognized token: xtoken".to_owned()
        ),
        resp.downcast().unwrap()
    );
}
