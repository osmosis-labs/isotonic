use anyhow::Result as AnyResult;

use cosmwasm_std::{Addr, Decimal, Empty};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use utils::interest::Interest;

use crate::msg::{
    ExecuteMsg, InstantiateMsg, ListMarketsResponse, MarketConfig, MarketResponse, QueryMsg,
};
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
pub struct SuiteBuilder {
    gov_contract: String,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            gov_contract: "owner".to_string(),
        }
    }

    pub fn with_gov(mut self, gov: impl ToString) -> Self {
        self.gov_contract = gov.to_string();
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        let gov_contract = self.gov_contract;

        let lendex_market_id = app.store_code(contract_market());
        let lendex_token_id = app.store_code(contract_token());
        let contract_id = app.store_code(contract_credit_agency());
        let contract = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    gov_contract,
                    lendex_market_id,
                    lendex_token_id,
                },
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
    /// Address of the Credit Agency contract
    contract: Addr,
}

impl Suite {
    pub fn create_market(&mut self, caller: &str, cfg: MarketConfig) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(caller),
            self.contract.clone(),
            &ExecuteMsg::CreateMarket(cfg),
            &[],
        )
    }

    pub fn create_market_quick(
        &mut self,
        caller: &str,
        lendex_token: &str,
        base_asset: &str,
    ) -> AnyResult<AppResponse> {
        self.create_market(
            caller,
            MarketConfig {
                name: lendex_token.to_string(),
                symbol: lendex_token.to_string(),
                decimals: 9,
                base_asset: base_asset.to_string(),
                interest_rate: Interest::Linear {
                    base: Decimal::percent(3),
                    slope: Decimal::percent(20),
                },
                distributed_token: "osmo".to_owned(),
            },
        )
    }

    /// Queries the Credit Agency contract for configuration
    pub fn query_config(&self) -> AnyResult<Config> {
        let resp: Config = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Configuration {})?;
        Ok(resp)
    }

    /// Queries the Credit Agency contract for market addr
    pub fn query_market(&self, asset: &str) -> AnyResult<MarketResponse> {
        let resp: MarketResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Market {
                base_asset: asset.to_string(),
            },
        )?;
        Ok(resp)
    }

    pub fn assert_market(&self, asset: &str) {
        let res = self.query_market(asset).unwrap();
        assert_eq!(res.base_asset, asset);
        // TODO: verify res.market is an existing contract in the app at this point?
    }

    /// Queries the Credit Agency contract for a list of markets with pagination
    pub fn list_markets(&self) -> AnyResult<ListMarketsResponse> {
        self.list_markets_with_pagination(None, None)
    }

    /// Queries the Credit Agency contract for a list of markets with pagination
    pub fn list_markets_with_pagination(
        &self,
        start_after: impl Into<Option<String>>,
        limit: impl Into<Option<u32>>,
    ) -> AnyResult<ListMarketsResponse> {
        let resp: ListMarketsResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::ListMarkets {
                start_after: start_after.into(),
                limit: limit.into(),
            },
        )?;
        Ok(resp)
    }
}
