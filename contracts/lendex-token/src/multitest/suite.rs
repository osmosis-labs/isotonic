use std::collections::HashMap;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::multitest::controller::Controller;
use anyhow::{anyhow, Result as AnyResult};
use cosmwasm_std::{Addr, Binary, Empty, Uint128};
use cw20::{BalanceResponse, TokenInfoResponse};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

fn contract_lendex() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
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
    /// Amount of tokens controller would allow to transfer
    transferable: HashMap<String, Uint128>,
}

impl SuiteBuilder {
    fn new() -> Self {
        Self {
            name: "lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            transferable: HashMap::new(),
        }
    }

    fn with_transfeable(mut self, sender: String, amount: Uint128) -> Self {
        *self.transferable.entry(sender).or_default() += amount;
        self
    }

    #[track_caller]
    fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let controller_contract = Controller::new(self.transferable);
        let controller_id = app.store_code(Box::new(controller_contract));
        let controller = app
            .instantiate_contract(
                controller_id,
                owner.clone(),
                &Empty {},
                &[],
                "Controller",
                None,
            )
            .unwrap();

        let lendex_id = app.store_code(contract_lendex());
        let lendex = app
            .instantiate_contract(
                lendex_id,
                owner.clone(),
                &InstantiateMsg {
                    name: self.name,
                    symbol: self.symbol,
                    decimals: self.decimals,
                    controller: controller.to_string(),
                },
                &[],
                "Lendex",
                None,
            )
            .unwrap();

        Suite {
            app,
            owner,
            controller,
            lendex,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Owner address used for admministrative messages
    owner: Addr,
    /// Address of controller contract
    controller: Addr,
    /// Address of lendex contract
    lendex: Addr,
}

impl Suite {
    /// Builds test suite with default configuration
    pub fn new() -> Self {
        SuiteBuilder::new().build()
    }

    /// Executes transfer on lendex contract
    fn transfer(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.lendex.clone(),
                &ExecuteMsg::Transfer {
                    recipient: recipient.to_owned(),
                    amount,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes send on lendex contract
    fn send(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
        msg: Binary,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.lendex.clone(),
                &ExecuteMsg::Send {
                    contract: recipient.to_owned(),
                    amount,
                    msg,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes mint on lendex contract
    fn mint(&mut self, sender: &str, recipient: &str, amount: Uint128) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.lendex.clone(),
                &ExecuteMsg::Mint {
                    recipient: recipient.to_owned(),
                    amount,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes burn on lendex contract
    fn burn(&mut self, sender: &str, amount: Uint128) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.lendex.clone(),
                &ExecuteMsg::Burn { amount },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Queries lendex contract for balance
    fn query_balance(&self, address: &str) -> AnyResult<Uint128> {
        let resp: BalanceResponse = self.app.wrap().query_wasm_smart(
            self.lendex.clone(),
            &QueryMsg::Balance {
                address: address.to_owned(),
            },
        )?;
        Ok(resp.balance)
    }

    /// Queries lendex contract for token info
    fn query_token_info(&self) -> AnyResult<TokenInfoResponse> {
        self.app
            .wrap()
            .query_wasm_smart(self.lendex.clone(), &QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }
}
