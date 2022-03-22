This repository is heavily modified, but in principle kept in same manner as https://github.com/cosmos/cosmjs repository with examples from `scripts/wasmd` directory.

Steps to run the tests:

- run `npm install` in main directory
- make sure you have docker daemon started
- go to `scripts` directory and run `./start.sh && ./init.sh`
- when test finishes, you can stop running container by calling `./stop.sh` script

`scripts/deploy_and_instantiate_contracts.js` - this file contains, as name suggests, all steps in order to deploy and instantiate all Isotonic contracts on chain. Wasm files are taken from `scripts/contracts` directory. In the end, those files should be downloaded from artifacts on CI.
Created is:

- Oracle
- Credit Agency with reward_token `ucosm`, common_token `ucosm` and liquidation_price `0.92`,
- two markets, first with denom `ustake`, second with denom `ucosm`. Both have collateral ratio set to `0.7`,
- price oracle has two prices set: `ustake`/`ucosm` at 0.5 and `ucosm`/`ustake` at 2.0

`scripts/market_play.js` - couple basic interactions from user B (user A instantiates the contracts) with two created markets and balance assertion.
