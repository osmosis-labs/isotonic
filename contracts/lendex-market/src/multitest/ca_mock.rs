use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};
use cw_multi_test::{Contract, ContractWrapper};
use cw_storage_plus::Map;

use crate::msg::CreditLineResponse;

pub const CLR: Map<&Addr, CreditLineResponse> = Map::new("clr");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiateMsg {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetCreditLine {
        collateral: Option<Uint128>,
        credit_line: Option<Uint128>,
        debt: Option<Uint128>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    TotalCreditLine { account: String },
}

fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    Ok(Response::default())
}

fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::SetCreditLine {
            collateral,
            credit_line,
            debt,
        } => {
            dbg!("Execute scl: {}", info.sender.clone());
            CLR.update(deps.storage, &info.sender, |old| -> StdResult<_> {
                dbg!("update!");
                let clr = match old {
                    Some(clr) => CreditLineResponse {
                        collateral: collateral.unwrap_or(clr.collateral),
                        credit_line: credit_line.unwrap_or(clr.credit_line),
                        debt: debt.unwrap_or(clr.debt),
                    },
                    None => CreditLineResponse {
                        collateral: collateral.unwrap_or_else(Uint128::zero),
                        credit_line: credit_line.unwrap_or_else(Uint128::zero),
                        debt: debt.unwrap_or_else(Uint128::zero),
                    },
                };
                dbg!(clr.clone());
                Ok(clr)
            })?;
        }
    }

    Ok(Response::new())
}

fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, StdError> {
    match msg {
        QueryMsg::TotalCreditLine { account } => {
            dbg!("Query scl: {}", account.clone());
            to_binary(&CLR.load(deps.storage, &Addr::unchecked(account))?)
        }
    }
}

pub fn contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}
