use std::alloc::alloc;
use std::collections::HashMap;

use anyhow::{anyhow, Result as AnyResult};
use cosmwasm_std::{from_slice, to_binary, Empty, Response, Uint128};
use cw_multi_test::Contract;

use crate::msg::{CanTransferResp, ControllerQuery};

/// Controller contract stub allowing to easy testing the transfer without actual controller
/// contract
#[derive(Default)]
pub struct Controller {
    /// Mapping for what can be transferred. Map key is an account, the value is how much amount
    /// can be transferred from this account.
    allowances: HashMap<String, Uint128>,
}

impl Controller {
    pub fn new(allowances: impl Into<HashMap<String, Uint128>>) -> Self {
        Self {
            allowances: allowances.into(),
        }
    }

    fn can_transfer(&self, account: &String, amount: Uint128) -> CanTransferResp {
        match self.allowances.get(account) {
            None => CanTransferResp::None,
            Some(allowed) if allowed == &Uint128::zero() => CanTransferResp::None,
            Some(allowed) if allowed >= &amount => CanTransferResp::Whole,
            Some(allowed) => CanTransferResp::Partial(allowed.clone()),
        }
    }
}

impl Contract<Empty> for Controller {
    fn instantiate(
        &self,
        _deps: cosmwasm_std::DepsMut,
        _env: cosmwasm_std::Env,
        _info: cosmwasm_std::MessageInfo,
        _msg: Vec<u8>,
    ) -> anyhow::Result<cosmwasm_std::Response<Empty>> {
        Ok(Response::default())
    }

    fn execute(
        &self,
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        Err(anyhow!("Controller stub execution"))
    }

    fn query(
        &self,
        deps: cosmwasm_std::Deps,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> anyhow::Result<cosmwasm_std::Binary> {
        use ControllerQuery::*;

        let msg: ControllerQuery = from_slice(&msg)?;

        match msg {
            CanTransfer {
                account, amount, ..
            } => to_binary(&self.can_transfer(&account, amount)).map_err(Into::into),
        }
    }

    fn sudo(
        &self,
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> anyhow::Result<cosmwasm_std::Response<Empty>> {
        Err(anyhow!("Controller stub sudo"))
    }

    fn migrate(
        &self,
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        Err(anyhow!("Controller stub migrate"))
    }

    fn reply(
        &self,
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        msg: cosmwasm_std::Reply,
    ) -> anyhow::Result<cosmwasm_std::Response<Empty>> {
        Err(anyhow!("Controller stub reply"))
    }
}
