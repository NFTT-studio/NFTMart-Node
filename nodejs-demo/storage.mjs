import { getApi, getModules, waitTx } from "./utils.mjs";
import { Keyring } from "@polkadot/api";
import { Command } from "commander";
import { xxhashAsHex } from "@polkadot/util-crypto";
import { bnToBn } from "@polkadot/util";
const unit = bnToBn("1000000000000");

function main() {
  const ss58Format = 12191;
  const keyring = new Keyring({ type: "sr25519", ss58Format });
  const program = new Command();
  program.command("get-version").action(async () => {
    await demo_get_version();
  });
  program
    .command("set-version <account> <ver>")
    .action(async (account, ver) => {
      await demo_set_version(keyring, account, ver);
    });
  program.parse();
}

async function demo_set_version(keyring, account, ver) {
  let api = await getApi();
  let moduleMetadata = await getModules(api);
  account = keyring.addFromUri(account);

  const module = xxhashAsHex("Nftmart", 128);
  const storage = xxhashAsHex("StorageVersion", 128);
  const key = module + storage.slice(2);

  const call = api.tx.sudo.sudo(api.tx.system.setStorage([[key, ver]]));

  const feeInfo = await call.paymentInfo(account);
  console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
  let [a, b] = waitTx(moduleMetadata);
  await call.signAndSend(account, a);
  await b();
  process.exit();
}

async function demo_get_version() {
  let api = await getApi();
  const module = xxhashAsHex("Nftmart", 128);
  const storage = xxhashAsHex("StorageVersion", 128);
  const key = module + storage.slice(2);
  let a = await api.rpc.state.getStorage(key);
  console.log(key, a.toString());
  process.exit();
}

main();
