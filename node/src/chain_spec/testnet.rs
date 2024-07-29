// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use super::*;

pub fn finney_testnet_config() -> Result<ChainSpec, String> {
    let path: PathBuf = std::path::PathBuf::from("./snapshot.json");
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    // We mmap the file into memory first, as this is *a lot* faster than using
    // `serde_json::from_reader`. See https://github.com/serde-rs/json/issues/160
    let file = File::open(&path)
        .map_err(|e| format!("Error opening genesis file `{}`: {}", path.display(), e))?;

    // SAFETY: `mmap` is fundamentally unsafe since technically the file can change
    //         underneath us while it is mapped; in practice it's unlikely to be a problem
    let bytes = unsafe {
        memmap2::Mmap::map(&file)
            .map_err(|e| format!("Error mmaping genesis file `{}`: {}", path.display(), e))?
    };

    let old_state: ColdkeyHotkeys =
        json::from_slice(&bytes).map_err(|e| format!("Error parsing genesis file: {e}"))?;

    let mut processed_stakes: Vec<(
        sp_runtime::AccountId32,
        Vec<(sp_runtime::AccountId32, (u64, u16))>,
    )> = Vec::new();
    for (coldkey_str, hotkeys) in old_state.stakes.iter() {
        let coldkey = <sr25519::Public as Ss58Codec>::from_ss58check(coldkey_str)
            .map_err(|e| e.to_string())?;
        let coldkey_account = sp_runtime::AccountId32::from(coldkey);

        let mut processed_hotkeys: Vec<(sp_runtime::AccountId32, (u64, u16))> = Vec::new();

        for (hotkey_str, amount_uid) in hotkeys.iter() {
            let (amount, uid) = amount_uid;
            let hotkey = <sr25519::Public as Ss58Codec>::from_ss58check(hotkey_str)
                .map_err(|e| e.to_string())?;
            let hotkey_account = sp_runtime::AccountId32::from(hotkey);

            processed_hotkeys.push((hotkey_account, (*amount, *uid)));
        }

        processed_stakes.push((coldkey_account, processed_hotkeys));
    }

    let mut balances_issuance: u64 = 0;
    let mut processed_balances: Vec<(sp_runtime::AccountId32, u64)> = Vec::new();
    for (key_str, amount) in old_state.balances.iter() {
        let key =
            <sr25519::Public as Ss58Codec>::from_ss58check(key_str).map_err(|e| e.to_string())?;
        let key_account = sp_runtime::AccountId32::from(key);

        processed_balances.push((key_account, *amount));
        balances_issuance = balances_issuance
            .checked_add(*amount)
            .ok_or("Balances issuance overflowed".to_string())?;
    }

    // Give front-ends necessary data to present to users
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    Ok(ChainSpec::builder(
        wasm_binary,
        Extensions {
            bad_blocks: Some(HashSet::new()),
            ..Default::default()
        },
    )
    .with_name("Bittensor")
    .with_boot_nodes(vec![
        "/dns/bootnode.test.chain.opentensor.ai/tcp/30333/p2p/12D3KooWPM4mLcKJGtyVtkggqdG84zWrd7Rij6PGQDoijh1X86Vr"
            .parse()
            .unwrap(),
    ])
    .with_protocol_id("bittensor")
    .with_id("bittensor")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(testnet_genesis(
        // Initial PoA authorities (Validators)
        // aura | grandpa
        vec![
            // Keys for debug
            //authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob"),
            authority_keys_from_ss58(
                "5D5ABUyMsdmJdH7xrsz9vREq5eGXr5pXhHxix2dENQR62dEo",
                "5H3qMjQjoeZxZ98jzDmoCwbz2sugd5fDN1wrr8Phf49zemKL",
            ), // key 1
            authority_keys_from_ss58(
                "5GbRc5sNDdhcPAU9suV2g9P5zyK1hjAQ9JHeeadY1mb8kXoM",
                "5GbkysfaCjK3cprKPhi3CUwaB5xWpBwcfrkzs6FmqHxej8HZ",
            ), // key 1
            authority_keys_from_ss58(
                "5CoVWwBwXz2ndEChGcS46VfSTb3RMUZzZzAYdBKo263zDhEz",
                "5HTLp4BvPp99iXtd8YTBZA1sMfzo8pd4mZzBJf7HYdCn2boU",
            ), // key 1
            authority_keys_from_ss58(
                "5EekcbqupwbgWqF8hWGY4Pczsxp9sbarjDehqk7bdyLhDCwC",
                "5GAemcU4Pzyfe8DwLwDFx3aWzyg3FuqYUCCw2h4sdDZhyFvE",
            ), // key 1
            authority_keys_from_ss58(
                "5GgdEQyS5DZzUwKuyucEPEZLxFKGmasUFm1mqM3sx1MRC5RV",
                "5EibpMomXmgekxcfs25SzFBpGWUsG9Lc8ALNjXN3TYH5Tube",
            ), // key 1
            authority_keys_from_ss58(
                "5Ek5JLCGk2PuoT1fS23GXiWYUT98HVUBERFQBu5g57sNf44x",
                "5Gyrc6b2mx1Af6zWJYHdx3gwgtXgZvD9YkcG9uTUPYry4V2a",
            ), // key 1
        ],
        // Sudo account
        Ss58Codec::from_ss58check("5GpzQgpiAKHMWNSH3RN4GLf96GVTDct9QxYEFAY7LWcVzTbx").unwrap(),
        // Pre-funded accounts
        vec![],
        true,
        processed_stakes.clone(),
        processed_balances.clone(),
        balances_issuance,
    ))
    .with_properties(properties)
    .build())
}

// Configure initial storage state for FRAME modules.
#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _root_key: AccountId,
    _endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
    _stakes: Vec<(AccountId, Vec<(AccountId, (u64, u16))>)>,
    _balances: Vec<(AccountId, u64)>,
    _balances_issuance: u64,
) -> serde_json::Value {
    serde_json::json!({
        "balances": {
            // Configure sudo balance
            "balances": vec![(
                <AccountId32 as Ss58Codec>::from_ss58check("5GpzQgpiAKHMWNSH3RN4GLf96GVTDct9QxYEFAY7LWcVzTbx")
                    .unwrap(),
                1000000000000u128,
            )],
        },
        "aura": {
            "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>(),
        },
        "grandpa": {
            "authorities": initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect::<Vec<_>>(),
        },
        "sudo": {
            "key": Some(
                <AccountId32 as Ss58Codec>::from_ss58check("5GpzQgpiAKHMWNSH3RN4GLf96GVTDct9QxYEFAY7LWcVzTbx")
                    .unwrap(),
            ),
        },
    })
}
