use aisland_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY,
};
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_core::crypto::UncheckedInto;
use sp_core::OpaquePeerId; // A struct wraps Vec<u8> to represent the node `PeerId`.
use aisland_runtime::NodeAuthorizationConfig; // The genesis config that serves the pallet.
use hex_literal::hex;
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
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					hex!("dc31445d24993e946ebf9f444dd17a9698fe859eeb574b78910100baab083b75").into(),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork id
		None,
		// Properties
		Some(aisland_properties()),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
				authority_keys_from_seed("Alice"), 
				authority_keys_from_seed("Bob"),
				],
				// Sudo account
				hex!("001a667f2603ce2cb86703796aff2372c5a78ddeef0ff9b540d9ad745c254447").into(),
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
			)
		},
		// Bootnodes
		vec![
            "/ip4/65.108.62.72/tcp/30333/p2p/12D3KooWPu79TFZHuZYU78mi72C1e8Dk37ot69Um8atNNiz9Hm2R"
                .parse()
                .unwrap(),
            "/ip4/94.130.184.125/tcp/30333/p2p/12D3KooWJRVgD2b3NDf6Jph9Vt2VCubxkEbkLiGf6YLQsH1UeSiE"
                .parse()
                .unwrap(),
            "/ip4/94.130.183.49/tcp/30333/p2p/12D3KooWSecRjwjJ6CFJLtCNacEzWBV2S46vrHD1DcC491fz13Ut"
                .parse()
                .unwrap(),
        ],
		// Telemetry
		None,
		// Protocol ID
		None,
		// ??
		None,
		// Properties
		Some(aisland_properties()),
		// Extensions
		None,
	))
}
pub fn public_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Aisland Testnet",
		// ID
		"aisland_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities (Aura,Grandpa)
				vec![
					(
					 hex!("da7c3f90a9ac2907026f909525fe59a36283e1a359c9cecf8a3e5d02a5b65a5c").unchecked_into(),
					 hex!("bec120902cdf3e20fcb2155abedf422a127f61c4fa1ffe2b56a0a8b374cacc2d").unchecked_into()
					),
										(
					 hex!("a65840c94f1bf99b95437300da9a93dd31655ac829dd46d098c31be1344ddd52").unchecked_into(),
					 hex!("35d9c81770177e172036b11560365088e23b4cf486b4814935eb396953277fa2").unchecked_into()
					),
										(
					 hex!("604c1792783545e4f11bc95f9b69fb16baebb6734d8d42f3f29502e13a21835b").unchecked_into(),
					 hex!("72a1bfc72dba31c0b91b3d2b8a07a569c758173325c943681c775730b0fde189").unchecked_into()
					)
				],
				// Sudo account
				hex!("001a667f2603ce2cb86703796aff2372c5a78ddeef0ff9b540d9ad745c254447").into(),

				// Pre-funded accounts
				vec![
					hex!("9ad611a1a67fcf50f16be2650316d3ef976452bc32eaee8caca2da485bf40202").into(),
					hex!("1ec3154ebd4d0fc993c91c04b22f402d863f360fef026c80debf7fee4cdc7e68").into(),
				],
				true,
			)
		},
		// Bootnodes
		vec![
            "/ip4/65.108.62.72/tcp/30333/p2p/12D3KooWJ4YEdFmRayeeXqdES4D6u4vQGHvYsxTsTvSB59STj8dz"
                .parse()
                .unwrap(),
            "/ip4/94.130.184.125/tcp/30333/p2p/12D3KooWJRVgD2b3NDf6Jph9Vt2VCubxkEbkLiGf6YLQsH1UeSiE"
                .parse()
                .unwrap(),
            "/ip4/94.130.183.49/tcp/30333/p2p/12D3KooWRSNSv85rsJsoWQ3JgAAcQfDg6aGek1ygW3Eyed7u9W6L"
                .parse()
                .unwrap(),
	        ],
		// Telemetry
		None,
		// Protocol ID
		None,
		// ??
		None,
		// Properties
		Some(aisland_properties()),
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
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		node_authorization: NodeAuthorizationConfig {
			nodes: vec![
			  (
				OpaquePeerId(bs58::decode("12D3KooWJ4YEdFmRayeeXqdES4D6u4vQGHvYsxTsTvSB59STj8dz").into_vec().unwrap()),
				endowed_accounts[0].clone()
			  ),
			  (
				OpaquePeerId(bs58::decode("12D3KooWJRVgD2b3NDf6Jph9Vt2VCubxkEbkLiGf6YLQsH1UeSiE").into_vec().unwrap()),
				endowed_accounts[1].clone()
			  ),
			  (
				OpaquePeerId(bs58::decode("12D3KooWRSNSv85rsJsoWQ3JgAAcQfDg6aGek1ygW3Eyed7u9W6L").into_vec().unwrap()),
				endowed_accounts[1].clone()
			  ),
			],
		  },
		
	}
}

/// Token
pub fn aisland_properties() -> Properties {
    let mut p = Properties::new();
    p.insert("ss58format".into(), 42.into());
    p.insert("tokenDecimals".into(), 18.into());
    p.insert("tokenSymbol".into(), "AISC".into());
    p
}
