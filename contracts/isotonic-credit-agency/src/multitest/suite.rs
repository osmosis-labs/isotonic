use anyhow::Result as AnyResult;

use cosmwasm_std::{Addr, Coin, ContractInfoResponse, Decimal, Empty};
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use isotonic_market::msg::{
    ExecuteMsg as MarketExecuteMsg, MigrateMsg as MarketMigrateMsg, QueryMsg as MarketQueryMsg,
    TokensBalanceResponse,
};
use isotonic_market::state::SECONDS_IN_YEAR;
use isotonic_oracle::msg::ExecuteMsg as OracleExecuteMsg;
use utils::credit_line::CreditLineResponse;
use utils::interest::Interest;
use utils::time::Duration;
use utils::token::Token;

use crate::msg::{
    ExecuteMsg, InstantiateMsg, IsOnMarketResponse, ListEnteredMarketsResponse,
    ListMarketsResponse, MarketConfig, MarketResponse, QueryMsg, SudoMsg,
};
use crate::state::Config;

fn contract_oracle() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        isotonic_oracle::contract::execute,
        isotonic_oracle::contract::instantiate,
        isotonic_oracle::contract::query,
    );

    Box::new(contract)
}

fn contract_credit_agency() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_sudo(crate::contract::sudo)
    .with_reply(crate::contract::reply);

    Box::new(contract)
}

pub fn contract_market() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        isotonic_market::contract::execute,
        isotonic_market::contract::instantiate,
        isotonic_market::contract::query,
    )
    .with_reply(isotonic_market::contract::reply)
    .with_migrate(isotonic_market::contract::migrate);

    Box::new(contract)
}

fn contract_token() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        isotonic_token::contract::execute,
        isotonic_token::contract::instantiate,
        isotonic_token::contract::query,
    );

    Box::new(contract)
}

/// Builder for test suite
#[derive(Debug)]
pub struct SuiteBuilder {
    gov_contract: String,
    reward_token: String,
    /// Initial funds to provide for testing
    funds: Vec<(Addr, Vec<Coin>)>,
    liquidation_price: Decimal,
    common_token: String,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            gov_contract: "owner".to_string(),
            reward_token: "reward".to_string(),
            funds: vec![],
            liquidation_price: Decimal::percent(92),
            common_token: "common".to_owned(),
        }
    }

    pub fn with_gov(mut self, gov: impl ToString) -> Self {
        self.gov_contract = gov.to_string();
        self
    }

    pub fn with_reward_token(mut self, denom: impl ToString) -> Self {
        self.reward_token = denom.to_string();
        self
    }

    /// Sets initial amount of distributable tokens on address
    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    pub fn with_liquidation_price(mut self, liquidation_price: Decimal) -> Self {
        self.liquidation_price = liquidation_price;
        self
    }

    pub fn with_common_token(mut self, common_token: &str) -> Self {
        self.common_token = common_token.to_owned();
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        let common_token = self.common_token;

        let oracle_id = app.store_code(contract_oracle());
        let oracle_contract = app
            .instantiate_contract(
                oracle_id,
                owner.clone(),
                &isotonic_oracle::msg::InstantiateMsg {
                    oracle: owner.to_string(),
                    maximum_age: Duration::new(999999999),
                },
                &[],
                "oracle",
                Some(owner.to_string()),
            )
            .unwrap();

        let isotonic_market_id = app.store_code(contract_market());
        let isotonic_token_id = app.store_code(contract_token());
        let contract_id = app.store_code(contract_credit_agency());
        let contract = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    gov_contract: self.gov_contract,
                    isotonic_market_id,
                    isotonic_token_id,
                    reward_token: Token::Native(self.reward_token),
                    common_token: Token::Native(common_token.clone()),
                    liquidation_price: self.liquidation_price,
                },
                &[],
                "credit-agency",
                Some(owner.to_string()),
            )
            .unwrap();

        let funds = self.funds;

        app.init_modules(|router, _, storage| -> AnyResult<()> {
            for (addr, coin) in funds {
                router.bank.init_balance(storage, &addr, coin)?;
            }
            Ok(())
        })
        .unwrap();

        Suite {
            app,
            owner,
            contract,
            common_token: Token::Native(common_token),
            oracle_contract,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Contract's owner
    owner: Addr,
    /// Address of the Credit Agency contract
    contract: Addr,
    /// Common token
    common_token: Token,
    /// Address of isotonic price oracle
    pub oracle_contract: Addr,
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
        isotonic_token: &str,
        market_token: &str,
        collateral_ratio: impl Into<Option<Decimal>>,
        interest_rates: impl Into<Option<(Decimal, Decimal)>>,
        reserve_factor: impl Into<Option<Decimal>>,
    ) -> AnyResult<AppResponse> {
        self.create_market(
            caller,
            MarketConfig {
                name: isotonic_token.to_string(),
                symbol: isotonic_token.to_string(),
                decimals: 9,
                market_token: Token::Native(market_token.to_string()),
                market_cap: None,
                interest_rate: match interest_rates.into() {
                    Some((base, slope)) => Interest::Linear { base, slope },
                    None => Interest::Linear {
                        base: Decimal::percent(3),
                        slope: Decimal::percent(20),
                    },
                },
                interest_charge_period: SECONDS_IN_YEAR as u64, // seconds
                collateral_ratio: collateral_ratio
                    .into()
                    .unwrap_or_else(|| Decimal::percent(50)),
                price_oracle: self.oracle_contract.to_string(),
                reserve_factor: reserve_factor.into().unwrap_or_else(|| Decimal::percent(0)),
            },
        )
    }

    pub fn enter_market(&mut self, market: &str, addr: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(market),
            self.contract.clone(),
            &ExecuteMsg::EnterMarket {
                account: addr.to_owned(),
            },
            &[],
        )
    }

    pub fn exit_market(&mut self, addr: &str, market: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(addr),
            self.contract.clone(),
            &ExecuteMsg::ExitMarket {
                market: market.to_owned(),
            },
            &[],
        )
    }

    pub fn common_token(&self) -> &Token {
        &self.common_token
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
                market_token: Token::Native(asset.to_string()),
            },
        )?;
        Ok(resp)
    }

    /// Queries all markets within agency and returns sum of credit lines
    pub fn query_total_credit_line(&self, account: &str) -> AnyResult<CreditLineResponse> {
        let resp: CreditLineResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::TotalCreditLine {
                account: account.to_string(),
            },
        )?;
        Ok(resp)
    }

    pub fn assert_market(&self, asset: &str) {
        let res = self.query_market(asset).unwrap();
        assert_eq!(res.market_token.native().unwrap(), asset);

        // We query the supposed market contract address to make extra sure
        // it was instantiated properly and exists.
        let resp: isotonic_market::state::Config = self
            .app
            .wrap()
            .query_wasm_smart(
                res.market,
                &isotonic_market::msg::QueryMsg::Configuration {},
            )
            .unwrap();
        assert_eq!(resp.market_token, asset);
    }

    /// Queries the Credit Agency contract for a list of markets with pagination
    pub fn list_markets(&self) -> AnyResult<ListMarketsResponse> {
        self.list_markets_with_pagination(None, None)
    }

    /// Queries the Credit Agency contract for a list of markets with pagination
    pub fn list_markets_with_pagination(
        &self,
        start_after: impl Into<Option<Token>>,
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

    /// Sets sell/buy price (rate) between market_token and common_token
    pub fn oracle_set_price_market_per_common(
        &mut self,
        market: Token,
        rate: Decimal,
    ) -> AnyResult<AppResponse> {
        let owner = self.owner.clone();

        self.app.execute_contract(
            owner,
            self.oracle_contract.clone(),
            &OracleExecuteMsg::SetPrice {
                buy: self.common_token.clone(),
                sell: market,
                rate,
            },
            &[],
        )
    }

    /// Deposit tokens on market selected by denom of Coin
    pub fn deposit_tokens_on_market(
        &mut self,
        account: &str,
        tokens: Coin,
    ) -> AnyResult<AppResponse> {
        let market = self.query_market(tokens.denom.as_str())?;

        self.app.execute_contract(
            Addr::unchecked(account),
            market.market,
            &MarketExecuteMsg::Deposit {},
            &[tokens],
        )
    }

    /// Borrow tokens from market selected by denom and amount of Coin
    pub fn borrow_tokens_from_market(
        &mut self,
        account: &str,
        tokens: Coin,
    ) -> AnyResult<AppResponse> {
        let market = self.query_market(tokens.denom.as_str())?;

        self.app.execute_contract(
            Addr::unchecked(account),
            market.market,
            &MarketExecuteMsg::Borrow {
                amount: tokens.amount,
            },
            &[],
        )
    }

    pub fn liquidate(
        &mut self,
        sender: &str,
        account: &str,
        tokens: &[Coin],
        collateral_denom: Token,
    ) -> AnyResult<AppResponse> {
        let ca = self.contract.clone();

        self.app.execute_contract(
            Addr::unchecked(sender),
            ca,
            &ExecuteMsg::Liquidate {
                account: account.to_owned(),
                collateral_denom,
            },
            tokens,
        )
    }

    pub fn repay_tokens_on_market(
        &mut self,
        account: &str,
        tokens: Coin,
    ) -> AnyResult<AppResponse> {
        let market = self.query_market(tokens.denom.as_str())?;

        self.app.execute_contract(
            Addr::unchecked(account),
            market.market,
            &MarketExecuteMsg::Repay {},
            &[tokens],
        )
    }

    pub fn list_entered_markets(
        &self,
        account: &str,
        start_after: impl Into<Option<String>>,
        limit: impl Into<Option<u32>>,
    ) -> AnyResult<Vec<Addr>> {
        let resp: ListEnteredMarketsResponse = self.app.wrap().query_wasm_smart(
            Addr::unchecked(&self.contract),
            &QueryMsg::ListEnteredMarkets {
                account: account.to_owned(),
                start_after: start_after.into(),
                limit: limit.into(),
            },
        )?;

        Ok(resp.markets)
    }

    pub fn list_all_entered_markets(&self, account: &str) -> AnyResult<Vec<Addr>> {
        self.list_entered_markets(account, None, None)
    }

    pub fn is_on_market(&self, account: &str, market: &str) -> AnyResult<bool> {
        let resp: IsOnMarketResponse = self.app.wrap().query_wasm_smart(
            Addr::unchecked(&self.contract),
            &QueryMsg::IsOnMarket {
                account: account.to_owned(),
                market: market.to_owned(),
            },
        )?;

        Ok(resp.participating)
    }

    /// Queries l/btokens balance on market pointed by denom for given account
    pub fn query_tokens_balance(
        &self,
        market_denom: &str,
        account: &str,
    ) -> AnyResult<TokensBalanceResponse> {
        let market = self.query_market(market_denom)?;
        let resp: TokensBalanceResponse = self.app.wrap().query_wasm_smart(
            market.market,
            &MarketQueryMsg::TokensBalance {
                account: account.to_owned(),
            },
        )?;
        Ok(resp)
    }

    /// Queries configuration from market selected by denom
    pub fn query_market_config(&self, denom: &str) -> AnyResult<isotonic_market::state::Config> {
        let market = self.query_market(denom)?;

        let resp: isotonic_market::state::Config = self
            .app
            .wrap()
            .query_wasm_smart(market.market, &MarketQueryMsg::Configuration {})?;
        Ok(resp)
    }

    pub fn query_contract_code_id(&mut self, contract_denom: &str) -> AnyResult<u64> {
        use cosmwasm_std::{QueryRequest, WasmQuery};
        let market = self.query_market(contract_denom)?;
        let query_result: ContractInfoResponse =
            self.app
                .wrap()
                .query(&QueryRequest::Wasm(WasmQuery::ContractInfo {
                    contract_addr: market.market.to_string(),
                }))?;
        Ok(query_result.code_id)
    }

    pub fn sudo_adjust_market_id(&mut self, new_market_id: u64) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app
            .wasm_sudo(contract, &SudoMsg::AdjustMarketId { new_market_id })
    }

    pub fn sudo_adjust_token_id(&mut self, new_token_id: u64) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app
            .wasm_sudo(contract, &SudoMsg::AdjustTokenId { new_token_id })
    }

    pub fn sudo_adjust_common_token(&mut self, new_common_token: &str) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::AdjustCommonToken {
                new_common_token: new_common_token.to_owned(),
            },
        )
    }

    pub fn sudo_migrate_market(
        &mut self,
        market: &str,
        migrate_msg: MarketMigrateMsg,
    ) -> AnyResult<AppResponse> {
        let contract = self.contract.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::MigrateMarket {
                contract: market.to_owned(),
                migrate_msg,
            },
        )
    }
}
