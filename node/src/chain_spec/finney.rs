// Allowed since it's actually better to panic during chain setup when there is an error
#![allow(clippy::unwrap_used)]

use super::*;

pub fn finney_mainnet_config() -> Result<ChainSpec, String> {
    let path: PathBuf = std::path::PathBuf::from("./snapshot.json");
    let wasm_binary = WASM_BINARY.ok_or("Development wasm not available".to_string())?;

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
        json::from_slice(&bytes).map_err(|e| format!("Error parsing genesis file: {}", e))?;

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
    .with_protocol_id("bittensor")
    .with_id("bittensor")
    .with_chain_type(ChainType::Live)
	.with_boot_nodes(vec![
        "/dns/bootnode.finney.chain.opentensor.ai/tcp/30333/ws/p2p/12D3KooWRwbMb85RWnT8DSXSYMWQtuDwh4LJzndoRrTDotTR5gDC"
            .parse()
            .unwrap(),
    ])
    .with_genesis_config_patch(finney_genesis(
        // Initial PoA authorities (Validators)
        // aura | grandpa
        vec![
            // Keys for debug
            authority_keys_from_ss58(
                "5EJUcFbe74FDQwPsZDbRVpdDxVZQQxjoGZA9ayJqJTbcRrGf",
                "5GRcfchgXZjkCfqgNvfjicjJw3vVGF4Ahqon2w8RfjXwyzy4",
            ), // key 1
            authority_keys_from_ss58(
                "5H5oVSbQxDSw1TohAvLvp9CTAua6PN4yHme19UrG4c1ojS8J",
                "5FAEYaHLZmLRX4XFs2SBHbLhkysbSPrcTp51w6sQNaYLa7Tu",
            ), // key 2
            authority_keys_from_ss58(
                "5CfBazEwCAsmscGj1J9rhXess9ZXZ5qYcuZvFWii9sxT977v",
                "5F6LgDAenzchE5tPmFHKGueYy1rj85oB2yxvm1xyKLVvk4gy",
            ), // key 3
            authority_keys_from_ss58(
                "5HZDvVFWH3ifx1Sx8Uaaa7oiT6U4fAKrR3LKy9r1zFnptc1z",
                "5GJY6A1X8KNvqHcf42Cpr5HZzG95FZVJkTHJvnHSBGgshEWn",
            ), // key 4
            authority_keys_from_ss58(
                "5H3v2VfQmsAAgj63EDaB1ZWmruTHHkJ4kci5wkt6SwMi2VW1",
                "5FXVk1gEsNweTB6AvS5jAWCivXQHTcyCWXs21wHvRU5UTZtb",
            ), // key 5
            authority_keys_from_ss58(
                "5CPhKdvHmMqRmMUrpFnvLc6GUcduVwpNHsPPEhnYQ7QXjPdz",
                "5GAzG6PhVvpeoZVkKupa2uZDrhwsUmk5fCHgwq95cN9s3Dvi",
            ), // key 6
            authority_keys_from_ss58(
                "5DZTjVhqVjHyhXLhommE4jqY9w1hJEKNQWJ8p6QnUWghRYS1",
                "5HmGN73kkcHaKNJrSPAxwiwAiiCkztDZ1AYi4gkpv6jaWaxi",
            ), // key 7
            authority_keys_from_ss58(
                "5ETyBUhi3uVCzsk4gyTmtf41nheH7wALqQQxbUkmRPNqEMGS",
                "5Cq63ca5KM5qScJYmQi7PvFPhJ6Cxr6yw6Xg9dLYoRYg33rN",
            ), // key 8
            authority_keys_from_ss58(
                "5DUSt6KiZWxA3tsiFkv3xYSNuox6PCfhyvqqM9x7N5kuHV2S",
                "5FF1kun4rb5B7C3tqh23XPVDDUJ3UchnaXxJeXu1i5n8KNHp",
            ), // key 9
            authority_keys_from_ss58(
                "5GgsDz9yixsdHxFu52SN37f6TrUtU2RwmGJejbHVmN1ERXL4",
                "5EZiep2gMyV2cz9x54TQDb1cuyFYYcwGRGZ7J19Ua4YSAWCZ",
            ), // key 10
            authority_keys_from_ss58(
                "5HjhkCMa89QJbFULs8WPZBgVg8kMq5qdX1nx7CnQpZgoyKAN",
                "5D5DL9sru2ep3AWoHvmEUbFLirVr7tJ6BxBWH5M8j3r9kUpe",
            ), // key 11
            authority_keys_from_ss58(
                "5F257gHitacwDGvYm2Xm7dBE882auTU8wraG6w4T3r63wh9V",
                "5CovRCaioWENKejfaeccDQY4vCF8kTGtZ5fwagSCeDGmiSyh",
            ), // key 12
            authority_keys_from_ss58(
                "5CtGLbiHWs6XVgNi9nW7oqSP4D4JMot7yHYuFokidZzAP6ny",
                "5DSxsR9aAiq33uSYXWt4zEibx6KT6xxtFGkT9S4GLaCavgDE",
            ), // key 13
            authority_keys_from_ss58(
                "5DeVtxyiniPzoHo4iQiLhGfhED6RP3V73B5nGSYWr5Mgt82c",
                "5HaWL2AvLZHwyPXofWFTEZ6jHVmUG8U9cFATggKZonN1xZjm",
            ), // key 14
            authority_keys_from_ss58(
                "5GF4a6pQ8TQuPhdkKqugzrZSW7YnpQtB4ihouKGZsVMwoTn6",
                "5DaEhFN8bWjvhDxavSWFBr962qoTAMB4b51QebdRZ75VA4h2",
            ), // key 15
            authority_keys_from_ss58(
                "5DAC8Did2NgeVfZeNmEfZuU6t7UseJNf9J68XTvhLf5yCsBZ",
                "5G27pyXx9ieSRCTuDoqPgTvpCynH6yhum9HiQQ1iMj3rAeaP",
            ), // key 16
            authority_keys_from_ss58(
                "5FmxaYznqMqiorPHQgKoRQgEHN7ud4yKsJWr6FvXuS6FS6be",
                "5Ch5XFMKETDiiPiuhUj9TumUtgsnVG1VzQRvBykP9bRdt4km",
            ), // key 17
            authority_keys_from_ss58(
                "5GNAkfKYmFbVRAYm1tPr1yG6bHCapaY7WKRmzkEdendDXj1j",
                "5EC6JjwnE11qaRnjKM85eevQFV1EoaKPPtcBRmTp1XsR7Kx3",
            ), // key 18
            authority_keys_from_ss58(
                "5GYk3B38R9F2TEcWoqCLojqPwx6AA1TsD3EovoTgggyRdzki",
                "5FjdhdAxujZVev6HYqQcTB6UBAKfKFKPoftgMLenoxbNWoe2",
            ), // key 19
            authority_keys_from_ss58(
                "5D7fthS7zBDhwi2u2JYd74t7FpQuseDkUkTuaLZoenXNpXPK",
                "5DhAKQ4MFg39mQAYzndzbznLGqSV4VMUJUyRXe8QPDqD5G1D",
            ), // key 20
        ],
        // Sudo account
        Ss58Codec::from_ss58check("5FCM3DBXWiGcwYYQtT8z4ZD93TqYpYxjaAfgv6aMStV1FTCT").unwrap(),
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
fn finney_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    _root_key: AccountId,
    _endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
    stakes: Vec<(AccountId, Vec<(AccountId, (u64, u16))>)>,
    balances: Vec<(AccountId, u64)>,
    balances_issuance: u64,
) -> serde_json::Value {
    serde_json::json!({
        "balances": { "balances": balances.to_vec() },
        "aura": { "authorities": initial_authorities.iter().map(|x| (x.0.clone())).collect::<Vec<_>>() },
        "grandpa": { "authorities": initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect::<Vec<_>>(),
        },
        "sudo": { "key": Some(<AccountId32 as Ss58Codec>::from_ss58check("5FCM3DBXWiGcwYYQtT8z4ZD93TqYpYxjaAfgv6aMStV1FTCT").unwrap()) },
        "subtensorModule": {
            "stakes": stakes,
            "balancesIssuance": balances_issuance,
        }
    })
}
