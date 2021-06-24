// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use sc_chain_spec::ChainSpecExtension;
use sp_core::{Pair, Public, crypto::UncheckedInto, sr25519};
use serde::{Serialize, Deserialize};
use node_runtime::{
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, CouncilConfig,
	DemocracyConfig, GrandpaConfig, ImOnlineConfig, SessionConfig, SessionKeys, StakerStatus,
	StakingConfig, ElectionsConfig, IndicesConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, wasm_binary_unwrap, MAX_NOMINATIONS,
	TokensConfig, OrmlNFTConfig,
};
use node_runtime::Block;
use node_runtime::constants::currency::*;
use sc_service::ChainType;
use hex_literal::hex;
use sc_telemetry::TelemetryEndpoints;
use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};
use sp_std::vec::Vec;

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<
	GenesisConfig,
	Extensions,
>;

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> GenesisConfig {
  let root_key: AccountId = hex!["12970155d02df21b7e39e289593065d0bbb67d5d38f36dd1b9d617614a006d00"].into(); // 5znMeMdGsDrENMFg9wvLMveuYdCSVCCGdaXE6HAU4UwTksei
  let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
    (
      hex!["d0a9b0c9ac0a3dc0432cb66f288c1ffc9bd159ca52739d994f789003b08b6630"].into(),              // 655aHzD3sX1QpZVxStEHPV4TVCqKVcfwfxrsX8spZndPfabe
      hex!["c43b6cda18d09359fe32ea27014601c6d723e17e2cc8ca14496f210595f95a26"].into(),              // 64oGxqAX2AW26AWQDx9vNNb7aTF741QMTn1n35qFRty6FaLc
      hex!["184f5672c5f405f12476c29ba35ab22fdf44f4e50d671802cb271f06adb5cb3f"].unchecked_into(),    // 5zureDa91LCdspDmqxkPUnGg9WLHPJQLs1XZ9uqmkUEcK3Ca
      hex!["2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"].unchecked_into(),    // 6167FvHPZP7MrPZbJKkwXbxZSupoRmDcAt5RhC1B2NuC2D6G
      hex!["2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"].unchecked_into(),    // 6167FvHPZP7MrPZbJKkwXbxZSupoRmDcAt5RhC1B2NuC2D6G
      hex!["2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"].unchecked_into(),    // 6167FvHPZP7MrPZbJKkwXbxZSupoRmDcAt5RhC1B2NuC2D6G
    ),
    (
      hex!["5e7704ab35a8a08fda1ca9ddca87013849daf02744e81cc5fb03d7395030744c"].into(),              // 62VqnJu5Xwc5qaNsQoeS8UAEA8rFFf8U6UeyeKgYQGfi23us
      hex!["c23b0e2abab64d27c630028830d5a3afc4785f0dd02ce069af8b3f2118bc682c"].into(),              // 64kekuPLYqkAHwwbeYjVUDkPFoc27VNGib3ezJrXCTY2qWSm
      hex!["b46c28b4f0db186814fe579e63d2e9b7c3dbb6c1f28dfe541a6cc11ccfc5fa3e"].unchecked_into(),    // 64SYg4L1MbtsREC8Qcrd42bMidA8bXq9jmNBYDwAg1fcuBm4
      hex!["0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"].unchecked_into(),    // 5zTqxMT5SG1gsH7SrM5dn8nmi1Cp8R3U9sBU6E1jBKfLLzrv
      hex!["0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"].unchecked_into(),    // 5zTqxMT5SG1gsH7SrM5dn8nmi1Cp8R3U9sBU6E1jBKfLLzrv
      hex!["0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"].unchecked_into(),    // 5zTqxMT5SG1gsH7SrM5dn8nmi1Cp8R3U9sBU6E1jBKfLLzrv
    ),
  ];

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(initial_authorities, vec![], root_key, Some(endowed_accounts))
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let mut prop = sc_service::Properties::new();
	prop.insert("tokenDecimals".to_string(), 12.into());
	prop.insert("tokenSymbol".to_string(), "NMT".into()); // NFT Mart Token
	let boot_nodes = vec![];
	ChainSpec::from_genesis(
		"Nftmart Staging",
		"nftmart_staging",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Staging telemetry url is valid; qed")),
		Some("nftmart"),
		Some(prop),
		Default::default(),
	)
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
	AccountId,
	AccountId,
	GrandpaId,
	BabeId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});
	// endow all authorities and nominators.
	initial_authorities.iter().map(|x| &x.0).chain(initial_nominators.iter()).for_each(|x| {
		if !endowed_accounts.contains(&x) {
			endowed_accounts.push(x.clone())
		}
	});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (MAX_NOMINATIONS as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.into_iter()
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;

	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|x| (x, ENDOWMENT))
				.collect()
		},
		indices: IndicesConfig {
			indices: vec![],
		},
		session: SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
				))
			}).collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			.. Default::default()
		},
		democracy: DemocracyConfig::default(),
		elections: ElectionsConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.map(|member| (member, STASH))
						.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.collect(),
			phantom: Default::default(),
		},
		sudo: SudoConfig {
			key: root_key,
		},
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(node_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		im_online: ImOnlineConfig {
			keys: vec![],
		},
		authority_discovery: AuthorityDiscoveryConfig {
			keys: vec![],
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		technical_membership: Default::default(),
		treasury: Default::default(),
		tokens: TokensConfig {
			endowed_accounts: endowed_accounts.iter()
				.flat_map(|x|{
					vec![
						(x.clone(), 2, 100 * nftmart_traits::constants_types::ACCURACY),
						(x.clone(), 3, 100 * nftmart_traits::constants_types::ACCURACY),
						(x.clone(), 4, 100 * nftmart_traits::constants_types::ACCURACY),
					]
				}).collect(),
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
		nftmart: Default::default(),
		nftmart_order: Default::default(),
		nftmart_conf: node_runtime::NftmartConfConfig {
			white_list: endowed_accounts,
			..Default::default()
		},
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
		],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	let mut prop = sc_service::Properties::new();
	prop.insert("tokenDecimals".to_string(), 12.into());
	prop.insert("tokenSymbol".to_string(), "NMT".into()); // NFT Mart Token
	ChainSpec::from_genesis(
		"Nftmart Testnet",
		"nftmart_testnet",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		Some("nftmart"),
		Some(prop),
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
		],
		vec![],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	let mut prop = sc_service::Properties::new();
	prop.insert("tokenDecimals".to_string(), 12.into());
	prop.insert("tokenSymbol".to_string(), "NMT".into()); // NFT Mart Token
	ChainSpec::from_genesis(
		"Nftmart Testnet",
		"nftmart_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Local Testnet telemetry url is valid; qed")),
		Some("nftmart"),
		Some(prop),
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, new_light_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![
				authority_keys_from_seed("Alice"),
			],
			vec![],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sc_service_test::connectivity(
			integration_test_config_with_two_authorities(),
			|config| {
				let NewFullBase { task_manager, client, network, transaction_pool, .. }
					= new_full_base(config,|_, _| ())?;
				Ok(sc_service_test::TestNetComponents::new(task_manager, client, network, transaction_pool))
			},
			|config| {
				let (keep_alive, _, client, network, transaction_pool) = new_light_base(config)?;
				Ok(sc_service_test::TestNetComponents::new(keep_alive, client, network, transaction_pool))
			}
		);
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		staging_testnet_config().build_storage().unwrap();
	}
}
