#!/usr/bin/env node

/* eslint-disable @typescript-eslint/naming-convention */
const { SigningCosmWasmClient, fromBinary } = require("@cosmjs/cosmwasm-stargate");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { calculateFee, GasPrice } = require("@cosmjs/stargate");
const { coins } = require("@cosmjs/amino");
const { assert } = require("@cosmjs/utils");

const endpoint = "http://localhost:26659";

const alice = {
  mnemonic: "enlist hip relief stomach skate base shallow young switch frequent cry park",
  address0: "wasm14qemq0vw6y3gc3u3e0aty2e764u4gs5lndxgyk",
};

const bob = {
  mnemonic: "remain fragile remove stamp quiz bus country dress critic mammal office need",
  address0: "wasm1lvrwcvrqlc5ktzp2c4t22xgkx29q3y83426at5",
}

async function main() {
  const gasPrice = GasPrice.fromString("0.025ucosm");
  const bobWallet = await DirectSecp256k1HdWallet.fromMnemonic(bob.mnemonic, { prefix: "wasm" });
  const client = await SigningCosmWasmClient.connectWithSigner(endpoint, bobWallet);

  const aliceWallet = await DirectSecp256k1HdWallet.fromMnemonic(alice.mnemonic, { prefix: "wasm" });
  const otherClient = await SigningCosmWasmClient.connectWithSigner(endpoint, aliceWallet);
  // Contract's addresses are generated deterministicly - if they are created in same order,
  // address will be always the same.
  const creditAgency = "wasm1wkwy0xh89ksdgj9hr347dyd2dw7zesmtrue6kfzyml4vdtz6e5wsxxsfq6";

  const fee = calculateFee(750_000, gasPrice);

  var ustakeMarket = (await client.queryContractSmart(
    creditAgency,
    { market: { market_token: "ustake" } },
  )).market;
  console.info("Query credit agency for ustake market: ", ustakeMarket);

  var ucosmMarket = (await client.queryContractSmart(
    creditAgency,
    { market: { market_token: "ucosm" } },
  )).market;
  console.info("Query credit agency for ucosm market: ", ucosmMarket);

  await client.execute(
    bob.address0,
    ustakeMarket,
    { deposit: {} },
    fee,
    "Deposit 10_000 ustake tokens to market: ${ustakeMarket}",
    coins(10000, "ustake")
  );
  console.info("Deposit 10_000 ustake tokens to market: ", ustakeMarket);

  // Deposit tokens to second market, so that it has some tokens to borrow
  await otherClient.execute(
    alice.address0,
    ucosmMarket,
    { deposit: {} },
    fee,
    "Deposit 10_000 ucosm tokens to market: ${ucosmMarket}",
    coins(10000, "ucosm")
  );

  await client.execute(
    bob.address0,
    ucosmMarket,
    { borrow: { amount: "500" } },
    fee,
    "Borrow 500 ucosm tokens from market: ${ucosmMarket}",
  );
  console.info("Borrow 500 ucosm tokens from market: ", ucosmMarket);


  {
    var balance  = await client.queryContractSmart(
      ustakeMarket,
      { tokens_balance: { account: bob.address0 } },
    );
    console.info("Tokens balance for address bob on ustake market: ", balance);
    assert(balance.ltokens == "10000", "Error: Incorrect ltokens balance!");

    var balance  = await client.queryContractSmart(
      ucosmMarket,
      { tokens_balance: { account: bob.address0 } },
    );
    console.info("Tokens balance for address bob on ucosm market: ", balance);
    assert(balance.btokens == "500", "Error: Incorrect btokens balance!");
  }

  {
    const creditLine  = await client.queryContractSmart(
      ustakeMarket,
      { credit_line: { account: bob.address0 } },
    );
    console.info("Credit line for address bob: ", creditLine);
    // collateral of ustake is 10_000 deposited * 0.5 ucosm (common token)
    assert(creditLine.collateral.amount === "5000", "Incorrect collateral balance!");
    // 5000 collateral * 0.7 collateral ratio
    assert(creditLine.credit_line.amount === "3500", "Incorrect credit line balance!");
  }

  {
    const tcr  = await client.queryContractSmart(
      creditAgency,
      { total_credit_line: { account: bob.address0 } },
    );
    console.info("Total credit line for address bob: ", tcr);
    assert(tcr.collateral.amount === "5000", "Incorrect total collateral balance!");
    assert(tcr.credit_line.amount === "3500", "Incorrect total credit line balance!");
    assert(tcr.debt.amount === "500", "Incorrect total debt balance!");
  }
}

main().then(
  () => {
    console.info("Market worked well.");
    process.exit(0);
  },
  (error) => {
    console.error(error);
    process.exit(1);
  },
);
