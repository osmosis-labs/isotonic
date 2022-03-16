use std::collections::HashMap;

use anyhow::Result as AnyResult;
use cosmwasm_std::Addr;
use cw_multi_test::{AppResponse, Contract, ContractWrapper, Executor};
use derivative::Derivative;
use osmo_bindings::{OsmosisMsg, OsmosisQuery};
use osmo_bindings_test::{OsmosisApp, Pool};

fn contract_osmosis_oracle() -> Box<dyn Contract<OsmosisMsg, OsmosisQuery>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );

    Box::new(contract)
}

#[derive(Derivative, Debug, Clone)]
#[derivative(Default = "new")]
pub struct SuiteBuilder {
    pools: HashMap<u64, Pool>,
}

impl SuiteBuilder {
    pub fn with_pool(mut self, id: u64, pool: Pool) -> Self {
        self.pools.insert(id, pool);
        self
    }

    pub fn build(mut self) -> Suite {
        let controller = Addr::unchecked("admin");

        let mut app = OsmosisApp::new();

        let oracle_code_id = app.store_code(contract_osmosis_oracle());
        let oracle_contract = app
            .instantiate_contract(
                oracle_code_id,
                controller.clone(),
                &crate::msg::InstantiateMsg {
                    controller: controller.to_string(),
                },
                &[],
                "osmosis_oracle",
                Some(controller.to_string()),
            )
            .unwrap();

        Suite {
            controller,
            app,
            osmosis_oracle: oracle_contract,
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Suite {
    controller: Addr,
    #[derivative(Debug = "ignore")]
    app: OsmosisApp,
    osmosis_oracle: Addr,
}

impl Suite {
    /// The controller of the oracle contract, allowed to add a pool to the oracle's list
    pub fn controller(&self) -> &Addr {
        &self.controller
    }

    /// The internal `OsmosisApp`
    pub fn app(&mut self) -> &mut OsmosisApp {
        &mut self.app
    }

    pub fn register_pool(
        &mut self,
        executor: &str,
        pool_id: u64,
        denom1: &str,
        denom2: &str,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(executor),
            self.osmosis_oracle.clone(),
            &crate::msg::ExecuteMsg::RegisterPool {
                pool_id,
                denom1: denom1.to_string(),
                denom2: denom2.to_string(),
            },
            &[],
        )
    }
}
