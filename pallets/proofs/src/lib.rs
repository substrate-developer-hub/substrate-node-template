#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub use weights::WeightInfo;

//	Information about the NFT 
#[derive(Encode, Decode, Default, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct ProofInfo<AccountId, Balance, BlockNumber, BoundedString> { 
	pub owner: AccountId,
	// 	The original minter
	pub issuer: AccountId, 
	//	NFT issuer 
	pub issued: BlockNumber,
	//	IPFS image link 
	pub proof: BoundedString, 
	//	The user friendly name of this NFT 
	pub name: BoundedString,
	//	Proof Description 
	pub name: BoundedString,
}
pub type CID = Vec<u8>;
pub const MAX_IPFS_CID_LEN = 200 as usize;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::{Randomness, }};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::Printable, print
	};
	use sp_std::vec::Vec;
	use sp_io::hashing::blake2_128;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config 
		+ orml_nft::Config<TokenData = TokenData<BalanceOf<Self>>, ClassData = ClassData<BalanceOf<Self>>{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//	This will represent the currency denominated during the crowdfund 
		type Currency: ReservableCurrency<Self::AccountId>;
		//	Benchmarking purposes 
		type WeightInfo: WeightInfo;
		
		// TODO! Configure in the runtime 
		//	The maximum lenght of a name or symbol stored on chain 
		#[pallet::constant]
		type StringLimit: Get<u32> 
	}
	//	Identifying a Proof Id 
	pub type ProofIndexOf<T> = <T as orml_nft::Config>::TokenId;
	//	Class id 
	pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
	//	Balance Of AccountId Based on Currency Pallet 	
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	//	Type to call AccountId
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	//	Information about Proof
	pub type ProofInfoOf<T> = ProofInfo<AccountIdOf<T>, BalanceOf<T>, BlockNumber<T>>;
	//	Info Of Proof
	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ProofIndex, 
		ProofInfoOf<T>,
		OptionQuery
	>; 
	#[pallet::storage]
	#[pallet::getter(fn meta_data)]
	pub(super) type MetaDataStore<T: Config> = StorageValue<_, Proof

	//	The class id for orml nft
	#[pallet::storage]
	#[pallet::getter(fn class_id)]
	pub type ClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

	//	TODO!
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(
		AccountIdOf<T> = "AccountId", 
		BalanceOf<T> = "Balance", 
		BlockNumber<T> = "BlockNumber",
		Option<BalanceOf<T>> = "Option<Balance>")]
		
	pub enum Event<T: Config> {
		Minted(AccountIdOf<T>, ProofIndexOf<T>, BlockNumber<T>),
		//	Proof Minted by the owner
		Burned(AccountIdOf<T>, ProofIndexOf<T>, BlockNumber<T>),
		//	Proof Previously Minted by the owner is burned
		TransferredOwnership(AccountIdOf<T>, AccountIdOf<T>, ProofIndexOf<T>),
		//	Proof is transferred to the new Owner
		PriceSet(AccountIdOf<T>, Option<BalanceOf<T>>),
		//	Owner sets a price of Proof 
		//	A sender bought the Proof ID
		ProofSold(AccountIdOf<T>, AccountIdOf<T>, ProofIndexOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidIndex, 
		//	The proof index specified does not exist 
		NotProofOwner,
		//	The sender is not the owner so the proof can't be transferred
		PriceTooLow, 
		//	There's not enough balance in the sender's account for the transfer
		BuyingFromSelf,
		//	The sender is also the buyer
		NotForSale,
		//	The sender cannot buy Proof that's not listed on the market 
		CannotBurnEmpty,
		//	The sender cannot burn Proof if there's no specified Proof dictated
		InsufficientBalance
		//	The sender does not have enough balance to take order
		NoProofImported
		//	The sender has not uploaded a proof of ownership via offchain worker 
		MAX_IPFS_CID_LEN,
	}
	impl<T: Config> Printable for Error<T> { 
		fn print(&self) { 
			match self { 
				Error::InvalidIndex => "Proof Id does not exist".print(),
				Error::NotProofOwner => "Sender of this transaction is not the owner".print(),
				Error::PriceTooLow => "The price cannot be lower than Zero".print(),
				Error::BuyingFromSelf => "You are buying from yourself".print(),
				Error::NotForSale => "Proof is not for sale".print(),
				Error::CannotBurnEmpty => "No Proof is found".print(),
				Error::InsufficientBalance => "Insufficient Balance".print(),
				_ => "Invalid Error Case".print(),
			}
		}
	}
	//	TODO! Implement an offchain worker for NFT Proofs [HTTP request]
	
	//	TODO! Implement an offchain storage

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::weight(1_000)]
		pub fn create_class(
			origin: OriginFor<T>,
			metadata: CID,
		) -> DispatchResultWithPostInfo { 

			let sender = ensure_sender(origin)?;
			ensure!(metadata.len() <  MAX_IPFS_CID_LEN, Error::<T>::MAX_IPFS_CID_LEN);
			let next_id = orml_nft::Pallet::<T>::next_class_id();
			let class_id = orml_nft::<T>::create_



		}



		#[pallet::weight(10_000)]
 		pub fn mint_proof(
			origin: OriginFor<T>,
			metadata: CID
		) -> DispatchResultWithPostInfo {
			let sender = ensure_sign(origin)?;
			let beneficiary = T::Lookup::lookup(beneficiary)?;
			let class_id = Self::class_id();

			let token_info = 
			

			let proof_nft = orml_nft::Pallet::<T>::mint(&sender, class_id, proof, )


			Ok(().into())
		}
		pub fn set_proof_info() {}
		pub fn burn_proof() {}
		pub fn transfer()  {}
		pub fn buy_proof() {}
		pub fn set_price() {}

	}

	//	Internal functions of the pallet 
	impl<T: Config> Pallet<T> {
		//	This function will give random values to represent as Proof ID
		//	ProofID can be used to access ProofInfoOf<T> -> ProofInfo 
		fn get_random_num(sender: &T::AccountId) -> [u8; 16] { 
			let payload = (
				T::Randomness::random(&b"id"[..]).0,
				&sender,
				<frame_system::Pallet<T>>::block_number());
			payload.using_encoded(blake2_128);
		}
	}
}




























