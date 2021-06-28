#!/usr/bin/env zsh

# Make sure we are in the right place.
if [ ! -f ./shell.nix ]; then
	exit 1
fi
if [ ! -f ./Cargo.toml ]; then
	exit 1
fi

rm -rf target/release/node3
target/release/nftmart-node \
 -d target/release/node3 \
 -lruntime=debug \
 --execution=NativeElseWasm \
 --rpc-cors=all \
 --rpc-methods=Unsafe \
 --unsafe-ws-external \
 --chain target/release/staging_spec_raw.json \
 --validator \
 --wasm-execution Interpreted \
 --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWH9cPA6NWqhdmxNcXFrETwxteKZsta5baAmVgHqNY9VMC \
 --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWRxU9AmmTUCRYTzNXx1V6mvbhL2aERPRvoE2QTG4KsVwj \
 --ws-max-connections=10000 \
 --port 30335 \
 --ws-port 9946 \
 --rpc-port 9935 \
 --no-prometheus
