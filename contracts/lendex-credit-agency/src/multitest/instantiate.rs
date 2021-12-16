use cosmwasm_std::Addr;

use super::suite::SuiteBuilder;
use crate::state::Config;

#[test]
fn market_instantiate_and_query_config() {
    let suite = SuiteBuilder::new().with_gov("gov").build();

    assert_eq!(
        Config {
            gov_contract: Addr::unchecked("gov"),
            lendex_market_id: 1,
            lendex_token_id: 2
        },
        suite.query_config().unwrap()
    );
}
