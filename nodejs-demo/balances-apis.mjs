import {getApi, getModules, waitTx} from "./utils.mjs";
import {Keyring} from "@polkadot/api";
import {bnToBn} from "@polkadot/util";
import {Command} from "commander";

const unit = bnToBn('1000000000000');

async function main() {
  const ss58Format = 50;
  const keyring = new Keyring({type: 'sr25519', ss58Format});
  const program = new Command();
  program.option('--ws <addr>', 'node ws addr', 'ws://tmp-chain.bcdata.top');

  program.command('transfer <from> <to>').action(async (from, to) => {
    await demo_transfer(program.opts().ws, keyring, from, to);
  });
  program.command('show-all').action(async () => {
    await demo_show_all(program.opts().ws, keyring);
  });
  program.command('show <account>').action(async (account) => {
    await demo_show(program.opts().ws, keyring, account);
  });
  await program.parseAsync(process.argv);
}

async function demo_show(ws, keyring, account) {
  let api = await getApi(ws);
  let addr = '';
  if (account.length === '62qUEaQwPx7g4vDz88cT36XXuEUQmYo3Y5dxnxScsiDkb8wy'.length) {
    addr = account;
  } else {
    addr = keyring.addFromUri(account).address;
  }
  account = await api.query.system.account(addr)
  console.log(addr, account.toHuman());
}

async function demo_show_all(ws, keyring) {
  let api = await getApi(ws);
  const all = await api.query.system.account.entries();
  for (const account of all) {
    let key = account[0];
    const len = key.length;
    key = key.buffer.slice(len - 32, len);
    const addr = keyring.encodeAddress(new Uint8Array(key));
    let data = account[1].toHuman();
    data.address = addr;
    console.log("%s", JSON.stringify(data));
  }
}

async function demo_transfer(ws, keyring, from, to) {
  let api = await getApi(ws);
  let moduleMetadata = await getModules(api);
  from = keyring.addFromUri(from);
  to = keyring.addFromUri(to).address;
  let [a, b] = waitTx(moduleMetadata);
  await api.tx.balances.transfer(to, bnToBn(1000000).mul(unit)).signAndSend(from, a);
  await b();
}

main().then(r => {
  process.exit();
}).catch(err => {
  console.log(err);
  process.exit();
});
