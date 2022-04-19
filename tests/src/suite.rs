use anyhow::Result as AnyResult;
use std::collections::HashMap;

use cosmwasm_std::{coin, Addr, Coin, ContractInfoResponse, Decimal};
use cw_multi_test::{AppResponse, Contract, ContractWrapper, Executor};
use isotonic_market::msg::{
    ExecuteMsg as MarketExecuteMsg, MigrateMsg as MarketMigrateMsg, QueryMsg as MarketQueryMsg,
    TokensBalanceResponse,
};
use isotonic_osmosis_oracle::msg::{
    ExecuteMsg as OracleExecuteMsg, InstantiateMsg as OracleInstantiateMsg,
};
use osmo_bindings::{OsmosisMsg, OsmosisQuery};
use osmo_bindings_test::{OsmosisApp, Pool};
use utils::{credit_line::CreditLineResponse, token::Token};

use isotonic_credit_agency::msg::{
    ExecuteMsg, InstantiateMsg, IsOnMarketResponse, ListEnteredMarketsResponse,
    ListMarketsResponse, MarketResponse, QueryMsg, SudoMsg,
};
use isotonic_credit_agency::state::Config;

use crate::MarketBuilder;

pub const COMMON: &str = "COMMON";

fn contract_osmosis_oracle() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new(
        isotonic_osmosis_oracle::contract::execute,
        isotonic_osmosis_oracle::contract::instantiate,
        isotonic_osmosis_oracle::contract::query,
    );

    Box::new(contract)
}

fn contract_credit_agency() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new(
        isotonic_credit_agency::contract::execute,
        isotonic_credit_agency::contract::instantiate,
        isotonic_credit_agency::contract::query,
    )
    .with_sudo(isotonic_credit_agency::contract::sudo)
    .with_reply(isotonic_credit_agency::contract::reply);

    Box::new(contract)
}

pub fn contract_market() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new(
        isotonic_market::contract::execute,
        isotonic_market::contract::instantiate,
        isotonic_market::contract::query,
    )
    .with_reply(isotonic_market::contract::reply)
    .with_migrate(isotonic_market::contract::migrate);

    Box::new(contract)
}

fn contract_token() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
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
    gov_contract: String,
    reward_token: String,
    /// Initial funds to provide for testing
    funds: Vec<(Addr, Vec<Coin>)>,
    liquidation_price: Decimal,
    common_token: String,
    pools: HashMap<u64, (Coin, Coin)>,
    markets: Vec<MarketBuilder>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            gov_contract: "owner".to_string(),
            reward_token: "reward".to_string(),
            funds: vec![],
            liquidation_price: Decimal::percent(92),
            common_token: COMMON.to_owned(),
            pools: HashMap::new(),
            markets: vec![],
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

    pub fn with_pool(mut self, id: u64, pool: (Coin, Coin)) -> Self {
        self.pools.insert(id, pool);
        self
    }

    pub fn with_market(mut self, market: MarketBuilder) -> Self {
        self.markets.push(market);
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let mut app = OsmosisApp::default();
        let owner = Addr::unchecked("owner");
        let common_token = self.common_token.clone();

        let oracle_id = app.store_code(contract_osmosis_oracle());
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

        let isotonic_market_id = app.store_code(contract_market());
        let isotonic_token_id = app.store_code(contract_token());
        let contract_id = app.store_code(contract_credit_agency());
        let credit_agency = app
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

        for market in self.markets {
            app.execute_contract(
                owner.clone(),
                credit_agency.clone(),
                &isotonic_credit_agency::msg::ExecuteMsg::CreateMarket(
                    market.build(oracle_contract.as_str()),
                ),
                &[],
            )
            .unwrap();
        }

        let funds = self.funds;

        app.init_modules(move |router, _, storage| -> AnyResult<()> {
            for (addr, coin) in funds {
                router.bank.init_balance(storage, &addr, coin)?;
            }

            Ok(())
        })
        .unwrap();

        Suite {
            app,
            owner,
            credit_agency,
            common_token: Token::Native(common_token),
            oracle_contract,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: OsmosisApp,
    /// Contract's owner
    owner: Addr,
    /// Address of the Credit Agency contract
    credit_agency: Addr,
    /// Common token
    common_token: Token,
    /// Address of isotonic price oracle
    pub oracle_contract: Addr,
}

impl Suite {
    pub fn app(&mut self) -> &mut OsmosisApp {
        &mut self.app
    }

    pub fn set_pool(&mut self, pools: &[(u64, (Coin, Coin))]) -> AnyResult<()> {
        let owner = self.owner.clone();
        let oracle = self.oracle_contract.clone();

        self.app
            .init_modules(|router, _, storage| -> AnyResult<()> {
                for (pool_id, (coin1, coin2)) in pools {
                    router.custom.set_pool(
                        storage,
                        *pool_id,
                        &Pool::new(coin1.clone(), coin2.clone()),
                    )?;
                }

                Ok(())
            })
            .unwrap();
        for (pool_id, (coin1, coin2)) in pools {
            self.app
                .execute_contract(
                    owner.clone(),
                    oracle.clone(),
                    &OracleExecuteMsg::RegisterPool {
                        pool_id: *pool_id,
                        denom1: coin1.denom.clone(),
                        denom2: coin2.denom.clone(),
                    },
                    &[],
                )
                .unwrap();
        }
        Ok(())
    }

    pub fn advance_seconds(&mut self, seconds: u64) {
        self.app.update_block(|block| {
            block.time = block.time.plus_seconds(seconds);
            block.height += std::cmp::max(1, seconds / 5); // block time
        });
    }

    pub fn enter_market(&mut self, market: &str, addr: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(market),
            self.credit_agency.clone(),
            &ExecuteMsg::EnterMarket {
                account: addr.to_owned(),
            },
            &[],
        )
    }

    pub fn exit_market(&mut self, addr: &str, market: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(addr),
            self.credit_agency.clone(),
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
            .query_wasm_smart(self.credit_agency.clone(), &QueryMsg::Configuration {})?;
        Ok(resp)
    }

    /// Queries the Credit Agency contract for market addr
    pub fn query_market(&self, asset: &str) -> AnyResult<MarketResponse> {
        let resp: MarketResponse = self.app.wrap().query_wasm_smart(
            self.credit_agency.clone(),
            &QueryMsg::Market {
                market_token: Token::Native(asset.to_string()),
            },
        )?;
        Ok(resp)
    }

    /// Queries all markets within agency and returns sum of credit lines
    pub fn query_total_credit_line(&self, account: &str) -> AnyResult<CreditLineResponse> {
        let resp: CreditLineResponse = self.app.wrap().query_wasm_smart(
            self.credit_agency.clone(),
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
            self.credit_agency.clone(),
            &QueryMsg::ListMarkets {
                start_after: start_after.into(),
                limit: limit.into(),
            },
        )?;
        Ok(resp)
    }

    /// Deposit tokens on market selected by denom of Coin
    pub fn deposit(&mut self, account: &str, tokens: Coin) -> AnyResult<AppResponse> {
        let market = self.query_market(tokens.denom.as_str())?;

        self.app.execute_contract(
            Addr::unchecked(account),
            market.market,
            &MarketExecuteMsg::Deposit {},
            &[tokens],
        )
    }

    pub fn withdraw(&mut self, account: &str, tokens: Coin) -> AnyResult<AppResponse> {
        let market = self.query_market(tokens.denom.as_str())?;

        self.app.execute_contract(
            Addr::unchecked(account),
            market.market,
            &MarketExecuteMsg::Withdraw {
                amount: tokens.amount,
            },
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
        let ca = self.credit_agency.clone();

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

    /// Attempts to withdraw the full "withdrawable" amount (as determined by the withdrawable query),
    /// then performs a couple checks to make sure nothing more than that could be withdrawn.
    pub fn attempt_withdraw_max(&mut self, sender: &str, token: &str) -> AnyResult<()> {
        let withdrawable = self.query_withdrawable(sender, token)?;
        self.withdraw(sender, withdrawable)?;

        // double check we cannot withdraw anything above this amount
        self.assert_withdrawable(sender, coin(0, token));
        assert!(self.withdraw(sender, coin(1, token)).is_err());

        Ok(())
    }

    pub fn assert_withdrawable(&self, account: impl ToString, coin: Coin) {
        let withdrawable = self.query_withdrawable(account, &coin.denom).unwrap();
        assert_eq!(withdrawable.amount, coin.amount.into());
    }

    pub fn query_withdrawable(&self, account: impl ToString, denom: &str) -> AnyResult<Coin> {
        let market = self.query_market(denom)?;

        let response: Coin = self.app.wrap().query_wasm_smart(
            market.market,
            &isotonic_market::msg::QueryMsg::Withdrawable {
                account: account.to_string(),
            },
        )?;
        Ok(response)
    }

    pub fn list_entered_markets(
        &self,
        account: &str,
        start_after: impl Into<Option<String>>,
        limit: impl Into<Option<u32>>,
    ) -> AnyResult<Vec<Addr>> {
        let resp: ListEnteredMarketsResponse = self.app.wrap().query_wasm_smart(
            Addr::unchecked(&self.credit_agency),
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
            Addr::unchecked(&self.credit_agency),
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

    pub fn query_price(&self, buy: &str, sell: &str) -> AnyResult<Decimal> {
        let price: isotonic_osmosis_oracle::msg::PriceResponse = self
            .app
            .wrap()
            .query_wasm_smart(
                self.oracle_contract.clone(),
                &isotonic_osmosis_oracle::msg::QueryMsg::Price {
                    sell: sell.to_string(),
                    buy: buy.to_string(),
                },
            )
            .unwrap();

        Ok(price.rate)
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
        let contract = self.credit_agency.clone();
        self.app
            .wasm_sudo(contract, &SudoMsg::AdjustMarketId { new_market_id })
    }

    pub fn sudo_adjust_token_id(&mut self, new_token_id: u64) -> AnyResult<AppResponse> {
        let contract = self.credit_agency.clone();
        self.app
            .wasm_sudo(contract, &SudoMsg::AdjustTokenId { new_token_id })
    }

    pub fn sudo_adjust_common_token(&mut self, new_common_token: &str) -> AnyResult<AppResponse> {
        let contract = self.credit_agency.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::AdjustCommonToken {
                new_common_token: Token::Native(new_common_token.to_owned()),
            },
        )
    }

    pub fn sudo_migrate_market(
        &mut self,
        market: &str,
        migrate_msg: MarketMigrateMsg,
    ) -> AnyResult<AppResponse> {
        let contract = self.credit_agency.clone();
        self.app.wasm_sudo(
            contract,
            &SudoMsg::MigrateMarket {
                contract: market.to_owned(),
                migrate_msg,
            },
        )
    }
}
