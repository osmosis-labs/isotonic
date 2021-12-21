use cosmwasm_std::{Addr, Decimal};
use utils::interest::Interest;

use super::suite::SuiteBuilder;
use crate::state::Config;

#[test]
fn market_instantiate_and_query_config() {
    let mut suite = SuiteBuilder::new().build();
    let time = suite.app().block_info().time.seconds();

    assert_eq!(
        Config {
            ltoken_contract: Addr::unchecked("Contract #2"),
            btoken_contract: Addr::unchecked("Contract #3"),
            name: "lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            token_id: 2,
            market_token: "native_denom".to_owned(),
            rates: Interest::Linear {
                base: Decimal::percent(3),
                slope: Decimal::percent(20)
            },
            interest_charge_period: 300,
            // env.block.time.seconds() - env.block.time.seconds() % epoch_length
            last_charged: time - time % 300,
            common_token: Some("common".to_owned()),
            collateral_ratio: Decimal::percent(50),
            price_oracle: "Contract #0".to_owned(),
        },
        suite.query_config().unwrap()
    );
}
