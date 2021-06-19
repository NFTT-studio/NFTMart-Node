#!/usr/bin/env zsh

# Make sure we are in the right place.
if [ ! -f ./shell.nix ]; then
	exit 1
fi
if [ ! -f ./ss58-registry.json ]; then
	exit 1
fi

local NFTMARTROOT=$(pwd)
cd $NFTMARTROOT/target/release
rm -f staging_spec_raw.json
cp $NFTMARTROOT/bin/node/cli/res/staging_spec_raw.json .
cd $NFTMARTROOT

#p2p node1:
#12D3KooWH9cPA6NWqhdmxNcXFrETwxteKZsta5baAmVgHqNY9VMC
#66aff6b1ad5902bc77b120ad8cf47d7e04a23d6504d97cbbceabfcedc8deaa7c
#p2p node2:
#12D3KooWRxU9AmmTUCRYTzNXx1V6mvbhL2aERPRvoE2QTG4KsVwj
#8608245741558c0a41c3b1704aedbf7385cf900fc34f561113f15bc1341b875e
rm -rf target/release/node1
target/release/substrate \
 -d target/release/node1 \
 -lruntime=debug \
 --node-key 66aff6b1ad5902bc77b120ad8cf47d7e04a23d6504d97cbbceabfcedc8deaa7c \
 --execution=NativeElseWasm \
 --rpc-cors=all \
 --rpc-methods=Unsafe \
 --unsafe-ws-external \
 --chain target/release/staging_spec_raw.json \
 --validator \
 --wasm-execution Interpreted \
 --ws-max-connections=10000 \
 --port 30333 \
 --ws-port 9944 \
 --rpc-port 9933 \
 --no-prometheus
