use std::collections::HashMap;

use crate::msg::{BalanceResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfoResponse};
use crate::multitest::controller::Controller;
use anyhow::{anyhow, Result as AnyResult};
use cosmwasm_std::{Addr, Binary, Empty, Uint128};
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
    pub fn new() -> Self {
        Self {
            name: "lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            transferable: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_symbol(mut self, symbol: impl ToString) -> Self {
        self.symbol = symbol.to_string();
        self
    }

    pub fn with_decimals(mut self, decimals: u8) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn with_transferable(mut self, sender: impl ToString, amount: Uint128) -> Self {
        *self.transferable.entry(sender.to_string()).or_default() += amount;
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
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
            controller,
            lendex,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
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

    /// Gives controller address back
    pub fn controller(&self) -> Addr {
        self.controller.clone()
    }

    /// Executes transfer on lendex contract
    pub fn transfer(
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
    pub fn send(
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
    pub fn mint(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
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
    pub fn burn(&mut self, sender: &str, amount: Uint128) -> AnyResult<AppResponse> {
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
    pub fn query_balance(&self, address: &str) -> AnyResult<Uint128> {
        let resp: BalanceResponse = self.app.wrap().query_wasm_smart(
            self.lendex.clone(),
            &QueryMsg::Balance {
                address: address.to_owned(),
            },
        )?;
        Ok(resp.balance)
    }

    /// Queries lendex contract for token info
    pub fn query_token_info(&self) -> AnyResult<TokenInfoResponse> {
        self.app
            .wrap()
            .query_wasm_smart(self.lendex.clone(), &QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }
}
