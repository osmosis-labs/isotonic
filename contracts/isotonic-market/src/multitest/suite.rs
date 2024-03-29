use anyhow::{anyhow, Result as AnyResult};
use std::collections::HashMap;
use utils::price::PriceRate;

use cosmwasm_std::{Addr, Coin, Decimal, StdResult, Uint128};
use cw20::BalanceResponse;
use cw_multi_test::{AppResponse, Contract, ContractWrapper, Executor};
use isotonic_osmosis_oracle::msg::{
    ExecuteMsg as OracleExecuteMsg, InstantiateMsg as OracleInstantiateMsg,
};
use osmo_bindings::{OsmosisMsg, OsmosisQuery};
use osmo_bindings_test::{OsmosisApp, Pool};
use utils::{
    credit_line::{CreditLineResponse, CreditLineValues},
    interest::Interest,
    token::Token,
};

use super::ca_mock::{
    self, contract as contract_credit_agency, ExecuteMsg as CAExecuteMsg,
    InstantiateMsg as CAInstantiateMsg,
};
use crate::msg::{
    ApyResponse, ExecuteMsg, InstantiateMsg, InterestResponse, MigrateMsg, QueryMsg,
    ReserveResponse, SudoMsg, TokensBalanceResponse, TransferableAmountResponse,
};
use crate::state::Config;

pub const COMMON: &str = "COMMON";

fn contract_oracle() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new(
        isotonic_osmosis_oracle::contract::execute,
        isotonic_osmosis_oracle::contract::instantiate,
        isotonic_osmosis_oracle::contract::query,
    );

    Box::new(contract)
}

pub fn contract_market() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_sudo(crate::contract::sudo)
    .with_reply(crate::contract::reply)
    .with_migrate(crate::contract::migrate);

    Box::new(contract)
}

pub fn contract_token() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new_with_empty(
        isotonic_token::contract::execute,
        isotonic_token::contract::instantiate,
        isotonic_token::contract::query,
    );

    Box::new(contract)
}

/// Builder for test suite
#[derive(Debug)]
pub struct SuiteBuilder {
    /// Isotonic token name
    name: String,
    /// Isotonic token symbol
    symbol: String,
    /// Isotonic token precision
    decimals: u8,
    /// Native denom for the base asset
    market_token: String,
    /// An optional cap on total number of tokens deposited into the market
    cap: Option<Uint128>,
    /// Initial funds to provide for testing
    funds: Vec<(Addr, Vec<Coin>)>,
    /// Initial CA funds
    ca_funds: Vec<Coin>,
    /// Initial funds stored on contract
    contract_funds: Option<Coin>,
    /// Initial interest rate
    interest_base: Decimal,
    /// Initial interest slope
    interest_slope: Decimal,
    /// Interest charge period (in seconds)
    interest_charge_period: u64,
    /// Common Token denom that comes from Credit Agency (same for all markets)
    common_token: String,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    collateral_ratio: Decimal,
    /// Defines the portion of borrower interest that is converted into reserves (0 <= x <= 1)
    reserve_factor: Decimal,
    pools: HashMap<u64, (Coin, Coin)>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            name: "isotonic".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            market_token: "native_denom".to_owned(),
            cap: None,
            funds: vec![],
            ca_funds: vec![],
            contract_funds: None,
            interest_base: Decimal::percent(3),
            interest_slope: Decimal::percent(20),
            interest_charge_period: 300,
            common_token: COMMON.to_owned(),
            collateral_ratio: Decimal::percent(50),
            reserve_factor: Decimal::percent(0),
            pools: HashMap::new(),
        }
    }

    pub fn with_market_token(mut self, denom: impl Into<String>) -> Self {
        self.market_token = denom.into();
        self
    }

    pub fn with_charge_period(mut self, charge_period: u64) -> Self {
        self.interest_charge_period = charge_period;
        self
    }

    pub fn with_common_token(mut self, denom: impl Into<String>) -> Self {
        self.common_token = denom.into();
        self
    }

    pub fn with_cap(mut self, cap: impl Into<Uint128>) -> Self {
        self.cap = Some(cap.into());
        self
    }

    /// Sets initial amount of distributable tokens on address
    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    /// Sets initial amount of distributable tokens on credit agency
    pub fn with_ca_funds(mut self, funds: &[Coin]) -> Self {
        self.ca_funds.extend(funds.iter().cloned());
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

    /// Sets initial collateral ratio
    pub fn with_collateral_ratio(mut self, collateral_ratio: Decimal) -> Self {
        self.collateral_ratio = collateral_ratio;
        self
    }

    pub fn with_reserve_factor(mut self, reserve_factor: u64) -> Self {
        self.reserve_factor = Decimal::percent(reserve_factor);
        self
    }

    pub fn with_pool(mut self, id: u64, pool: (Coin, Coin)) -> Self {
        self.pools.insert(id, pool);
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = OsmosisApp::default();
        let owner = Addr::unchecked("owner");

        let market_token = Token::Native(self.market_token.clone());
        let common_token = Token::Native(self.common_token.clone());

        let oracle_id = app.store_code(contract_oracle());
        let oracle_contract = app
            .instantiate_contract(
                oracle_id,
                owner.clone(),
                &OracleInstantiateMsg {
                    controller: owner.to_string(),
                },
                &[],
                "oracle",
                Some(owner.to_string()),
            )
            .unwrap();

        // initialize the pools for osmosis oracle
        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (pool_id, (coin1, coin2)) in self.pools.clone() {
                router
                    .custom
                    .set_pool(storage, pool_id, &Pool::new(coin1, coin2))?;
            }

            Ok(())
        })
        .unwrap();
        for (pool_id, (coin1, coin2)) in self.pools {
            app.execute_contract(
                owner.clone(),
                oracle_contract.clone(),
                &OracleExecuteMsg::RegisterPool {
                    pool_id,
                    denom1: coin1.denom,
                    denom2: coin2.denom,
                },
                &[],
            )
            .unwrap();
        }

        let ca_id = app.store_code(contract_credit_agency());
        let ca_contract = app
            .instantiate_contract(
                ca_id,
                owner.clone(),
                &CAInstantiateMsg {},
                &[],
                "credit-agency",
                Some(owner.to_string()),
            )
            .unwrap();

        let token_id = app.store_code(contract_token());
        let contract_id = app.store_code(contract_market());
        let contract = app
            .instantiate_contract(
                contract_id,
                // set credit agency mock as owner of market
                ca_contract.clone(),
                &InstantiateMsg {
                    name: self.name,
                    symbol: self.symbol,
                    decimals: self.decimals,
                    token_id,
                    market_token: market_token.clone(),
                    market_cap: self.cap,
                    interest_rate: Interest::Linear {
                        base: self.interest_base,
                        slope: self.interest_slope,
                    },
                    distributed_token: Token::Native("osmo".to_owned()),
                    interest_charge_period: self.interest_charge_period,
                    common_token: common_token.clone(),
                    collateral_ratio: self.collateral_ratio,
                    price_oracle: oracle_contract.to_string(),
                    reserve_factor: self.reserve_factor,
                },
                &[],
                "market",
                Some(owner.to_string()),
            )
            .unwrap();

        let funds = self.funds;
        let ca_funds = self.ca_funds;
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
            router.bank.init_balance(storage, &ca_contract, ca_funds)?;

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
            owner,
            contract,
            ltoken_contract: config.ltoken_contract,
            btoken_contract: config.btoken_contract,
            market_token,
            common_token,
            ca_contract,
            collateral_ratio: self.collateral_ratio,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: OsmosisApp,
    owner: Addr,
    /// Address of Market contract
    contract: Addr,
    /// Address of LToken contract
    ltoken_contract: Addr,
    /// Address of BToken contract
    btoken_contract: Addr,
    /// The market's token denom deposited and lended by the contract
    market_token: Token,
    /// Credit agency token's common denom (with other markets)
    common_token: Token,
    /// Credit Agency contract address
    ca_contract: Addr,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    collateral_ratio: Decimal,
}

impl Suite {
    pub fn app(&mut self) -> &mut OsmosisApp {
        &mut self.app
    }

    pub fn credit_agency(&self) -> String {
        self.ca_contract.to_string()
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

    /// The denom of the common token
    pub fn common_token(&self) -> Token {
        self.common_token.clone()
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
                amount: Uint128::from(amount),
            },
            &[],
        )
    }

    /// Attempts to withdraw the full "withdrawable" amount (as determined by the withdrawable query),
    /// then performs a couple checks to make sure nothing more than that could be withdrawn.
    pub fn attempt_withdraw_max(&mut self, sender: &str) -> AnyResult<()> {
        let withdrawable = self.query_withdrawable(sender)?;
        let withdrawable_in_common =
            withdrawable.amount * self.query_price_market_per_common()?.rate_sell_per_buy;
        self.withdraw(sender, withdrawable.amount.u128())?;

        // mock the change in credit line
        let mut crl = self
            .query_total_credit_line(sender)?
            .validate(&self.common_token())?;
        crl.collateral = crl.collateral.saturating_sub(withdrawable_in_common);
        crl.credit_line = crl
            .credit_line
            .saturating_sub(withdrawable_in_common * self.collateral_ratio);
        self.set_credit_line(sender, crl)?;

        // double check we cannot withdraw anything above this amount
        self.assert_withdrawable(sender, 0);
        assert!(self.withdraw(sender, 1).is_err());

        Ok(())
    }

    /// Borrow base asset from the lending pool and mint b-token
    pub fn borrow(&mut self, sender: &str, amount: u128) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Borrow {
                amount: Uint128::from(amount),
            },
            &[],
        )
    }

    /// Attempts to borrow the full "borrowable" amount (as determined by the borrowable query),
    /// then performs a couple checks to make sure nothing more than that could be borrowed.
    pub fn attempt_borrow_max(&mut self, sender: &str) -> AnyResult<()> {
        let borrowable = self.query_borrowable(sender)?;
        let borrowable_in_common =
            borrowable.amount * self.query_price_market_per_common()?.rate_sell_per_buy;
        self.borrow(sender, borrowable.amount.u128())?;

        // mock the change in credit line
        let mut crl = self
            .query_total_credit_line(sender)?
            .validate(&self.common_token())?;
        crl.debt += borrowable_in_common;
        self.set_credit_line(sender, crl)?;

        // double check we cannot borrow anything above this amount
        self.assert_borrowable(sender, 0);
        assert!(self.borrow(sender, 1).is_err());

        Ok(())
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

    pub fn swap_withdraw_from(
        &mut self,
        sender: impl Into<String>,
        account: impl Into<String>,
        sell_limit: Uint128,
        buy: utils::coin::Coin,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::SwapWithdrawFrom {
                account: account.into(),
                sell_limit,
                buy,
            },
            &[],
        )
    }

    pub fn distribute_as_ltokens(&mut self, sender: &str, funds: Coin) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::DistributeAsLTokens {},
            &[funds],
        )
    }

    pub fn adjust_common_token(&mut self, sender: &str, new_token: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::AdjustCommonToken {
                new_token: Token::Native(new_token.to_owned()),
            },
            &[],
        )
    }

    /// Shortcut for querying base asset balance in the market contract
    pub fn query_asset_balance(&self, owner: &str) -> StdResult<u128> {
        let amount = self
            .app
            .wrap()
            .query_balance(owner, self.market_token.clone().native().unwrap())?
            .amount;
        Ok(amount.into())
    }

    /// Shortcut for querying base asset balance in the market contract
    pub fn query_contract_asset_balance(&self) -> StdResult<u128> {
        self.query_asset_balance(self.contract.as_str())
    }

    pub fn query_transferable_amount(
        &self,
        token: impl ToString,
        account: impl ToString,
    ) -> AnyResult<TransferableAmountResponse> {
        let resp: TransferableAmountResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::TransferableAmount {
                token: token.to_string(),
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    fn query_token_balance(
        &self,
        contract_address: &Addr,
        address: impl ToString,
    ) -> AnyResult<Uint128> {
        let response: BalanceResponse = self.app.wrap().query_wasm_smart(
            contract_address,
            &isotonic_token::QueryMsg::Balance {
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

    /// Queries current interest and utilisation rates
    pub fn query_credit_line(&self, account: impl ToString) -> AnyResult<CreditLineResponse> {
        let resp: CreditLineResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::CreditLine {
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    /// Queries the total credit line from the mock CA
    pub fn query_total_credit_line(&self, account: impl ToString) -> AnyResult<CreditLineResponse> {
        let resp: CreditLineResponse = self.app.wrap().query_wasm_smart(
            self.credit_agency(),
            &ca_mock::QueryMsg::TotalCreditLine {
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    /// Queries the tokens balance of the account
    pub fn query_tokens_balance(&self, account: impl ToString) -> AnyResult<TokensBalanceResponse> {
        let resp: TokensBalanceResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::TokensBalance {
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    /// Queries btoken contract for token info
    pub fn query_btoken_info(&self) -> AnyResult<isotonic_token::msg::TokenInfoResponse> {
        let btoken = self.btoken_contract.clone();
        self.app
            .wrap()
            .query_wasm_smart(btoken, &isotonic_token::msg::QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }

    /// Queries ltoken contract for token info
    pub fn query_ltoken_info(&self) -> AnyResult<isotonic_token::msg::TokenInfoResponse> {
        let ltoken = self.ltoken_contract.clone();
        self.app
            .wrap()
            .query_wasm_smart(ltoken, &isotonic_token::msg::QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }

    /// Queries for APY
    pub fn query_apy(&self) -> AnyResult<ApyResponse> {
        self.app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Apy {})
            .map_err(|err| anyhow!(err))
    }

    /// Sets TotalCreditLine response for CA mock
    pub fn set_credit_line(
        &mut self,
        account: impl ToString,
        credit_line: CreditLineValues,
    ) -> AnyResult<AppResponse> {
        let common_token = self.common_token();
        self.app.execute_contract(
            Addr::unchecked(account.to_string()),
            self.ca_contract.clone(),
            &CAExecuteMsg::SetCreditLine {
                credit_line: credit_line.make_response(common_token),
            },
            &[],
        )
    }

    /// Sets TotalCreditLine with arbitrary high credit line and no debt
    pub fn set_high_credit_line(&mut self, account: impl ToString) -> AnyResult<AppResponse> {
        self.set_credit_line(
            account,
            CreditLineValues {
                collateral: Uint128::new(10_000_000_000_000_000_000),
                credit_line: Uint128::new(10_000_000_000_000_000_000),
                debt: Uint128::zero(),
            },
        )
    }

    /// Queries reserves
    pub fn query_reserve(&self) -> AnyResult<Uint128> {
        let response: ReserveResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Reserve {})?;
        Ok(response.reserve)
    }

    pub fn query_config(&self) -> AnyResult<Config> {
        let response: Config = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Configuration {})?;
        Ok(response)
    }

    pub fn query_withdrawable(&self, account: impl ToString) -> AnyResult<Coin> {
        let response: Coin = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Withdrawable {
                account: account.to_string(),
            },
        )?;
        Ok(response)
    }

    pub fn query_borrowable(&self, account: impl ToString) -> AnyResult<Coin> {
        let response: Coin = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Borrowable {
                account: account.to_string(),
            },
        )?;
        Ok(response)
    }

    /// Queries the tokens balance of the account
    pub fn query_price_market_per_common(&self) -> AnyResult<PriceRate> {
        let resp: PriceRate = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::PriceMarketLocalPerCommon {},
        )?;
        Ok(resp)
    }

    /// Migrates the contract, possibly changing some cfg values via MigrateMsg.
    pub fn migrate(&mut self, new_code_id: u64, msg: &MigrateMsg) -> AnyResult<AppResponse> {
        let owner = self.owner.clone();
        self.app
            .migrate_contract(owner, self.contract.clone(), msg, new_code_id)
    }

    /// Changes collateral ratio parmeter in config through sudo. Pass new ratio as percentage.
    pub fn sudo_adjust_collateral_ratio(&mut self, new_ratio: u64) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::AdjustCollateralRatio {
                new_ratio: Decimal::percent(new_ratio),
            },
        )
    }

    /// Changes reserve factor parmeter in config through sudo. Pass new ratio as percentage.
    pub fn sudo_adjust_reserve_factor(&mut self, new_factor: u64) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::AdjustReserveFactor {
                new_factor: Decimal::percent(new_factor),
            },
        )
    }

    pub fn sudo_adjust_price_oracle(&mut self, new_oracle: &str) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::AdjustPriceOracle {
                new_oracle: new_oracle.to_owned(),
            },
        )
    }

    pub fn sudo_adjust_market_cap(
        &mut self,
        new_cap: impl Into<Option<Uint128>>,
    ) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::AdjustMarketCap {
                new_cap: new_cap.into(),
            },
        )
    }

    pub fn sudo_adjust_interest_rates(
        &mut self,
        new_interest_rates: Interest,
    ) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::AdjustInterestRates { new_interest_rates },
        )
    }

    pub fn assert_ltoken_balance(&self, account: impl ToString, amount: impl Into<Uint128>) {
        let balance = self.query_tokens_balance(account).unwrap();
        assert_eq!(balance.ltokens, amount.into());
    }

    pub fn assert_btoken_balance(&self, account: impl ToString, amount: impl Into<Uint128>) {
        let balance = self.query_tokens_balance(account).unwrap();
        assert_eq!(balance.btokens, amount.into());
    }

    pub fn assert_debt(&self, account: impl ToString, amount: u128) {
        let crl = self.query_credit_line(account).unwrap();
        assert_eq!(crl.debt.amount, Uint128::from(amount));
    }

    pub fn assert_collateral(&self, account: impl ToString, amount: u128) {
        let crl = self.query_credit_line(account).unwrap();
        assert_eq!(crl.collateral.amount, Uint128::from(amount));
    }

    pub fn assert_withdrawable(&self, account: impl ToString, amount: u128) {
        let withdrawable = self.query_withdrawable(account).unwrap();
        assert_eq!(withdrawable.amount, Uint128::from(amount));
    }

    pub fn assert_borrowable(&self, account: impl ToString, amount: u128) {
        let borrowable = self.query_borrowable(account).unwrap();
        assert_eq!(borrowable.amount, Uint128::from(amount));
    }
}
