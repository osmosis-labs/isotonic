use anyhow::Result as AnyResult;

use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::Config;

fn contract_market() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
        .with_reply(crate::contract::reply);

    Box::new(contract)
}

/// Builder for test suite
#[derive(Debug)]
pub struct SuiteBuilder {
    /// Lendex token name
    name: String,
    /// Lendex token symbol
    symbol: String,
    /// Lendex token precision
    decimals: u8,
    /// codeId used to create lendex tokens
    token_id: u64,
    /// Native denom for the base asset
    base_asset: String,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            name: "lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            token_id: 1,
            base_asset: "native_denom".to_owned(),
        }
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let contract_id = app.store_code(contract_market());
        let contract = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    name: self.name,
                    symbol: self.symbol,
                    decimals: self.decimals,
                    token_id: self.token_id,
                    base_asset: self.base_asset,
                },
                &[],
                "market",
                Some(owner.to_string()),
            )
            .unwrap();

        Suite { app, contract }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Address of Market contract
    contract: Addr,
}

impl Suite {
    /// Queries market contract for configuration
    pub fn query_config(&self) -> AnyResult<Config> {
        let resp: Config = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Configuration {})?;
        Ok(resp)
    }
}
