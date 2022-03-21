use std::collections::HashMap;

use crate::display_amount::DisplayAmount;
use crate::msg::{
    BalanceResponse, ExecuteMsg, FundsResponse, InstantiateMsg, MultiplierResponse, QueryMsg,
    TokenInfoResponse,
};
use crate::multitest::controller::Controller;
use crate::multitest::receiver::{QueryResp as ReceiverQueryResp, Receiver};
use anyhow::{anyhow, Result as AnyResult};
use cosmwasm_std::{Addr, Binary, Coin, Decimal, Empty, Uint128};
use cw_multi_test::{App, AppResponse, BasicAppBuilder, Contract, ContractWrapper, Executor};

use utils::token::Token;

fn contract_token() -> Box<dyn Contract<Empty>> {
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
    /// Isotonic token name
    name: String,
    /// Isotonic token symbol
    symbol: String,
    /// Isotonic token precision
    decimals: u8,
    /// Amount of tokens controller would allow to transfer
    transferable: HashMap<String, Uint128>,
    /// Token distributed by this contract
    distributed_token: String,
    /// Initial funds of native tokens
    funds: Vec<(Addr, Vec<Coin>)>,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            name: "isotonic".to_owned(),
            symbol: "LDX".to_owned(),
            decimals: 9,
            transferable: HashMap::new(),
            distributed_token: "gov".to_owned(),
            funds: Vec::new(),
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

    pub fn with_distributed_token(mut self, token: impl ToString) -> Self {
        self.distributed_token = token.to_string();
        self
    }

    pub fn with_funds(mut self, addr: &str, tokens: impl IntoIterator<Item = Coin>) -> Self {
        self.funds
            .push((Addr::unchecked(addr), tokens.into_iter().collect()));
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let funds = self.funds;
        let mut app = BasicAppBuilder::new().build(move |router, _api, storage| {
            for (addr, tokens) in funds {
                router.bank.init_balance(storage, &addr, tokens).unwrap();
            }
        });
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

        let token_id = app.store_code(contract_token());
        let token = app
            .instantiate_contract(
                token_id,
                owner.clone(),
                &InstantiateMsg {
                    name: self.name,
                    symbol: self.symbol,
                    decimals: self.decimals,
                    controller: controller.to_string(),
                    distributed_token: Token::Native(self.distributed_token),
                },
                &[],
                "Isotonic",
                None,
            )
            .unwrap();

        let receiver_id = app.store_code(Box::new(Receiver::new()));
        let receiver = app
            .instantiate_contract(receiver_id, owner, &Empty {}, &[], "Receiver", None)
            .unwrap();

        Suite {
            app,
            controller,
            token,
            receiver,
        }
    }
}

/// Test suite
pub struct Suite {
    /// The multitest app
    app: App,
    /// Address of controller contract
    controller: Addr,
    /// Address of token contract
    token: Addr,
    /// Address of cw1 contract
    receiver: Addr,
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

    /// Gives receiver address back
    pub fn receiver(&self) -> Addr {
        self.receiver.clone()
    }

    /// Gives token address back
    pub fn token(&self) -> Addr {
        self.token.clone()
    }

    /// Executes transfer on token contract
    pub fn transfer(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::Transfer {
                    recipient: recipient.to_owned(),
                    amount: DisplayAmount::raw(amount),
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes send on token contract
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
                self.token.clone(),
                &ExecuteMsg::Send {
                    contract: recipient.to_owned(),
                    amount: DisplayAmount::raw(amount),
                    msg,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes mint on token contract
    pub fn mint(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
    ) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::Mint {
                    recipient: recipient.to_owned(),
                    amount: DisplayAmount::raw(amount),
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes burn on token contract
    pub fn burn(&mut self, sender: &str, account: &str, amount: Uint128) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender),
                self.token.clone(),
                &ExecuteMsg::BurnFrom {
                    owner: account.to_string(),
                    amount: DisplayAmount::raw(amount),
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes rebase on token contract
    pub fn rebase(&mut self, executor: &str, ratio: Decimal) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(executor),
                self.token.clone(),
                &ExecuteMsg::Rebase { ratio },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Executes distribute on token contract
    pub fn distribute<'a>(
        &mut self,
        executor: &str,
        sender: impl Into<Option<&'a str>>,
        funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        let sender = sender.into().map(str::to_owned);
        self.app
            .execute_contract(
                Addr::unchecked(executor),
                self.token.clone(),
                &ExecuteMsg::Distribute { sender },
                funds,
            )
            .map_err(|err| anyhow!(err))
    }

    /// Execute withdraw_funds on token contract
    pub fn withdraw_funds(&mut self, executor: &str) -> AnyResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(executor),
                self.token.clone(),
                &ExecuteMsg::WithdrawFunds {},
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    /// Queries token contract for balance
    pub fn query_balance(&self, address: &str) -> AnyResult<DisplayAmount> {
        let resp: BalanceResponse = self.app.wrap().query_wasm_smart(
            self.token.clone(),
            &QueryMsg::Balance {
                address: address.to_owned(),
            },
        )?;
        Ok(resp.balance)
    }

    /// Queries token contract for token info
    pub fn query_token_info(&self) -> AnyResult<TokenInfoResponse> {
        self.app
            .wrap()
            .query_wasm_smart(self.token.clone(), &QueryMsg::TokenInfo {})
            .map_err(|err| anyhow!(err))
    }

    /// Queries receiver for count of valid messages it received
    pub fn query_receiver(&self) -> AnyResult<u128> {
        let resp: ReceiverQueryResp = self
            .app
            .wrap()
            .query_wasm_smart(self.receiver.clone(), &Empty {})
            .map_err(|err| anyhow!(err))?;

        Ok(resp.counter.into())
    }

    /// Queries multiplier
    pub fn query_multiplier(&self) -> AnyResult<Decimal> {
        let resp: MultiplierResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.token.clone(), &QueryMsg::Multiplier {})
            .map_err(|err| anyhow!(err))?;

        Ok(resp.multiplier)
    }

    /// Queries distributed funds
    pub fn query_distributed_funds(&self) -> AnyResult<Coin> {
        let resp: FundsResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.token.clone(), &QueryMsg::DistributedFunds {})
            .map_err(|err| anyhow!(err))?;

        Ok(resp.funds)
    }

    /// Queries undistributed funds
    pub fn query_undistributed_funds(&self) -> AnyResult<Coin> {
        let resp: FundsResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.token.clone(), &QueryMsg::UndistributedFunds {})
            .map_err(|err| anyhow!(err))?;

        Ok(resp.funds)
    }

    /// Queries withdrawable funds
    pub fn query_withdrawable_funds(&self, addr: &str) -> AnyResult<Coin> {
        let resp: FundsResponse = self
            .app
            .wrap()
            .query_wasm_smart(
                self.token.clone(),
                &QueryMsg::WithdrawableFunds {
                    owner: addr.to_owned(),
                },
            )
            .map_err(|err| anyhow!(err))?;

        Ok(resp.funds)
    }

    /// Queries for balance of native token
    pub fn native_balance(&self, addr: &str, token: &str) -> AnyResult<u128> {
        let amount = self
            .app
            .wrap()
            .query_balance(&Addr::unchecked(addr), token)?
            .amount;
        Ok(amount.into())
    }
}
