#!/usr/bin/env node

/* eslint-disable @typescript-eslint/naming-convention */
const { SigningCosmWasmClient, fromBinary } = require("@cosmjs/cosmwasm-stargate");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { calculateFee, GasPrice } = require("@cosmjs/stargate");
const fs = require("fs");

const endpoint = "http://localhost:26659";
const alice = {
  mnemonic: "enlist hip relief stomach skate base shallow young switch frequent cry park",
  address0: "wasm14qemq0vw6y3gc3u3e0aty2e764u4gs5lndxgyk",
};

const oracleInit = {
  label: "From deploy_isotonic_market.js (0)",
  msg: {
    oracle: alice.address0,
    maximum_age: 86400,
  },
  admin: undefined,
};

const creditAgencyInit = {
  label: "From deploy_isotonic_market.js (0)",
  msg: {
    gov_contract: alice.address0,
    isotonic_market_id: 1,
    isotonic_token_id: 2,
    reward_token: { Native: "ucosm" },
    common_token: { Native: "ucosm" },
    liquidation_price: "0.92",
  },
  admin: undefined,
};

const firstMarket = {
  "create_market": {
    "name": "first market",
    "symbol": "FST",
    "decimals": 5,
    "token_id": 2,
    "market_token": { Native: "ustake" },
    "interest_rate": {
      "linear": {
        "base": "0.04",
        "slope": "0.2"
      }
    },
    "interest_charge_period": 3600,
    "collateral_ratio": "0.7",
    "price_oracle": "wasm1hrpna9v7vs3stzyd4z3xf00676kf78zpe2u5ksvljswn2vnjp3ys8c5wp9",
    "reserve_factor": "0.1"
  }
};

const secondMarket = {
  "create_market": {
    "name": "second market",
    "symbol": "SND",
    "decimals": 5,
    "token_id": 2,
    "market_token": { Native: "ucosm" },
    "interest_rate": {
      "linear": {
        "base": "0.04",
        "slope": "0.2"
      }
    },
    "interest_charge_period": 3600,
    "collateral_ratio": "0.7",
    "price_oracle": "wasm1hrpna9v7vs3stzyd4z3xf00676kf78zpe2u5ksvljswn2vnjp3ys8c5wp9",
    "reserve_factor": "0.1"
  }
};

async function main() {
  const gasPrice = GasPrice.fromString("0.025ucosm");
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(alice.mnemonic, { prefix: "wasm" });
  const client = await SigningCosmWasmClient.connectWithSigner(endpoint, wallet);

  var wasm = fs.readFileSync(__dirname + "/contracts/isotonic_market.wasm");
  const uploadFee = calculateFee(2_500_000, gasPrice);
  const uploadReceiptMarket = await client.upload(
    alice.address0,
    wasm,
    uploadFee,
    "Upload isotonic-market contract",
  );
  console.info(`Market upload succeeded. Receipt: ${JSON.stringify(uploadReceiptMarket)}`);

  var wasm = fs.readFileSync(__dirname + "/contracts/isotonic_token.wasm");
  const uploadReceiptToken = await client.upload(
    alice.address0,
    wasm,
    uploadFee,
    "Upload isotonic-token contract",
  );
  console.info(`Token upload succeeded. Receipt: ${JSON.stringify(uploadReceiptToken)}`);

  var wasm = fs.readFileSync(__dirname + "/contracts/isotonic_oracle.wasm");
  const uploadReceiptOracle = await client.upload(
    alice.address0,
    wasm,
    uploadFee,
    "Upload isotonic-oracle contract",
  );
  console.info(`Oracle upload succeeded. Receipt: ${JSON.stringify(uploadReceiptOracle)}`);

  var wasm = fs.readFileSync(__dirname + "/contracts/isotonic_credit_agency.wasm");
  const uploadReceiptAgency = await client.upload(
    alice.address0,
    wasm,
    uploadFee,
    "Upload isotonic-credit-agency contract",
  );
  console.info(`Credit Agency upload succeeded. Receipt: ${JSON.stringify(uploadReceiptAgency)}`);

  const instantiateFee = calculateFee(600_000, gasPrice);

  var oracle = "";
  var { label, msg, admin } = oracleInit;
  {
    const { contractAddress } = await client.instantiate(
      alice.address0,
      uploadReceiptOracle.codeId,
      msg,
      label,
      instantiateFee,
      {
        memo: `Create a isotonic-oracle instance.`,
        admin: admin,
      },
    );
    oracle = contractAddress;
    console.info(`Oracle contract instantiated at ${contractAddress}`);
  }

  var creditAgency = "";
  var { label, msg, admin } = creditAgencyInit;
  {
    const { contractAddress } = await client.instantiate(
      alice.address0,
      uploadReceiptAgency.codeId,
      msg,
      label,
      instantiateFee,
      {
        memo: `Create a isotonic-credit-agency instance`,
        admin: admin,
      },
    );
    console.info(`Credit Agency contract instantiated at ${contractAddress}`);
    creditAgency = contractAddress;
  }

  await client.execute(
    alice.address0,
    creditAgency,
    firstMarket,
    instantiateFee,
    "Create a isotonic-market instance through CA",
  );
  console.info(`Create a first isotonic-market (ustake) instance through CA completed.`);

  await client.execute(
    alice.address0,
    creditAgency,
    secondMarket,
    instantiateFee,
    "Create a isotonic-market instance through CA",
  );
  console.info(`Create a second isotonic-market (ucosm) instance through CA completed.`);

  await client.execute(
    alice.address0,
    oracle,
    { "set_price": {
      "sell": { Native: "ucosm"},
      "buy": { Native: "ustake"},
      "rate": "2",
    } },
    instantiateFee,
    "Set price for ucosm/ustake at 2.0",
  );
  console.info(`Set price for ucosm/ustake at 2.0 in Oracle.`);

  await client.execute(
    alice.address0,
    oracle,
    { "set_price": {
      "sell": { Native: "ustake" },
      "buy": { Native: "ucosm" },
      "rate": "0.5",
    } },
    instantiateFee,
    "Set price for ucstake/ucosm at 0.5",
  );
  console.info(`Set price for ustake/ucosm at 0.5 in Oracle.`);

}

main().then(
  () => {
    console.info("All done, let the coins flow.");
    process.exit(0);
  },
  (error) => {
    console.error(error);
    process.exit(1);
  },
);
