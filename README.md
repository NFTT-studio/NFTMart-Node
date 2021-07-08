# Substrate Node Template

## Metadata

```json
{
  "Properties": "u8",
  "NFTMetadata": "Vec<u8>",
  "BlockNumber": "u32",
  "BlockNumberOf": "BlockNumber",

  "OrderData": {
    "currencyId": "Compact<CurrencyIdOf>",
    "price": "Compact<Balance>",
    "deposit": "Compact<Balance>",
    "deadline": "Compact<BlockNumberOf>",
    "categoryId": "Compact<CategoryIdOf>"
  },

  "CategoryId": "u32",
  "CategoryIdOf": "CategoryId",
  "CategoryData": {
    "metadata": "NFTMetadata",
    "nftCount": "Compact<Balance>"
  },

  "CurrencyId": "u32",
  "CurrencyIdOf": "CurrencyId",
  "Amount": "i128",
  "AmountOf": "Amount",

  "ClassId": "u32",
  "ClassIdOf": "ClassId",
  "ClassInfoOf": {
    "metadata": "NFTMetadata",
    "totalIssuance": "TokenId",
    "owner": "AccountId",
    "data": "ClassData"
  },
  "ClassData": {
    "deposit": "Compact<Balance>",
    "properties": "Properties",
    "name": "Vec<u8>",
    "description": "Vec<u8>",
    "createBlock": "Compact<BlockNumberOf>"
  },

  "TokenId": "u64",
  "TokenIdOf": "TokenId",
  "TokenInfoOf": {"metadata": "NFTMetadata", "owner": "AccountId", "data": "TokenData"},
  "TokenData": {
    "deposit": "Compact<Balance>",
    "createBlock": "Compact<BlockNumberOf>"
  }
}
```


[![Try on playground](https://img.shields.io/badge/Playground-Node_Template-brightgreen?logo=Parity%20Substrate)](https://playground.substrate.dev/?deploy=node-template)

A fresh FRAME-based [Substrate](https://www.substrate.io/) node, ready for hacking :rocket:

## Init command with node.js demo

When you start the node ,you can use js script to init the environment，in order use the pallet properly.

Before use the node script ,we need to install package first:
```
cd nodejs-demo & yarn
```

### Add white list 

The whitelist manage the user who can create class and nft in NFTMart, so we need to add the address in whitelist before create class and nfts.

```
node nft-apis.mjs --ws ws://localhost:9944 add-whitelist //Alice <your address> 
```
Replace `<your address>` to your address.

### Add category

Category as a market tag, when user want to list his NFT to market, he need select a catagory first. And the explore page use the category to filter the listing NFTs.

```
node nft-apis.mjs --ws ws://localhost:9944 create-category "{\"name\":\"Art\"}" //Alice

```
Use this cmd, we can create a category named `Art`.
> Notice: In terminal , we need use `\"` to escape `"`, in order the data will be parsed to Json in the fontend project.

## Explanation of nouns

Maybe we have some confuse about `Category` `Class` `Collections`, here is the explanation.
### Category

Category is manage by sudo account in our App, in order to split NFT market.
If user wants to listing his NFT to the market ,he must select a category befor listing.
And other user can find the nft in specified category.
### Class / Collections

Class is the same thing as Collections, we call it class in substrate node, and call it collections in fontend app.

Its create by the whitelist user, so if you want to create class/collection in our app ,you need to request whitelist permission from the administrator(sudo account).

### Permission rule
- normal user —— normal users are NFT trader or investor, they can buy and listing their NFT.
- whitelist user —— whitelist users are NFT issuer, they can create class/collection and mint NFT. Alse can trade other NFTs.
- administrator —— administrator is manager of this App, he can manage category , manage whitelist and have other rights.

## Getting Started

Follow the steps below to get started with the Node Template, or get it up and running right from your browser
in just a few clicks using [Playground](https://playground.substrate.dev/) :hammer_and_wrench:

### Using Nix

Install [nix](https://nixos.org/) and optionally [direnv](https://github.com/direnv/direnv) and [lorri](https://github.com/target/lorri) for a fully plug
and play experience for setting up the development environment. To get all the correct dependencies activate direnv `direnv allow` and lorri `lorri shell`.

### Rust Setup

First, complete the [basic Rust setup instructions](./doc/rust-setup.md).

### Run

Use Rust's native `cargo` command to build and launch the template node:

```sh
cargo run --release -- --dev --tmp
```

### Build

The `cargo run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
cargo build --release
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/node-template -h
```

## Run

The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/node-template --dev
```

Purge the development chain's state:

```bash
./target/release/node-template purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/node-template -lruntime=debug --dev
```

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

## Template Structure

A Substrate project such as this consists of a number of components that are spread across a few
directories.

### Node

A blockchain node is an application that allows users to participate in a blockchain network.
Substrate-based blockchain nodes expose a number of capabilities:

-   Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking stack to allow the
    nodes in the network to communicate with one another.
-   Consensus: Blockchains must have a way to come to
    [consensus](https://substrate.dev/docs/en/knowledgebase/advanced/consensus) on the state of the
    network. Substrate makes it possible to supply custom consensus engines and also ships with
    several consensus mechanisms that have been built on top of
    [Web3 Foundation research](https://research.web3.foundation/en/latest/polkadot/NPoS/index.html).
-   RPC Server: A remote procedure call (RPC) server is used to interact with Substrate nodes.

There are several files in the `node` directory - take special note of the following:

-   [`chain_spec.rs`](./node/src/chain_spec.rs): A
    [chain specification](https://substrate.dev/docs/en/knowledgebase/integrate/chain-spec) is a
    source code file that defines a Substrate chain's initial (genesis) state. Chain specifications
    are useful for development and testing, and critical when architecting the launch of a
    production chain. Take note of the `development_config` and `testnet_genesis` functions, which
    are used to define the genesis state for the local development chain configuration. These
    functions identify some
    [well-known accounts](https://substrate.dev/docs/en/knowledgebase/integrate/subkey#well-known-keys)
    and use them to configure the blockchain's initial state.
-   [`service.rs`](./node/src/service.rs): This file defines the node implementation. Take note of
    the libraries that this file imports and the names of the functions it invokes. In particular,
    there are references to consensus-related topics, such as the
    [longest chain rule](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#longest-chain-rule),
    the [Aura](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#aura) block authoring
    mechanism and the
    [GRANDPA](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#grandpa) finality
    gadget.

After the node has been [built](#build), refer to the embedded documentation to learn more about the
capabilities and configuration parameters that it exposes:

```shell
./target/release/node-template --help
```

### Runtime

In Substrate, the terms
"[runtime](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#runtime)" and
"[state transition function](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#stf-state-transition-function)"
are analogous - they refer to the core logic of the blockchain that is responsible for validating
blocks and executing the state changes they define. The Substrate project in this repository uses
the [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) framework to construct a
blockchain runtime. FRAME allows runtime developers to declare domain-specific logic in modules
called "pallets". At the heart of FRAME is a helpful
[macro language](https://substrate.dev/docs/en/knowledgebase/runtime/macros) that makes it easy to
create pallets and flexibly compose them to create blockchains that can address
[a variety of needs](https://www.substrate.io/substrate-users/).

Review the [FRAME runtime implementation](./runtime/src/lib.rs) included in this template and note
the following:

-   This file configures several pallets to include in the runtime. Each pallet configuration is
    defined by a code block that begins with `impl $PALLET_NAME::Config for Runtime`.
-   The pallets are composed into a single runtime by way of the
    [`construct_runtime!`](https://crates.parity.io/frame_support/macro.construct_runtime.html)
    macro, which is part of the core
    [FRAME Support](https://substrate.dev/docs/en/knowledgebase/runtime/frame#support-library)
    library.

### Pallets

The runtime in this project is constructed using many FRAME pallets that ship with the
[core Substrate repository](https://github.com/paritytech/substrate/tree/master/frame) and a
template pallet that is [defined in the `pallets`](./pallets/template/src/lib.rs) directory.

A FRAME pallet is compromised of a number of blockchain primitives:

-   Storage: FRAME defines a rich set of powerful
    [storage abstractions](https://substrate.dev/docs/en/knowledgebase/runtime/storage) that makes
    it easy to use Substrate's efficient key-value database to manage the evolving state of a
    blockchain.
-   Dispatchables: FRAME pallets define special types of functions that can be invoked (dispatched)
    from outside of the runtime in order to update its state.
-   Events: Substrate uses [events](https://substrate.dev/docs/en/knowledgebase/runtime/events) to
    notify users of important changes in the runtime.
-   Errors: When a dispatchable fails, it returns an error.
-   Config: The `Config` configuration interface is used to define the types and parameters upon
    which a FRAME pallet depends.

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command (`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/node-template --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/node-template purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```


## Use Nodejs to access nftmart blockchain

```shell
git clone https://github.com/NFTT-studio/NFTMart-Node.git
cd nftmart/nodejs-demo
yarn install

# Create a class(ID: 0) by Alice with local testing node.
node nft-apis.mjs --ws ws://127.0.0.1:9944 create-class //Alice

# Add a new class administrator to the class, ID: 0
node nft-apis.mjs --ws ws://127.0.0.1:9944 add-class-admin //Bob

# Create an another class(ID: 1) managed by Alice.
node nft-apis.mjs --ws ws://127.0.0.1:9944 create-class //Alice

# List all classes
node nft-apis.mjs --ws ws://127.0.0.1:9944 show-class-info
[
  '{"metadata":"https://xx.com/aa.jpg","totalIssuance":0,"owner":"62qUEaQwPx7g4vDz88bdp1tmZkSpPtVRL4pS98P7VEbZnM9w","data":{"deposit":2280000000000,"properties":3,"name":"0x616161","description":"0x62626262","createBlock":489},"classID":1,"adminList":[[{"delegate":"65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB","proxyType":"Any","delay":0}],261000000000000]}',
  '{"metadata":"https://xx.com/aa.jpg","totalIssuance":0,"owner":"62qUEaQwPx7g4vDz88bN4zMBTFmcwLPYbPsvbBhH2QiqWhfB","data":{"deposit":2280000000000,"properties":3,"name":"0x616161","description":"0x62626262","createBlock":8},"classID":0,"adminList":[[{"delegate":"63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw","proxyType":"Any","delay":0},{"delegate":"65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB","proxyType":"Any","delay":0}],459000000000000]}'
]

# Mint three nft tokens to Bob in the class which ID is 0.
node nft-apis.mjs --ws ws://127.0.0.1:9944 mint-nft //Bob 0

# List all nfts in the class, `ID:0`
node nft-apis.mjs --ws ws://127.0.0.1:9944 show-all-nfts 0
{"metadata":"0x6161626263636464","owner":"63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw","data":{"deposit":1080000000000,"createBlock":554}}
{"metadata":"0x6161626263636464","owner":"63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw","data":{"deposit":1080000000000,"createBlock":554}}
{"metadata":"0x6161626263636464","owner":"63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw","data":{"deposit":1080000000000,"createBlock":554}}

```
