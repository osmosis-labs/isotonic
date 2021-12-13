mod deposit;
pub mod suite;
mod withdraw;

use cosmwasm_std::{Addr, StdError, Uint128};

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
        },
        suite.query_config().unwrap()
    );
}

#[test]
fn query_transferable_amount() {
    let suite = SuiteBuilder::new().build();

    let btoken = suite.btoken();
    let resp = suite.query_transferable_amount(btoken, "actor").unwrap();
    assert_eq!(Uint128::zero(), resp);

    // TODO: Mint tokens and query this again during/after
    // https://github.com/confio/lendex/issues/6
    let ltoken = suite.ltoken();
    let resp = suite.query_transferable_amount(ltoken, "actor").unwrap();
    assert_eq!(Uint128::zero(), resp);

    let resp = suite
        .query_transferable_amount("xtoken", "actor")
        .unwrap_err();
    assert_eq!(
        StdError::generic_err(
            "Querier contract error: Generic error: Unrecognized token: xtoken".to_owned()
        ),
        resp.downcast().unwrap()
    );
}
