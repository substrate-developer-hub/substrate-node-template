//! A dead simple runtime that has a single boolean storage value and three transactions. The transactions
//! available are Set, Clear, and Toggle.

#![cfg_attr(not(feature = "std"), no_std)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use parity_scale_codec::{Decode, Encode};

use sp_std::if_std;
use sp_std::prelude::*;
use sp_api::impl_runtime_apis;
use sp_runtime::{
	ApplyExtrinsicResult,
	create_runtime_str,
	generic,
	transaction_validity::{
		TransactionLongevity,
		TransactionSource,
		TransactionValidity,
		ValidTransaction
	},
	traits::{
		BlakeTwo256,
		Block as BlockT,
		Extrinsic,
	}
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

pub const BOOLEAN_KEY: [u8; 7] = *b"boolean";
pub const HEADER_KEY: [u8; 6] = *b"header";

type Nonce = u8;

/// The Extrinsic type for this runtime. Currently extrinsics are unsigned.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum FramelessTransaction {
	Set(Nonce),
	Clear(Nonce),
	Toggle(Nonce),
}

impl Extrinsic for FramelessTransaction {
	type Call = FramelessTransaction;
	type SignaturePayload = ();

	fn new(call: Self::Call, _signed_data: Option<Self::SignaturePayload>) -> Option<Self> {
		Some(call)
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
			if_std!{
				println!("Entering execute_block with {:?}", block);
			}
			Self::initialize_block(&block.header);

			for transaction in block.extrinsics {
				match Self::apply_extrinsic(transaction) {
					Ok(_) => {},
					Err(e) => {
						if_std!{
							println!("Apply extrinsic error {:?}", e);
						}
					}
				};
			}

			//TODO is this necessary? What method is it even calling?
			// In frame executive, they call final_checks, but that might be different
			Self::finalize_block();
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			if_std!{
				println!("Entering initialize_block with {:?}", header);
			}
			// Store the header info we're given for later use when finalizing block.
			sp_io::storage::set(&HEADER_KEY, &header.encode());
		}
	}

	// https://substrate.dev/rustdocs/master/sc_block_builder/trait.BlockBuilderApi.html
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			if_std!{
				println!("Entering apply_extrinsic");
			}

			let previous_state = sp_io::storage::get(&BOOLEAN_KEY)
				.map(|bytes| <bool as Decode>::decode(&mut &*bytes).unwrap_or(false))
				.unwrap_or(false);

			let next_state = match (previous_state, extrinsic) {
				(_, FramelessTransaction::Set(_)) => true,
				(_, FramelessTransaction::Clear(_)) => false,
				(prev_state, FramelessTransaction::Toggle(_)) => !prev_state,
			};

			sp_io::storage::set(&BOOLEAN_KEY, &next_state.encode());
			Ok(Ok(()))
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			if_std!{
				println!("Entering finalize_block");
			}
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
			Vec::new()
		}

		// This runtime does not expect any inherents, so it does not do any inherent checking.
		fn check_inherents(
			_block: Block,
			_data: sp_inherents::InherentData
		) -> sp_inherents::CheckInherentsResult {
			sp_inherents::CheckInherentsResult::default()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			_source: TransactionSource,
			_tx: <Block as BlockT>::Extrinsic,
			_block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			// Any transaction of the correct type is valid
			Ok(ValidTransaction{
				priority: 1u64,
				requires: Vec::new(),
				// This hach was necessary to make the node accept any transactions at all
				// When I was setting provides to an empty vec, submiting a transaction failed
				// the RPC responded {"code":-32603,"message":"Unknown error occurred","data":"Pool(NoTagsProvided)"}
				// Adding this provides tag solved that. Solutions moving forward:
				// 1. Require a nonce with each transaction
				// 2. Try to relax the TxPool's requirement that every transaction provide some tag
				provides: vec![vec![0]],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
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
			seed.unwrap_or_else(|| vec![0])
		}

		fn decode_session_keys(
			_encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			None
		}
	}

}
