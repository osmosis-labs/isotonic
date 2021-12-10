use anyhow::Result as AnyResult;

use cosmwasm_std::{Addr, Empty, Uint128};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::{InstantiateMsg, QueryMsg, TransferableAmountResponse};
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
    /// Lendex token name
    name: String,
    /// Lendex token symbol
    symbol: String,
    /// Lendex token precision
    decimals: u8,
    /// Native denom for the base asset
    base_asset: String,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            name: "lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            base_asset: "native_denom".to_owned(),
        }
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let token_id = app.store_code(contract_token());
        let contract_id = app.store_code(contract_market());
        let contract = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    name: self.name,
                    symbol: self.symbol,
                    decimals: self.decimals,
                    token_id,
                    base_asset: self.base_asset,
                },
                &[],
                "market",
                Some(owner.to_string()),
            )
            .unwrap();

        // query for token contracts
        let config: Config = app
            .wrap()
            .query_wasm_smart(contract.clone(), &QueryMsg::Configuration {})
            .unwrap();

        Suite {
            app,
            contract,
            ltoken_contract: config.ltoken_contract,
            btoken_contract: config.btoken_contract,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Address of Market contract
    contract: Addr,
    /// Address of LToken contract
    ltoken_contract: Addr,
    /// Address of BToken contract
    btoken_contract: Addr,
}

impl Suite {
    /// Gives btoken contract address back
    pub fn btoken(&self) -> Addr {
        self.btoken_contract.clone()
    }

    /// Gives ltoken contract address back
    pub fn ltoken(&self) -> Addr {
        self.ltoken_contract.clone()
    }

    /// Queries market contract for configuration
    pub fn query_config(&self) -> AnyResult<Config> {
        let resp: Config = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Configuration {})?;
        Ok(resp)
    }

    pub fn query_transferable_amount(
        &self,
        token: impl ToString,
        account: impl ToString,
    ) -> AnyResult<Uint128> {
        let resp: TransferableAmountResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::TransferableAmount {
                token: token.to_string(),
                account: account.to_string(),
            },
        )?;
        Ok(resp.transferable)
    }
}
