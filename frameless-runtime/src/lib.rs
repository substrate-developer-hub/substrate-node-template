//! A dead simple runtime that has a single boolean storage value and three transactions. The transactions
//! available are Set, Clear, and Toggle.

#![cfg_attr(not(feature = "std"), no_std)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use parity_scale_codec::{Decode, Encode};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;

use log::info;

use sp_std::prelude::*;
use sp_api::impl_runtime_apis;
use sp_runtime::{
	ApplyExtrinsicResult,
	BoundToRuntimeAppPublic,
	create_runtime_str,
	generic,
	transaction_validity::{
		TransactionLongevity,
		TransactionSource,
		TransactionValidity,
		TransactionValidityError,
		InvalidTransaction,
		ValidTransaction,
	},
	traits::{
		BlakeTwo256,
		Block as BlockT,
		Extrinsic,
	},
	// Importing impl_opaque_keys requires scale info
	impl_opaque_keys,
};
use sp_core::{
	H256,
	H512,
	sr25519::{Public, Signature}
};
// This strange-looking import is usually done by the `construct_runtime!` macro
use sp_block_builder::runtime_decl_for_BlockBuilder::BlockBuilder;
#[cfg(feature = "std")]
use sp_storage::well_known_keys;

#[cfg(any(feature = "std", test))]
use sp_runtime::{BuildStorage, Storage};

use sp_core::OpaqueMetadata;

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// An index to a block.
pub type BlockNumber = u32;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
	use super::*;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, FramelessTransaction>;

	// This part is necessary for generating session keys in the runtime
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: AuraAppPublic,
			pub grandpa: GrandpaAppPublic,
		}
	}

	// Typically these are not implemented manually, but rather for the pallet associated with the keys.
	// Here we are not using the pallets, and these implementations are trivial, so we just re-write them.
	pub struct AuraAppPublic;
	impl BoundToRuntimeAppPublic for AuraAppPublic {
		type Public = AuraId;
	}

	pub struct GrandpaAppPublic;
	impl BoundToRuntimeAppPublic for GrandpaAppPublic {
		type Public = sp_finality_grandpa::AuthorityId;
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("frameless-runtime"),
	impl_name: create_runtime_str!("frameless-runtime"),
	authoring_version: 1,
	spec_version: 1,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// The type that provides the genesis storage values for a new chain
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Default))]
pub struct GenesisConfig;

#[cfg(feature = "std")]
impl BuildStorage for GenesisConfig {
	fn assimilate_storage(&self, storage: &mut Storage) -> Result<(), String> {
		// Declare the storage items we need
		let storage_items = vec![
			(BOOLEAN_KEY.encode(), false.encode()),
			(well_known_keys::CODE.into(), WASM_BINARY.unwrap().to_vec()),
		];

		// Put them into genesis storage
		storage.top.extend(
			storage_items.into_iter()
		);

		Ok(())
	}
}

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, FramelessTransaction>;

// 626F6F6C65616E raw storage key
pub const BOOLEAN_KEY: [u8; 7] = *b"boolean";

// 686561646572 raw storage key
pub const HEADER_KEY: [u8; 6] = *b"header";

type Salt = u8;

/// The Extrinsic type for this runtime. Currently extrinsics are unsigned.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum FramelessCall {
	Set,
	Clear,
	Toggle,
}

// So we can have a cli utility to create transactions. Not necessary for the runtime itself.
#[cfg(feature = "std")]
impl core::str::FromStr for FramelessCall {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, String> {
		match s.to_lowercase().as_str() {
			"set" => Ok(FramelessCall::Set),
			"clear" => Ok(FramelessCall::Clear),
			"toggle" => Ok(FramelessCall::Toggle),
			_ => Err(String::from("Failed to parse frameless transaction.")),
		}
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct FramelessTransaction {
	call: FramelessCall,
	salt: Salt,
	signature: H512,
	signer: H256,
}

impl Extrinsic for FramelessTransaction {
	type Call = FramelessCall;
	type SignaturePayload = (Salt, H512, H256);

	fn new(call: Self::Call, salt_sig_signer: Option<Self::SignaturePayload>) -> Option<Self> {
		match salt_sig_signer {
			Some((salt, signature, signer)) => Some(Self { call, salt, signature, signer }),
			None => None,
		}
	}
}

/// Checks a transaction that came in from the outside world.
/// Returns the same type needed by the transaction pool.
/// This is seperated into a function so it can be re-used
/// when executing transactions.
fn check_framesless_transaction(tx: &FramelessTransaction) -> TransactionValidity {
	info!(target: "frameless", "🖼️ Checking Frameless Transaction");

	// Any transaction with a verifiable signature is valid
	let signed_payload = (&tx.call, &tx.salt).encode();
	let valid_sig = sp_io::crypto::sr25519_verify(
		&Signature::from_raw(*tx.signature.as_fixed_bytes()),
		&signed_payload,
		&Public::from_h256(tx.signer)
	);

	if valid_sig {
		info!(target: "frameless", "🖼️ Valid signature by: {:?}", tx.signer);
		Ok(ValidTransaction{
			priority: 1u64,
			requires: Vec::new(),
			// Every transaction must provide _some_ tag to de-duplicate it in the pool
			provides: vec![tx.encode()],
			longevity: TransactionLongevity::max_value(),
			propagate: true,
		})
	}
	else {
		Err(TransactionValidityError::Invalid(InvalidTransaction::BadProof))
	}
}

/// The main struct in this module. In frame this comes from `construct_runtime!`
pub struct Runtime;

impl_runtime_apis! {
	// https://substrate.dev/rustdocs/master/sp_api/trait.Core.html
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			info!(target: "frameless", "🖼️ Entering execute_block. block: {:?}", block);

			Self::initialize_block(&block.header);

			for transaction in block.extrinsics {
				match Self::apply_extrinsic(transaction) {
					Ok(_) => {},
					Err(e) => {
						info!(target: "frameless", "Apply extrinsic error {:?}", e);
					}
				};
			}

			// In frame executive, they call final_checks, but that might be different
			Self::finalize_block();
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			info!(target: "frameless", "🖼️ Entering initialize_block. header: {:?}", header);
			// Store the header info we're given for later use when finalizing block.
			sp_io::storage::set(&HEADER_KEY, &header.encode());
		}
	}

	// https://substrate.dev/rustdocs/master/sc_block_builder/trait.BlockBuilderApi.html
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			info!(target: "frameless", "🖼️ Entering apply_extrinsic: {:?}", extrinsic);

			//TODO log the signer somewhere
			// First check the transaction
			check_framesless_transaction(&extrinsic)?;

			// Now we can actually apply the changes
			let previous_state = sp_io::storage::get(&BOOLEAN_KEY)
				.map(|bytes| <bool as Decode>::decode(&mut &*bytes).unwrap_or(false))
				.unwrap_or(false);

			info!(target: "frameless", "🖼️ Previous stored state was: {:?}", previous_state);

			let next_state = match (previous_state, extrinsic.call) {
				(_, FramelessCall::Set) => true,
				(_, FramelessCall::Clear) => false,
				(prev_state, FramelessCall::Toggle) => !prev_state,
			};

			info!(target: "frameless", "🖼️ Newly stored state is: {:?}", next_state);

			sp_io::storage::set(&BOOLEAN_KEY, &next_state.encode());
			Ok(Ok(()))
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			info!(target: "frameless", "🖼️ Entering finalize block.");
			// https://substrate.dev/rustdocs/master/sp_runtime/generic/struct.Header.html
			let raw_header = sp_io::storage::get(&HEADER_KEY)
				.expect("We initialized with header, it never got mutated, qed");

			// Clear the raw header out of storage when we are done with it.
			sp_io::storage::clear(&HEADER_KEY);

			let mut header = <Block as BlockT>::Header::decode(&mut &*raw_header)
				.expect("we put a valid header in in the first place, qed");

			let raw_state_root = &sp_io::storage::root(sp_storage::StateVersion::default())[..];

			header.state_root = sp_core::H256::decode(&mut &raw_state_root[..]).unwrap();
			header
		}

		// This runtime does not expect any inherents so it does not insert any into blocks it builds.
		fn inherent_extrinsics(_data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			info!(target: "frameless", "🖼️ Entering inherent_extrinsics.");
			Vec::new()
		}

		// This runtime does not expect any inherents, so it does not do any inherent checking.
		fn check_inherents(
			block: Block,
			_data: sp_inherents::InherentData
		) -> sp_inherents::CheckInherentsResult {
			info!(target: "frameless", "🖼️ Entering check_inherents. block: {:?}", block);
			sp_inherents::CheckInherentsResult::default()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			info!(target: "frameless", "🖼️ Entering validate_transaction. source: {:?}, tx: {:?}, block hash: {:?}", source, tx, block_hash);

			check_framesless_transaction(&tx)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(vec![0])
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(_header: &<Block as BlockT>::Header) {
			// we do not do anything.
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			info!(target: "frameless", "🖼️ Entering generate_session_keys. seed: {:?}", seed);
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	// Here is the Aura API for the sake of making this runtime work with the node template node
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			// Three-second blocks
			sp_consensus_aura::SlotDuration::from_millis(3000)
		}

		fn authorities() -> Vec<AuraId> {
			// The only authority is Alice. This makes things work nicely in `--dev` mode
			use sp_application_crypto::ByteArray;

			vec![
				AuraId::from_slice(&hex_literal::hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d").to_vec()).unwrap()
			]
		}
	}

	impl sp_finality_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_finality_grandpa::AuthorityList {
			use sp_application_crypto::ByteArray;
			vec![
				(
					sp_finality_grandpa::AuthorityId::from_slice(&hex_literal::hex!("88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee").to_vec()).unwrap(),
					1
				)
			]
		}

		fn current_set_id() -> sp_finality_grandpa::SetId {
			0u64
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_finality_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				sp_runtime::traits::NumberFor<Block>,
			>,
			_key_owner_proof: sp_finality_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_finality_grandpa::SetId,
			_authority_id: sp_finality_grandpa::AuthorityId,
		) -> Option<sp_finality_grandpa::OpaqueKeyOwnershipProof> {
			None
		}
	}

}
