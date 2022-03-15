use std::collections::HashMap;

use cosmwasm_std::Addr;
use cw_multi_test::{Contract, ContractWrapper, Executor};
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
pub struct SuiteBuilder {}

impl SuiteBuilder {
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
}
