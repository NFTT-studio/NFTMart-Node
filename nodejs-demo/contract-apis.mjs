import {
  waitTx,
  hexToUtf8,
  unit,
  ensureAddress,
  Global_Api,
  Global_ModuleMetadata,
  initApi,
} from "./utils.mjs";
import { Keyring } from "@polkadot/api";
import { bnToBn } from "@polkadot/util";
import { Command } from "commander";

async function main() {
  const ss58Format = 50;
  const keyring = new Keyring({ type: "sr25519", ss58Format });
  const program = new Command();
  program.option("--ws <url>", "node ws addr", "ws://192.168.0.2:9944");

  // node contract-apis.mjs --ws 'ws://81.70.132.13:9944' deploy //Alice
  // node contract-apis.mjs deploy //Alice
  program
    .command("deploy <signer> <contractpath>")
    .action(async (signer, contractpath) => {
      await deploy(program.opts().ws, keyring, signer, contractpath);
    });
  await program.parseAsync(process.argv);
}

async function deploy(ws, keyring, signer) {
  await initApi(ws);
  signer = keyring.addFromUri(signer);

  const endowment = bnToBn("2000").mul(unit);
  const gas_limit = bnToBn("200000000000");
  const data = "0x9bae9d5e00e0b0b0e5e407000000000000000000";
  const salt =
    "0xaa39dcb42a0342caafa1c4e547020e5f2a1535cda5b4d93651bda27edd157983";

  let [a, b] = waitTx(Global_ModuleMetadata);
  await Global_Api.tx.contracts
    .instantiateWithCode(endowment, gas_limit, code, data, salt)
    .signAndSend(signer, a);
  await b();
}

main()
  .then((r) => {
    process.exit();
  })
  .catch((err) => {
    console.log(err);
    process.exit();
  });
