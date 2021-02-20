use sp_core::{Pair, Public, sr25519};
use node_template_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, GenesisConfigModuleConfig, WASM_BINARY, Signature
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::ChainType;
use hex_literal::hex;

// Note this is the URL for the telemetry server
const TAO_STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
				authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
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
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		pallet_aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		}),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key,
		}),
		pallet_genesis_config: Some(GenesisConfigModuleConfig {
			something: 9,
			something_two: 10,
			some_account_value: endowed_accounts.iter().cloned().map(|k|(k, 2)).collect(),
		})
	}
}


fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online }
}


// public staging network
pub fn mrpai_staging_testnet_config() -> ChainSpec {
	let boot_nodes = vec![];

	ChainSpec::from_genesis(
		"mrpai Staging Testnet",
		"mrpai_staging",
		ChainType::Live,
		mrpai_staging_testnet_genesis,
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(TAO_STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("Westend Staging telemetry url is valid; qed")
		),
		Some("mrpai_staging"),
		None,
		Default::default(),
	)
}

fn mrpai_staging_testnet_genesis() -> GenesisConfig {
	// subkey inspect "$SECRET"
	let endowed_accounts = vec![
		// 5CQGWJrYe6Jkv8V4nZs3DeVrRkiATyXMCT1xNhvhbYroR9zf
		hex!["0eeb91a2e70aa9c814355b4490fc05c7c2f09094025c34b1c72a4756c1d78f3a"].into(),
		// 5DhJmqzx3PrD1ZbS3RZkYC69xbJMA1X6vStbgbWQ1J4Xbi1n
		hex!["48269bcbea5d6d6545ff4591c010c9646a55569ec256522b9e94fd7cb639ed5f"].into(),
		// 5Gc4weP5S1Z3rEN4hyxwLFaX78CoamSTyxx5cJHPzSEDEn5o
		hex!["c8dc85c1ac2b6275cd2cf676eb4f662a6e13a6e2b4dd7fcb660d14fc84f6a101"].into(),
		// 5D2qzhvUtYAqxgobC8h9CTeVSimSQszf5ZLvc2J2PUyNXmmj
		hex!["2ad123b94031cf587435093cda28b1507b08354f4440d0892e52cc97812ce97a"].into(),
	];

	// for i in 1 2 3 4; do for j in stash controller; do subkey inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in babe; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in grandpa; do subkey --ed25519 inspect "$SECRET//$i//$j"; done; done
	// for i in 1 2 3 4; do for j in im_online; do subkey --sr25519 inspect "$SECRET//$i//$j"; done; done
	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
	)> = vec![(
		// 
		hex!["ac1c1cd139fd2a03b680fac4f2b329177854ba60f8b880f0b26f87182d0d0828"].into(),
		// 
		hex!["ccfc3fa11670dbee9467902037b850e01282c0691f013bf91028904faabd367f"].into(),
		// 
		hex!["1e4d0fae5601d9d2c8dbeb872c7da34c8d83a8afbde0ea5e0faa723c989d7a12"].unchecked_into(),
		// 
		hex!["6e0d2f92f1544705e8ecd3f911e76431d6057f8458f7b772a3deba3f57a8c077"].unchecked_into(),
		// 
		hex!["2019c25be28dd34622a6537e53aefd9ced06164271e2aecb5f665cdce2e5a706"].unchecked_into(),
	),(
		// 
		hex!["00eb48e0a7e603d6cc2bc92a2774883b2cb7ad2aa06211270ad193c201d6b32e"].into(),
		// 
		hex!["bc320125ea7364ec2085044ae5a03685d8aa72a8636545c0a6fbec1f643ac577"].into(),
		// 
		hex!["f429f0c5bb3828e44a9b2847ab25db2441449dfba0ff074ccfb6eebfe3d07a28"].unchecked_into(),
		// 
		hex!["e87deee76c0c5ece12606672f46b32f9f9d0e600e4d8d5117b7baae408f838c1"].unchecked_into(),
		// 
		hex!["22a4d14485852bea6fa11056c39ca5c5423e4fc80e61e6794e04d2f03c0f9f66"].unchecked_into(),
	),(
		// 
		hex!["889345d153750ea523ccace94fc61a079733526690ddc64d04252f22dc45650e"].into(),
		// 
		hex!["d25e30e262c90fe26d084afe96c71f1b87a01f2b4cb6899f7b0862aad533f17a"].into(),
		// 
		hex!["9c10cefceeb0e63ef5e66829ff35d4d97162649836575c5b4fa047b2c62e3f40"].unchecked_into(),
		// 
		hex!["e6324db0e7a39d32d31a141b1bc1fec09d3667dbc8ee0b8ad58ef6ad412d753e"].unchecked_into(),
		// 
		hex!["86da83002911f076e2f84ba171280ce437443eb83c30db648428c9901af61d32"].unchecked_into(),
	),(
		// 
		hex!["1a0bc9e9f516f07c17c541f05f9b851bf2d4a95ab937654aaa60c7137284cd5d"].into(),
		// 
		hex!["0ca3343aa49c2aa7aec9485bc64d6b1284360f0552c05a2254fbf84707ae000f"].into(),
		// 
		hex!["d61ce6db89aa66eb6869d2406e286e80378ca1cc19d1e8086bebb1863c938402"].unchecked_into(),
		// 
		hex!["62c4dd07ce2e41423b82b42146509c758e4039d51432fe8657df7dce834f4350"].unchecked_into(),
		// 
		hex!["dc47f67f7d8ed709ad4ae71a6eda8ac9a509a774c9a302fbaf2bee5136cb3b6b"].unchecked_into(),
	)];
	
	const ENDOWMENT: u128 = 1_000_000 * DOLLARS;
	const STASH: u128 = 100 * DOLLARS;
	let num_endowed_accounts = endowed_accounts.len();
	
	GenesisConfig {
		system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: endowed_accounts.iter()
				.map(|k: &AccountId| (k.clone(), ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		}),
		babe: Some(BabeConfig {
			authorities: vec![],
		}),
		grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		sudo: Some(SudoConfig {
			key: endowed_accounts[0].clone(),
		}),
		session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(
					x.0.clone(),
					x.0.clone(),
					session_keys(x.2.clone(), x.3.clone(), x.4.clone())
				)
			}).collect::<Vec<_>>(),
		}),
		staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: Forcing::ForceNone,
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		}),
		im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		democracy: Some(DemocracyConfig::default()),
		elections_phragmen: Some(ElectionsConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.map(|member| (member, STASH))
						.collect(),
		}),
		collective_Instance1: Some(CouncilConfig::default()),
		collective_Instance2: Some(TechnicalCommitteeConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.collect(),
			phantom: Default::default(),
		}),
		membership_Instance1: Some(Default::default()),
		treasury: Some(Default::default()),
	}
}
