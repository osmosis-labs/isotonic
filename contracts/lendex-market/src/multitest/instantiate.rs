use cosmwasm_std::Addr;

use super::suite::SuiteBuilder;
use crate::state::Config;

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
