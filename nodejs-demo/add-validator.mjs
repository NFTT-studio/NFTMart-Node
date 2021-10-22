import { Keyring } from "@polkadot/api";
import * as BN from "bn.js";
import fs from "fs";
import { getApi, getModules, waitTx, hexToUtf8 } from "./utils.mjs";
import { Command } from "commander";
import yaml from "yaml";

const MNEMONIC_WORDS_COUNT = 12;

class Validator {
  /*
  api;
  rootAccount;
  stashAccount;
  controllerAccount;
  sessionKey;
  rootBalance;
  stashBalance;
  controllerBalance;
  */

  constructor(config) {
    this.env = yaml.parse(fs.readFileSync(config, "utf8"));
    console.log(`initializing new validator from config file ${config}`);
    console.log(this.env);
  }

  /**
   * Initialization - Connecting to blockchain.
   */
  async init() {
    console.log(`Connecting to blockchain: ${this.env.PROVIDER}`);

    this.api = await getApi(this.env.PROVIDER);
    await this.api.isReady;
    const chain = await this.api.rpc.system.chain();
    console.log(`Connected to: ${chain}\n`);

    console.log("Check if syncing...");
    await this.callWithRetry(this.isSyncing.bind(this), {
      maxDepth: 100,
    });
    console.log("Sync is complete!");
  }

  async isSyncing() {
    const response = await this.api.rpc.system.health();

    if (response.isSyncing.valueOf()) {
      throw new Error("Node is syncing");
    }
  }

  async setIdentity(account, name) {
    const identityInfo = this.api.createType("IdentityInfo", {
      additional: [],
      display: { raw: name },
      legal: { none: null },
      web: { none: null },
      riot: { none: null },
      email: { none: null },
      image: { none: null },
      twitter: { none: null },
    });
    return new Promise((res, rej) => {
      this.api.tx.identity
        .setIdentity(identityInfo)
        .signAndSend(account, this.sendStatusCb.bind(this, res, rej))
        .catch((err) => rej(err));
    });
  }

  /**
   * Load stash and controller accounts.
   */
  async loadAccounts() {
    console.log(`Loading your accounts`);
    const keyring = new Keyring({ type: "sr25519" });

    console.log(`Loading root`);
    this.rootAccount = keyring.addFromMnemonic(this.env.ROOT_ACCOUNT_MNEMONIC);
    console.log(`Loading stash`);
    this.stashAccount = keyring.addFromMnemonic(
      this.env.STASH_ACCOUNT_MNEMONIC
    );
    console.log(`Loading controller`);
    this.controllerAccount = keyring.addFromMnemonic(
      this.env.CONTROLLER_ACCOUNT_MNEMONIC
    );

    console.log(`Requesting endowment`);
    await this.requestEndowment(this.stashAccount);
    await this.requestEndowment(this.controllerAccount);

    console.log(`Setting identities`);
    await this.setIdentity(this.stashAccount, this.env.STASH_ACCOUNT_MNEMONIC);
    await this.setIdentity(
      this.controllerAccount,
      this.env.CONTROLLER_ACCOUNT_MNEMONIC
    );

    this.rootBalance = await this.getBalance(this.rootAccount);
    this.stashBalance = await this.getBalance(this.stashAccount);
    this.controllerBalance = await this.getBalance(this.controllerAccount);

    console.log(
      `Your Root Account is ${this.rootAccount.address} and balance is ${this.rootBalance}`
    );
    console.log(
      `Your Stash Account is ${this.stashAccount.address} (${this.env.STASH_ACCOUNT_MNEMONIC}) and balance is ${this.stashBalance}`
    );
    console.log(
      `Your Controller Account is ${this.controllerAccount.address} (${this.env.CONTROLLER_ACCOUNT_MNEMONIC}) and balance is ${this.controllerBalance}\n`
    );
  }

  /**
   * Generate session key
   */
  async generateSessionKey() {
    console.log(`\nGenerating Session Key`);
    this.sessionKey = await this.api.rpc.author.rotateKeys();
    console.log(`Session Key: ${this.sessionKey}`);
  }

  /**
   * Add validator to the node
   * @param bondValue The amount to be stashed
   * @param payee The rewards destination account
   */
  async addValidator() {
    console.log(`\nAdding validator`);
    const bondValue = BigInt(Number(this.env.BOND_VALUE));
    console.log(`Bond value is ${bondValue}`);
    if (this.stashBalance <= Number(bondValue)) {
      throw new Error(
        `Bond value needs to be lesser than balance. (Bond ${bondValue} should be less than stash balance ${this.stashBalance})`
      );
    }

    const transaction = this.api.tx.staking.bond(
      this.controllerAccount.address,
      bondValue,
      "Staked"
    );

    return new Promise((res, rej) => {
      transaction
        .signAndSend(this.stashAccount, this.sendStatusCb.bind(this, res, rej))
        .catch((err) => rej(err));
    });
  }

  async setController() {
    console.log(`\n Setting controller account`);
    const transaction = this.api.tx.staking.setController(
      this.controllerAccount.address
    );

    return new Promise((res, rej) => {
      transaction
        .signAndSend(this.stashAccount, this.sendStatusCb.bind(this, res, rej))
        .catch((err) => rej(err));
    });
  }

  /**
   * Set session key
   * @param sessionKey session key
   */
  async setSessionKey() {
    console.log(`\nSetting session key`);
    const EMPTY_PROOF = new Uint8Array();
    const transaction = this.api.tx.session.setKeys(
      this.sessionKey,
      EMPTY_PROOF
    );

    return new Promise((res, rej) => {
      transaction
        .signAndSend(
          this.controllerAccount,
          this.sendStatusCb.bind(this, res, rej)
        )
        .catch((err) => rej(err));
    });
  }

  /**
   * set rewards commission
   * @param REWARD_COMMISSION rewards commission
   */
  async setCommission() {
    console.log(`\nSetting reward commission`);
    // https://github.com/polkadot-js/apps/blob/23dad13c9e67de651e5551e4ce7cba3d63d8bb47/packages/page-staking/src/Actions/partials/Validate.tsx#L53
    const COMM_MUL = 10000000;
    const commission = +this.env.REWARD_COMMISSION * COMM_MUL;
    const transaction = this.api.tx.staking.validate({
      commission,
    });

    return new Promise((res, rej) => {
      transaction
        .signAndSend(
          this.controllerAccount,
          this.sendStatusCb.bind(this, res, rej)
        )
        .catch((err) => rej(err));
    });
  }

  async requestEndowment(account) {
    console.log("Requesting endowment for account", account.address);
    const oldBalance = await this.getBalance(account);
    const transfer = this.api.tx.balances.transfer(
      account.address,
      1000000000000000
    );
    const hash = await transfer.signAndSend(this.rootAccount, { nonce: -1 });

    while (true) {
      let newBalance = await this.getBalance(account);
      if (newBalance > oldBalance) {
        break;
      }
      console.log("please wait for transaction to finalize...");
      await this.sleep(1000);
    }
  }

  async getBalance(account) {
    const result = await this.api.query.system.account(account.address);
    const {
      data: { free: balance },
    } = result;

    return Number(balance);
  }

  async sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  async callWithRetry(fn, options = { maxDepth: 5 }, depth = 0) {
    try {
      return await fn();
    } catch (e) {
      if (depth > options.maxDepth) {
        throw e;
      }
      const seconds = parseInt(this.env.WAIT_SECONDS, 10);
      console.log(`Wait ${seconds}s.`);
      await this.sleep(seconds * 1000);

      return this.callWithRetry(fn, options, depth + 1);
    }
  }

  sendStatusCb(res, rej, { events = [], status }) {
    if (status.isInvalid) {
      console.info("Transaction invalid");
      rej("Transaction invalid");
    } else if (status.isReady) {
      console.info("Transaction is ready");
    } else if (status.isBroadcast) {
      console.info("Transaction has been broadcasted");
    } else if (status.isInBlock) {
      const hash = status.asInBlock.toHex();
      console.info(`Transaction is in block: ${hash}`);
    } else if (status.isFinalized) {
      const hash = status.asFinalized.toHex();
      console.info(`Transaction has been included in blockHash ${hash}`);
      events.forEach(({ event }) => {
        if (event.method === "ExtrinsicSuccess") {
          console.info("Transaction succeeded");
        } else if (event.method === "ExtrinsicFailed") {
          console.info("Transaction failed");
          throw new Error("Transaction failed");
        }
      });

      res(hash);
    }
  }
}

async function newValidator(config) {
  const validator = new Validator(config);
  await validator.init();
  await validator.loadAccounts();
  await validator.generateSessionKey();
  await validator.addValidator();
  await validator.setSessionKey();
  await validator.setCommission();

  console.log("Validator added successfully!");
}

async function main() {
  var program = new Command();

  program.arguments("<config...>").action(async () => {
    for (const config of program.args) {
      console.log(`Adding validator from config ${config}`);
      await newValidator(config);
    }
  });

  await program.parseAsync(process.argv);
}

main()
  .catch(console.error)
  .finally(() => process.exit());
