use anyhow::Result as AnyResult;

use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::Config;

fn contract_credit_agency() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);

    Box::new(contract)
}

fn contract_market() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        lendex_market::contract::execute,
        lendex_market::contract::instantiate,
        lendex_market::contract::query,
    )
    .with_reply(lendex_market::contract::reply);

    Box::new(contract)
}

fn contract_token() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        lendex_token::contract::execute,
        lendex_token::contract::instantiate,
        lendex_token::contract::query,
    );

    Box::new(contract)
}

/// Builder for test suite
#[derive(Debug)]
pub struct SuiteBuilder {}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {}
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let _token_id = app.store_code(contract_token());
        let _market_id = app.store_code(contract_market());
        let contract_id = app.store_code(contract_credit_agency());
        let contract = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {},
                &[],
                "credit-agency",
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
