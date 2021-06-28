#!/usr/bin/env zsh

# Make sure we are in the right place.
if [ ! -f ./shell.nix ]; then
	exit 1
fi
if [ ! -f ./Cargo.toml ]; then
	exit 1
fi

rm -rf target/release/node2
target/release/nftmart-node \
 -d target/release/node2 \
 -lruntime=debug \
 --node-key 8608245741558c0a41c3b1704aedbf7385cf900fc34f561113f15bc1341b875e \
 --execution=NativeElseWasm \
 --rpc-cors=all \
 --rpc-methods=Unsafe \
 --unsafe-ws-external \
 --chain target/release/staging_spec_raw.json \
 --validator \
 --wasm-execution Interpreted \
 --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWH9cPA6NWqhdmxNcXFrETwxteKZsta5baAmVgHqNY9VMC \
 --ws-max-connections=10000 \
 --port 30334 \
 --ws-port 9945 \
 --rpc-port 9934 \
 --no-prometheus
