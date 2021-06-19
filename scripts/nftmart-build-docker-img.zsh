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

tee Dockerfile <<-'EOF'
FROM ubuntu:20.04
WORKDIR /data
ADD substrate /usr/bin/substrate
ADD staging_spec_raw.json /root
EXPOSE 9944/tcp
EXPOSE 30333/tcp
EOF
docker build -t nftmart -f Dockerfile ./

rm -f staging_spec_raw.json Dockerfile
docker image prune --force


