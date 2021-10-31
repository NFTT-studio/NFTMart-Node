/**
 * See @href https://wiki.polkadot.network/docs/en/learn-identity
 */
import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { blake2AsHex } from "@polkadot/util-crypto";
import yaml from "yaml";
import fs from "fs";
import { Command } from "commander";
import { getApi, getModules, waitTx, hexToUtf8 } from "./utils.mjs";

// DEFUALT FEE is 1 Unit
const DEFAULT_REGISTRAR_FEE = 1000000000000;
const DEFAULT_DEMOCRACY_VOTE_FEE = 1000000000000;
const DEFAULT_DEMOCRACY_PROPOSAL_FEE = 1000000000000;
const DEFAULT_SLEEP_INTERVAL = 6;

function sleep(seconds) {
  return new Promise((resolve) => {
    setTimeout(resolve, seconds * 1000);
  });
}

class Registrar {
  /**
   * A wrapped APIs for block chain
   * @constructor
   */
  constructor(config) {
    this.config = yaml.parse(fs.readFileSync(config, "utf8"));
    this.wsProvider = new WsProvider(this.config.PROVIDER);
    this.keyring = new Keyring({ type: "sr25519" });

    this.unsubscribeEventListener = null;
  }

  /**
   * Connect to a configured block chain, such as polkadot, westend or local chain.
   */
  async init() {
    const _alice = "//Alice";
    const _bob = "//Bob";
    const _charlie = "//Charlie";
    const _dave = "//Dave";
    const _eve = "//Eve";
    if (!this.api) {
      this.api = await getApi(this.config.PROVIDER);
      await this.api.isReady;
    }
    if (!this.mythis) {
      this.mythis = this.keyring.addFromUri(_alice);
      this.alice = this.keyring.addFromUri(_alice);
      this.bob = this.keyring.addFromUri(_bob);
      this.charlie = this.keyring.addFromUri(_charlie);
      this.dave = this.keyring.addFromUri(_dave);
      this.eve = this.keyring.addFromUri(_eve);
    }
    return this.api;
  }
  async signAndSend(tx, account) {
    try {
      const block = await tx.signAndSend(account);
      return block;
    } catch (error) {
      console.log(`Error occurs:`);
      console.trace(error);
    }
    return null;
  }
  /**
   * identity
   */
  async identityRegistrars() {
    const registrars = await this.api.query.identity.registrars();
    console.log(`[identity.registrars]: ${registrars}`);
    return registrars;
  }
  async identityIdentityOf() {
    const identityOf = await this.api.query.identity.identityOf(
      this.mythis.address
    );
    console.log(`[identity.identityOf]: ${identityOf.toHuman()}`);
    return identityOf;
  }
  async identityAddRegistrar(account, registrarAccount) {
    const tx = this.api.tx.identity.addRegistrar(registrarAccount.address);
    console.log(`[identity.addRegistrar]: ${tx}`);
    return tx;
  }

  async identitySetFee(account, regIndex = 0, fee = DEFAULT_REGISTRAR_FEE) {
    const tx = this.api.tx.identity.setFee(regIndex, fee);
    await this.signAndSend(tx, account);
    console.log(`[identity.setFee]: ${tx}`);
    return tx;
  }
  /**
   * democracy
   */
  async democracyPublicPropCount() {
    const publicPropCount = await this.api.query.democracy.publicPropCount();
    console.log(`[democracy.publicPropCount]: ${publicPropCount}`);
    return publicPropCount;
  }
  async democracyPublicProps() {
    const publicProps = await this.api.query.democracy.publicProps();
    console.log(`[democracy.publicProposals]: ${publicProps}`);
    return publicProps;
  }
  async democracyReferendumCount() {
    const referendumCount = await this.api.query.democracy.referendumCount();
    console.log(`[democracy.referendumCount]: ${referendumCount}`);
    return referendumCount;
  }
  async democracyReferendumInfoOf() {
    const referendumCount = await this.democracyReferendumCount();
    let referendumInfo = [];
    for (let i = 0; i < referendumCount; i++) {
      const info = await this.api.query.democracy.referendumInfoOf(i);
      console.log(`[democracy.referendumInfoOf]: ${info}`);
      referendumInfo.push(info);
    }

    return referendumInfo;
  }

  async democracyPropose(
    account,
    func,
    args,
    value = DEFAULT_DEMOCRACY_PROPOSAL_FEE
  ) {
    // const toAddRegistrar = await this.identityAddRegistrar(registrarAccount);
    const result = await func(...args);
    const encodedProposal = result.method.toHex();
    console.log("encodeProposal: ", encodedProposal);
    const preimage = blake2AsHex(encodedProposal);
    console.log("preimage: ", preimage);
    const tx = this.api.tx.democracy.propose(preimage, value);
    await this.signAndSend(tx, account);
    console.log(`[democracy.propose]: ${tx}`);
    return tx;
  }

  async democracyNotePreimage(account, func, args) {
    // const toAddRegistrar = await this.identityAddRegistrar();
    const result = await func(...args);
    // const encodedProposal = toAddRegistrar.method.toHex();
    const encodedProposal = result.method.toHex();
    const tx = this.api.tx.democracy.notePreimage(encodedProposal);
    await this.signAndSend(tx, account);
    console.log(`[democracy.notePreimage]: ${tx}`);
    return tx;
  }

  async democracyVote(account, balance = DEFAULT_DEMOCRACY_VOTE_FEE) {
    const referendumInfo = await this.democracyReferendumInfoOf();
    const vote = {
      Standard: {
        vote: true,
        conviction: "None",
        // 0.1 Unit
        // balance: 1000000000000000
        balance: balance,
      },
    };
    console.log(
      `vote on referendumInfo: ${referendumInfo[referendumInfo.length - 1]}`
    );
    const tx = this.api.tx.democracy.vote(referendumInfo.length - 1, vote);
    await this.signAndSend(tx, account);
    console.log(`[democracy.vote]: ${tx}`);
    return tx;
  }

  async proxyProxies(account) {
    const resp = await this.api.query.proxy.proxies(account);
    console.log(`[proxy.proxies]: ${resp}`);
    return [null, resp];
  }

  async proxyAddProxy(
    account,
    delegateAccount,
    proxyType = "IdentityJudgement",
    delay = 0
  ) {
    const tx = this.api.tx.proxy.addProxy(delegateAccount, proxyType, delay);
    const resp = await this.signAndSend(tx, account);
    console.log(`[identity.RequestJudgement] tx: ${tx}`);
    console.log(`[identity.RequestJudgement] resp: ${resp}`);
    return [tx, resp];
  }

  async disconnect() {
    console.log(`Disconnect from chain`);
    await this.api.disconnect();
  }

  /**
   * @description set up a registrar for an account
   */
  async setupRegistrar(registrarAccount) {
    // FIXME: Enforce address mapping
    const account2registrar = {
      "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY":
        "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5", //Alice : prefix 42 <=> prefix 0
      "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty":
        "14E5nqKAp3oAJcmzgZhUD2RcptBeUBScxKHgJKU4HPNcKVf3", //Bob   : prefix 42 <=> prefix 0
    };
    console.log(`[setupRegistrar] Try to add registrar: `);
    console.log(registrarAccount.toJson());
    /**
     * check if there is a registrar
     */
    let registrars = await this.identityRegistrars();

    if (registrars.length > 0) {
      for (let registrar of registrars.toArray()) {
        console.log(`registrar.value: ${registrar.value.account}`);
        console.log(`registrarAccount: ${registrarAccount.address}`);
        if (
          `${registrar.value.account}` ===
          `${account2registrar[registrarAccount.address]}`
        ) {
          console.log(`registrar already existed, ignore.`);
          return;
        }
      }
    }
    /**
     * create a proposal for registrar
     */
    let publicProps = await this.democracyPublicProps();
    await sleep(DEFAULT_SLEEP_INTERVAL);
    if (`${publicProps.length}` === "0") {
      await this.democracyNotePreimage(
        this.alice,
        this.identityAddRegistrar.bind(this),
        [this.alice, registrarAccount]
      );
      await sleep(DEFAULT_SLEEP_INTERVAL);
      await this.democracyPropose(
        this.alice,
        this.identityAddRegistrar.bind(this),
        [this.alice, registrarAccount]
      );
      await sleep(DEFAULT_SLEEP_INTERVAL);
    }
    let referendumInfo = await this.democracyReferendumInfoOf();
    // Make sure there is at least one referendum in array
    while (`${referendumInfo.length}` === "0") {
      await sleep(DEFAULT_SLEEP_INTERVAL);
      referendumInfo = await this.democracyReferendumInfoOf();
    }
    // Extract latest referendum from given array
    let lastReferendumInfo = referendumInfo[referendumInfo.length - 1];
    // Make sure this referendum is `isOngoing` status
    while (!lastReferendumInfo.value.isOngoing) {
      await sleep(DEFAULT_SLEEP_INTERVAL);
      referendumInfo = await this.democracyReferendumInfoOf();
      lastReferendumInfo = referendumInfo[referendumInfo.length - 1];
    }
    // Now we can safely vote this proposal
    await this.democracyVote(this.alice);
    await sleep(DEFAULT_SLEEP_INTERVAL);
    /**
     * query the result of registrar
     */
    registrars = await this.identityRegistrars();

    let waiting = true;
    let regIndex = -1;
    while (waiting) {
      await sleep(DEFAULT_SLEEP_INTERVAL);
      registrars = await this.identityRegistrars();
      console.log(`Number of existed registrars: ${registrars.length}`);
      for (let registrar of registrars) {
        regIndex += 1;

        console.log(`registrar.value: ${registrar.value.account}`);
        console.log(`registrarAccount: ${registrarAccount.address}`);

        if (
          `${registrar.value.account}` ===
          `${account2registrar[registrarAccount.address]}`
        ) {
          waiting = false;
          break;
        }
      }
    }

    /**
     * set registrar fee and query results
     */
    const fee = DEFAULT_REGISTRAR_FEE;
    await this.identitySetFee(registrarAccount, regIndex, fee);
    await sleep(DEFAULT_SLEEP_INTERVAL);
    await this.identityRegistrars();
    await this.proxyAddProxy(registrarAccount, this.eve);
  }
}

async function newRegistrar(config) {
  const registrar = new Registrar(config);
  await registrar.init();
  await registrar.setupRegistrar(registrar.alice);
  /* eslint-disable-next-line */
  const [tx, resp] = await registrar.proxyProxies(registrar.alice.address);
  let shouldAddProxy = true;
  if (resp && resp[0]) {
    for (let tmp of resp[0]) {
      const delegateAccount = `${tmp.delegate}`;
      console.log(delegateAccount);
      if (
        delegateAccount === "16D2eVuK5SWfwvtFD3gVdBC2nc2BafK31BY6PrbZHBAGew7L"
      ) {
        shouldAddProxy = false;
        break;
      }
    }
  }
  if (shouldAddProxy) {
    console.log("Should add proxy");
    await registrar.proxyAddProxy(registrar.alice, registrar.eve.address);
  } else {
    console.log("No need to add proxy");
  }

  await registrar.disconnect();
}

async function main() {
  var program = new Command();

  program.arguments("<config...>").action(async () => {
    for (const config of program.args) {
      console.log(`Adding registrar from config ${config}`);
      await newRegistrar(config);
    }
  });

  await program.parseAsync(process.argv);
}

main()
  .catch(console.error)
  .finally(() => process.exit());
