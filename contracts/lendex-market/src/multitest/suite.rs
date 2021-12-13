use anyhow::Result as AnyResult;

use cosmwasm_std::{Addr, Coin, Empty, StdResult, Uint128};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TransferableAmountResponse};
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
    /// Initial funds to provide for testing
    funds: Vec<(Addr, Vec<Coin>)>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            name: "lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            base_asset: "native_denom".to_owned(),
            funds: vec![],
        }
    }

    pub fn with_base_asset(mut self, denom: impl Into<String>) -> Self {
        self.base_asset = denom.into();
        self
    }

    /// Sets initial amount of distributable tokens on address
    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let funds = self.funds;

        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (addr, coin) in funds {
                router.bank.init_balance(storage, &addr, coin)?;
            }

            Ok(())
        })
        .unwrap();

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
                    base_asset: self.base_asset.clone(),
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
            base_asset: self.base_asset,
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
    /// The base asset deposited and lended by the contract
    base_asset: String,
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

    /// Deposit base asset in the lending pool and mint l-token
    pub fn deposit(&mut self, sender: &str, funds: &[Coin]) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Deposit {},
            funds,
        )
    }

    /// Withdraw base asset from the lending pool and burn l-token
    pub fn withdraw(&mut self, sender: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Withdraw {
                amount: amount.into(),
            },
            &[],
        )
    }

    /// Shortcut for querying base asset balance in the market contract
    pub fn query_asset_balance(&self, owner: &str) -> StdResult<u128> {
        let amount = self
            .app
            .wrap()
            .query_balance(owner, &self.base_asset)?
            .amount;
        Ok(amount.into())
    }

    /// Shortcut for querying base asset balance in the market contract
    pub fn query_contract_asset_balance(&self) -> StdResult<u128> {
        self.query_asset_balance(self.contract.as_str())
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

    pub fn query_ltoken_balance(&self, account: impl ToString) -> AnyResult<Uint128> {
        self.query_transferable_amount(&self.ltoken_contract, account)
    }
}
