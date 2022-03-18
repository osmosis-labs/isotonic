use std::collections::HashMap;

use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, Coin, Decimal, StdResult};
use cw_multi_test::{Contract, ContractWrapper, Executor};
use derivative::Derivative;
use osmo_bindings::{OsmosisMsg, OsmosisQuery};
use osmo_bindings_test::{OsmosisApp, Pool};

use crate::msg::*;

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
    pools: HashMap<u64, (Coin, Coin)>,
}

impl SuiteBuilder {
    pub fn with_pool(mut self, id: u64, pool: (Coin, Coin)) -> Self {
        self.pools.insert(id, pool);
        self
    }

    pub fn build(self) -> Suite {
        fn set_up_pools(
            app: &mut OsmosisApp,
            pools: HashMap<u64, (Coin, Coin)>,
            controller: &Addr,
            oracle: &Addr,
        ) -> AnyResult<()> {
            app.init_modules(|router, _, storage| -> StdResult<()> {
                for (pool_id, (coin1, coin2)) in pools.clone() {
                    router
                        .custom
                        .set_pool(storage, pool_id, &Pool::new(coin1, coin2))?;
                }

                Ok(())
            })?;

            for (pool_id, (coin1, coin2)) in pools {
                app.execute_contract(
                    controller.clone(),
                    oracle.clone(),
                    &crate::msg::ExecuteMsg::RegisterPool {
                        pool_id,
                        denom1: coin1.denom,
                        denom2: coin2.denom,
                    },
                    &[],
                )?;
            }

            Ok(())
        }

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

        set_up_pools(&mut app, self.pools, &controller, &oracle_contract).unwrap();

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
    pub fn query_price(&self, sell: &str, buy: &str) -> StdResult<Decimal> {
        let resp: crate::msg::PriceResponse = self.app.wrap().query_wasm_smart(
            self.osmosis_oracle.clone(),
            &QueryMsg::Price {
                sell: sell.to_string(),
                buy: buy.to_string(),
            },
        )?;

        Ok(resp.rate)
    }
}
