#!/usr/bin/env zsh

# Make sure we are in the right place.
if [ ! -f ./shell.nix ]; then
	exit 1
fi
if [ ! -f ./ss58-registry.json ]; then
	exit 1
fi

echo '/*'
local root_json=$(target/release/substrate key generate --network nftmart --output-type Json | jq -c)
echo '#'root: "$root_json"

local session_json=$(target/release/substrate key generate --network nftmart --output-type Json | jq -c)
echo '#'session: "$session_json"

local root_sk=$(echo "$root_json" | jq -r ".secretPhrase")
local root_pk=$(echo "$root_json" | jq -r ".publicKey")
local root_addr=$(echo "$root_json" | jq -r ".ss58Address")

local session_sk=$(echo "$session_json" | jq -r ".secretPhrase")
local session_pk=$(echo "$session_json" | jq -r ".publicKey")
local session_addr=$(echo "$session_json" | jq -r ".ss58Address")

local seq=("${@:1}")

for i in $seq; do
	for j in stash controller; do
		local json=$(target/release/substrate key inspect-key --network nftmart "$root_sk"//nftmart/"$j"/"$i" --output-type Json | jq -c)
		echo '#'"$j""$i": "$json"
	done
done

for i in $seq; do
	echo 'local ip_of_node'"${i}"'=11.22.33.4'"${i}"
done

for i in $seq; do
	local json_grandpa=$(target/release/substrate key inspect-key --scheme Ed25519 --network nftmart "${session_sk}"//nftmart//grandpa//${i} --output-type Json | jq -c)
	local json_babe=$(target/release/substrate key inspect-key --scheme Sr25519 --network nftmart "${session_sk}"//nftmart//babe//"${i}" --output-type Json | jq -c)
	echo '#'grandpa"${i}": "$json_grandpa"
	echo '#'babe"${i}": "$json_babe"
	local pk_grandpa=$(echo "${json_grandpa}" | jq -r ".publicKey")
	local pk_babe=$(echo "${json_babe}" | jq -r ".publicKey")
	echo "ssh -o IdentitiesOnly=yes root@\$ip_of_node${i} <<'EOF'"
	echo curl --header '"''Content-Type:application/json;charset=utf-8''"' --request POST --data "'"{'"'jsonrpc'"':'"'2.0'"', '"'id'"':1, '"'method'"':'"'author_insertKey'"', '"'params'"': '[''"'gran'"', '"'${session_sk}//nftmart//grandpa//${i}'"', '"'${pk_grandpa}'"'']'}"'" http://localhost:9933
	echo curl --header '"''Content-Type:application/json;charset=utf-8''"' --request POST --data "'"{'"'jsonrpc'"':'"'2.0'"', '"'id'"':1, '"'method'"':'"'author_insertKey'"', '"'params'"': '[''"'babe'"', '"'${session_sk}//nftmart//babe//${i}'"', '"'${pk_babe}'"'']'}"'" http://localhost:9933
	echo curl --header '"''Content-Type:application/json;charset=utf-8''"' --request POST --data "'"{'"'jsonrpc'"':'"'2.0'"', '"'id'"':1, '"'method'"':'"'author_insertKey'"', '"'params'"': '[''"'imon'"', '"'${session_sk}//nftmart//babe//${i}'"', '"'${pk_babe}'"'']'}"'" http://localhost:9933
	echo curl --header '"''Content-Type:application/json;charset=utf-8''"' --request POST --data "'"{'"'jsonrpc'"':'"'2.0'"', '"'id'"':1, '"'method'"':'"'author_insertKey'"', '"'params'"': '[''"'audi'"', '"'${session_sk}//nftmart//babe//${i}'"', '"'${pk_babe}'"'']'}"'" http://localhost:9933
	echo 'EOF'
done

for i in $seq; do
	echo p2p node$i:
	target/release/substrate key generate-node-key 2>&1
	echo
done
echo '*/'

echo let root_key: 'AccountId = hex!["'${root_pk#0x}'"].into();' // ${root_addr}
for i in $seq; do
	for j in stash controller; do
		local json=$(target/release/substrate key inspect-key --network nftmart "${root_sk}"//nftmart/"${j}"/"${i}" --output-type Json | jq -c)
		local pk=$(echo "${json}" | jq -r ".publicKey")
		local addr=$(echo "${json}" | jq -r ".ss58Address")
		echo let "${j}""${i}": 'AccountId = hex!["'${pk#0x}'"].into();' // "${addr}"
	done
done

echo '  let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec!['
for i in $seq; do
	local json_grandpa=$(target/release/substrate key inspect-key --scheme Ed25519 --network nftmart "${session_sk}"//nftmart//grandpa//"${i}" --output-type Json | jq -c)
	local json_babe=$(target/release/substrate key inspect-key --scheme Sr25519 --network nftmart "${session_sk}"//nftmart//babe//"${i}" --output-type Json | jq -c)
	local pk_grandpa=$(echo "${json_grandpa}" | jq -r ".publicKey")
	local pk_babe=$(echo "${json_babe}" | jq -r ".publicKey")
	echo '('stash"${i}", controller"${i}",
	echo 'hex!["'${pk_grandpa#0x}'"].unchecked_into(),' // "$(echo "${json_grandpa}" | jq -r '.ss58Address')"
	echo 'hex!["'${pk_babe#0x}'"].unchecked_into(),' // "$(echo "${json_babe}" | jq -r '.ss58Address')"
	echo 'hex!["'${pk_babe#0x}'"].unchecked_into(),' // "$(echo "${json_babe}" | jq -r '.ss58Address')"
	echo 'hex!["'${pk_babe#0x}'"].unchecked_into(),),' // "$(echo "${json_babe}" | jq -r '.ss58Address')"
done
echo '];'
