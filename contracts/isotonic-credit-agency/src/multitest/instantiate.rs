use cosmwasm_std::{Addr, Decimal};

use super::suite::{SuiteBuilder, COMMON};
use crate::state::Config;

#[test]
fn market_instantiate_and_query_config() {
    let suite = SuiteBuilder::new()
        .with_gov("gov")
        .with_reward_token("ENG")
        .build();

    assert_eq!(
        Config {
            gov_contract: Addr::unchecked("gov"),
            isotonic_market_id: 2,
            isotonic_token_id: 3,
            reward_token: "ENG".to_owned(),
            common_token: COMMON.to_owned(),
            liquidation_price: Decimal::percent(92),
            liquidation_fee: Decimal::permille(45),
            liquidation_initiation_fee: Decimal::permille(5),
        },
        suite.query_config().unwrap()
    );
}
