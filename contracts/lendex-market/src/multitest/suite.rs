use anyhow::{anyhow, Result as AnyResult};

use cosmwasm_std::{Addr, Coin, Decimal, Empty, StdResult, Uint128};
use cw20::BalanceResponse;
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use utils::interest::Interest;

use crate::msg::{
    ExecuteMsg, InstantiateMsg, InterestResponse, QueryMsg, TransferableAmountResponse,
};
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
    /// Initial funds stored on contract
    contract_funds: Option<Coin>,
    /// Initial interest rate
    interest_base: Decimal,
    /// Initial interest slope
    interest_slope: Decimal,
    /// Interest charge period (in seconds)
    interest_charge_period: u64,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            name: "lendex".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            base_asset: "native_denom".to_owned(),
            funds: vec![],
            contract_funds: None,
            interest_base: Decimal::percent(3),
            interest_slope: Decimal::percent(20),
            interest_charge_period: 300,
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

    /// Sets initial amount of distributable tokens on address
    pub fn with_contract_funds(mut self, funds: Coin) -> Self {
        self.contract_funds = Some(funds);
        self
    }

    /// Sets initial interest base and slope (in percentage)
    pub fn with_interest(mut self, base: u64, slope: u64) -> Self {
        self.interest_base = Decimal::percent(base);
        self.interest_slope = Decimal::percent(slope);
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");

        let base_asset = self.base_asset;

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
                    base_asset: base_asset.clone(),
                    interest_rate: Interest::Linear {
                        base: self.interest_base,
                        slope: self.interest_slope,
                    },
                    distributed_token: "osmo".to_owned(),
                    interest_charge_period: self.interest_charge_period,
                },
                &[],
                "market",
                Some(owner.to_string()),
            )
            .unwrap();

        let funds = self.funds;
        let contract_funds = self.contract_funds;

        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (addr, coin) in funds {
                router.bank.init_balance(storage, &addr, coin)?;
            }
            if let Some(contract_funds) = contract_funds {
                // initialize contract's balance as well
                router
                    .bank
                    .init_balance(storage, &contract, vec![contract_funds])?;
            }

            Ok(())
        })
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
            base_asset,
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
    pub fn app(&mut self) -> &mut App {
        &mut self.app
    }

    pub fn advance_seconds(&mut self, seconds: u64) {
        self.app.update_block(|block| {
            block.time = block.time.plus_seconds(seconds);
            block.height += std::cmp::max(1, seconds / 5); // block time
        });
    }

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

    /// Borrow base asset from the lending pool and mint b-token
    pub fn borrow(&mut self, sender: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Borrow {
                amount: amount.into(),
            },
            &[],
        )
    }

    /// Repay borrowed tokens from the lending pool and burn b-token
    pub fn repay(&mut self, sender: &str, funds: Coin) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Repay {},
            &[funds],
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

    fn query_token_balance(
        &self,
        contract_address: &Addr,
        address: impl ToString,
    ) -> AnyResult<Uint128> {
        let response: BalanceResponse = self.app.wrap().query_wasm_smart(
            contract_address,
            &lendex_token::QueryMsg::Balance {
                address: address.to_string(),
            },
        )?;
        Ok(response.balance)
    }

    /// Queries ltoken contract for balance
    pub fn query_ltoken_balance(&self, account: impl ToString) -> AnyResult<Uint128> {
        self.query_token_balance(&self.ltoken_contract, account)
    }

    /// Queries btoken contract for balance
    pub fn query_btoken_balance(&self, account: impl ToString) -> AnyResult<Uint128> {
        self.query_token_balance(&self.btoken_contract, account)
    }

    /// Queries current interest and utilisation rates
    pub fn query_interest(&self) -> AnyResult<InterestResponse> {
        let resp: InterestResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Interest {})?;
        Ok(resp)
    }

    /// Queries btoken contract for token info
    pub fn query_btoken_info(&self) -> AnyResult<lendex_token::msg::TokenInfoResponse> {
        let btoken = self.btoken_contract.clone();
        self.app
            .wrap()
            .query_wasm_smart(btoken, &lendex_token::msg::QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }

    /// Queries ltoken contract for token info
    pub fn query_ltoken_info(&self) -> AnyResult<lendex_token::msg::TokenInfoResponse> {
        let ltoken = self.ltoken_contract.clone();
        self.app
            .wrap()
            .query_wasm_smart(ltoken, &lendex_token::msg::QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }
}
