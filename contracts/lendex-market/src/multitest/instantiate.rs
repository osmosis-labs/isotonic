use cosmwasm_std::{Addr, Decimal, Timestamp};
use utils::interest::Interest;

use super::suite::SuiteBuilder;
use crate::state::Config;

#[test]
fn market_instantiate_and_query_config() {
    let mut suite = SuiteBuilder::new().build();
    let time = suite.app().block_info().time.seconds();

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
            },
            interest_charge_period: 300,
            // env.block.time.seconds() - env.block.time.seconds() % epoch_length
            last_charged: time - time % 300
        },
        suite.query_config().unwrap()
    );
}
