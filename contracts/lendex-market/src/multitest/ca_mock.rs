use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Uint128, Binary, Empty, Deps, DepsMut, Env, Response, MessageInfo, StdError};
use cw_multi_test::{Contract, ContractWrapper};
use cw_storage_plus::Map;

use crate::msg::CreditLineResponse;

pub const CLR: Map<&String, CreditLineResponse> = Map::new("clr");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstantiateMsg {
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    Ok(Response::default())
}

fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::SetCreditLine { collateral, credit_line, debt } => {
            let credit_line = CreditLineResponse {
                collateral: collateral.unwrap_or(clr.collateral),
                credit_line: credit_line.unwrap_or(clr.credit_line),
                debt: debt.unwrap_or(clr.debt),
            };
            CLR.update(deps.storage, &credit_line)?;
        }
    }

    Ok(Response::new())
}

fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, StdError> {
    match msg {
        QueryMsg::TotalCreditLine { .. } => {
            to_binary(&CLR.load(deps.storage)?)
        }
    }
}

pub fn contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}
