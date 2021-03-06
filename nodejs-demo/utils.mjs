import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { Client as WebSocket } from "rpc-websockets";
import { bnToBn } from "@polkadot/util";
import types from "./types.mjs";

export const convert = (from, to) => (str) =>
  Buffer.from(str, from).toString(to);
export const utf8ToHex = convert("utf8", "hex");
export const hexToUtf8 = convert("hex", "utf8");
export const unit = bnToBn("1000000000000");

export function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export const NativeCurrencyID = 0;

export let Global_Api = null;
export let Global_ModuleMetadata = null;

export async function initApi(ws) {
  if (Global_Api === null || Global_ModuleMetadata === null) {
    Global_Api = await getApi(ws);
    Global_ModuleMetadata = await getModules(Global_Api);
  }
}

export function getKeyring() {
  const ss58Format = Global_Api.consts.system.ss58Prefix.toNumber();
  return new Keyring({ type: "sr25519", ss58Format });
}

export function getRandomInt(max) {
  return Math.floor(Math.random() * max);
}

export function waitTx(moduleMetadata) {
  let signal = false;
  return [
    ({ events = [], status }) => {
      // console.log(JSON.stringify(status));
      // if (status.isFinalized) {
      if (status.isInBlock) {
        // console.log('%s BlockHash(%s)', status.type, status.asFinalized.toHex());
        console.log("%s BlockHash(%s)", status.type, status.asInBlock.toHex());
        events.forEach(({ phase, event: { data, method, section } }) => {
          if ("system.ExtrinsicFailed" === section + "." + method) {
            let processed = false;
            for (let d of data) {
              if (d.isModule) {
                let mErr = d.asModule;
                let module = moduleMetadata[mErr.index];
                console.log(
                  "error: %s.%s",
                  module.name,
                  module.errors[mErr.error].name
                );
                processed = true;
              }
            }
            if (!processed) {
              console.log(
                "event: " +
                  phase.toString() +
                  " " +
                  section +
                  "." +
                  method +
                  " " +
                  data.toString()
              );
            }
          } else if ("system.ExtrinsicSuccess" === section + "." + method) {
            // ignore
          } else if ("proxy.ProxyExecuted" === section + "." + method) {
            // console.log(data.toString());
            for (let d of data) {
              d = d.toJSON();
              if (
                d.err &&
                d.err.module &&
                d.err.module.index &&
                d.err.module.error
              ) {
                let module = moduleMetadata[d.err.module.index];
                console.log(
                  "proxy.ProxyExecuted: %s.%s",
                  module.name,
                  module.errors[d.err.module.error].name
                );
              } else {
                console.log(
                  "event: " +
                    phase.toString() +
                    " " +
                    section +
                    "." +
                    method +
                    " " +
                    data.toString()
                );
              }
            }
          } else {
            console.log(
              "event: " +
                phase.toString() +
                " " +
                section +
                "." +
                method +
                " " +
                data.toString()
            );
          }
        });
        signal = true;
      }
    },
    async function () {
      for (;;) {
        await sleep(100);
        if (signal) break;
      }
    },
  ];
}

export async function getApi(dest) {
  // https://github.com/elpheria/rpc-websockets/blob/master/API.md#new-websocketaddress-options---client
  const ws = new WebSocket(dest, { max_reconnects: 0 });
  let connected = false;
  ws.on("open", function () {
    process.on("unhandledRejection", (error) => {
      console.log("global error handle: ", error.message);
    });
    connected = true;
  });
  const provider = new WsProvider(dest);

  const api = await ApiPromise.create({ provider, types });
  const [chain, nodeName, nodeVersion] = await Promise.all([
    api.rpc.system.chain(),
    api.rpc.system.name(),
    api.rpc.system.version(),
  ]);
  console.log(
    `You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`
  );
  api.ws = ws;
  while (!connected) {
    await sleep(300);
  }
  console.log("ws client has connected to %s", dest);
  return api;
}

export function ensureAddress(keyring, account) {
  if (
    account.length !==
    "nmtSEocH8xUt7rnrZdKLE7bHZcL9VKemo1DtpXiqaG4PEbhTz".length
  ) {
    account = keyring.addFromUri(account);
    account = account.address;
  }
  return account;
}

export function secondsToString(seconds) {
  let numyears = Math.floor(seconds / 31536000);
  let numdays = Math.floor((seconds % 31536000) / 86400);
  let numhours = Math.floor(((seconds % 31536000) % 86400) / 3600);
  let numminutes = Math.floor((((seconds % 31536000) % 86400) % 3600) / 60);
  let numseconds = (((seconds % 31536000) % 86400) % 3600) % 60;
  return (
    numyears +
    " years " +
    numdays +
    " days " +
    numhours +
    " hours " +
    numminutes +
    " minutes " +
    Math.round(numseconds) +
    " seconds"
  );
}

export async function getModules(api) {
  let metadata = await api.rpc.state.getMetadata();
  metadata = metadata.asLatest.modules;
  metadata.index = {};
  for (const a of metadata) {
    metadata.index[a.index] = a;
    // console.log(a.index.toString());
  }
  return metadata;
}

export function showStorage(s, verbose) {
  console.log("********** storage ***************");
  // noinspection JSUnresolvedVariable
  if (!s.isNone) {
    let storage = s.unwrap();
    console.log("prefix in key-value databases: [%s]", storage.prefix);
    for (let s of storage.items) {
      // noinspection JSUnresolvedVariable
      console.log(
        "%s: modifier[%s] %s",
        s.name,
        s.modifier,
        s.documentation[0]
      );
      if (verbose) console.log(s.toHuman());
    }
  }
}

export function showCalls(s, verbose) {
  console.log("********** calls ***************");
  // noinspection JSUnresolvedVariable
  if (!s.isNone) {
    let calls = s.unwrap();
    for (let s of calls) {
      // noinspection JSUnresolvedVariable
      console.log("%s: %s", s.name, s.documentation[0]);
      if (verbose) console.log(s.toHuman());
    }
  }
}

export function showErrors(errors, verbose) {
  console.log("********** errors ***************");
  // noinspection JSUnresolvedVariable
  for (let e of errors) {
    // noinspection JSUnresolvedVariable
    console.log("%s: %s", e.name, e.documentation[0]);
    if (verbose) console.log(e.toHuman());
  }
}

export function showEvents(e, verbose) {
  console.log("********** events ***************");
  // noinspection JSUnresolvedVariable
  if (!e.isNone) {
    let events = e.unwrap();
    for (let e of events) {
      // noinspection JSUnresolvedVariable
      console.log("%s: %s", e.name, e.documentation[0]);
      if (verbose) console.log(e.toHuman());
    }
  }
}

export function showConstants(constants) {
  console.log("********** constants ***************");
  for (let c of constants) {
    // noinspection JSUnresolvedVariable
    console.log("%s %s = %s", c.type, c.name, c.documentation);
  }
}

export function findModule(name, moduleMetadata) {
  for (let module of moduleMetadata) {
    // console.log(module.name.toHuman());
    if (name === module.name.toHuman()) {
      return module;
    }
  }
  return {};
}

export function findConstantFrom(name, module) {
  for (let c of module["constants"]) {
    // console.log(module.name.toHuman());
    if (name === c.name.toHuman()) {
      return c;
    }
  }
  return {};
}

export function reverseEndian(x) {
  let buf = Buffer.allocUnsafe(4);
  buf.writeUIntLE(x, 0, 4);
  return buf.readUIntBE(0, 4);
}

export async function getEventsByNumber(api, num) {
  const hash = await api.rpc.chain.getBlockHash(num);
  const events = await api.query.system.events.at(hash);
  // noinspection JSUnresolvedFunction
  return [hash.toHex(), events];
}

export async function getExtrinsicByNumber(api, num) {
  const hash = await api.rpc.chain.getBlockHash(num);
  return api.rpc.chain.getBlock(hash);
  // block.block.extrinsics.forEach((ex, index) => {
  //     console.log(index, ex.method);
  // });
}
