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

pub type CID = Vec<u8>;

//	Type of NFT
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProofInfo<Balance, AccountId, BoundedString> { 
	pub owner: AccountId,
	pub issuer: AccountId,
	pub freezer: AccountId,
	pub supply: u32,
	pub is_frozen: bool,
	pub is_transferable: bool,
	pub is_burnable: bool,
	//	Metadata
	//	pub metadata: ProofInfo<BoundedString>
}
//	MetaData of NFT 
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProofMetaData<BoundedString> { 
	pub name: BoundedString, 
	pub nick_name: BoundedString,
	pub collection: BoundedString,
	pub description: BoundedString,
	pub external_link: CID, 
}

//	Set default values if no ProofInfo is found 

pub type ProofIdOf<T> = <T as orml_nft::Config>::TokenId;
//	Class id 
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
//	Balance Of AccountId Based on Currency Pallet 	
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
//	Type to call AccountId
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub type ProofInfoOf<T> = ProofInfo<T::Balance, T::AccountId>;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config 
		+ orml_nft::Config<TokenData = TokenData<BalanceOf<Self>>, ClassData = ClassData<BalanceOf<Self>>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//	This will represent the currency denominated during the crowdfund 
		type Currency: ReservableCurrency<Self::AccountId>;
		//	Benchmarking purposes 
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type StringLimit: Get<u32>;

		#[pallet::constant]
		type Depositbalance: Get<u32>;

		#[pallet::constant]
		type MaxSupplyAllowed: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//	Proof info Class Storage of Proof 
	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type ProofInfoStorage<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		ClassIdOf<T>,
		ProofInfoOf<T>,
		OptionQuery
	>;
	//	Proof MetaData Storage 
	// #[pallet::storage]
	// pub type MetaData<T: Config> = StorageMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	ClassIdOf<T>,
	// 	ProofMetaData<BoundedVec<u8, T::StringLimit>>,
	// 	ValueQuery,
	// >;


	//	token id and class id => metadata of nft
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub type ProofMetaDataStorage<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		ClassIdOf<T>,
		ProofMetaData<T::StringLimit, T::Depositbalance>,
		ValueQuery,
	>;

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}


	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClassDefined(T::AccountId, ProofIdOf<T>),

		Minted(T::AccountId, T::AccountId, ClassIdOf<T>, u32),

		NewMetaData(T::AccoundId, ClassIdOf<T>),
		
		Transferred(T::AccountId, T::AccountId, ProofIdOf<T>, ClassIdOf<T>, u32),

		Burned(T::AccountId, ClassIdOf<T>, ProofIdOf<T>),

		Frozen(T::AccountId, ClassIdOf<T>, ProofIdOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		BadMetaData,
		MaxSupplyExceeded,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//	Issue a new type of NFT with varying attributes 
		//	To differentiate from other classes, we generate a class_id 

		//	NOTE: This will only set the type of NFT to be minted, no NFT will be minted until executing mint() extrinsic
		
		//	Parameter: 
		//	'tokenid' - ProofIdOf<T>
		//	'classid' - ClassIdOf<T>
			
		//	Create a new class, take Deposit
		#[pallet::weight(T::WeightInfo::class_create())]
		pub fn class_create(
			origin: OriginFor<T>,
			beneficiary: <T as StaticLookup>::Source,
			is_frozen: bool,
			is_burnable: bool,
			is_transferable: bool,
			supply: u32,
			freezer: <T as StaticLookup>::Source,
			issuer: <T as StaticLookup>::Source,

		) -> DispatchResultWithPostInfo {
			
			let sender = ensure_origin(origin)?;
			let owner = T::Lookup::lookup(beneficiary)?;
			let freeze_account_id = T::Lookup::lookup(freezer)?;
			let issue_account_id = T::Lookup::lookup(issuer)?; 
			let metadata = CID;
			
			//	Create a new class
			ensure!(supply < MaxSupplyAllowed, Error::<T>::MaxSupplyExceeded)?; 
			let class_id = orml_nft::Pallet::<T>::next_class_id();			
			let info = ProofInfo { 
				owner,
				issuer, 
				freezer: freeze_account.clone(),
				supply,
				is_frozen,
				is_transferable,
				is_burnable,
			};
			orml_nft::Pallet::<T>::create_class(&sender, CID, info)?;
			<ProofInfoStorage<T>>::insert(class_id, info);

			Self::deposit_event(Event::ClassDefined(sender, metadata, class_id));

			Ok(().into())
		}
		//	Setting Custom Metadata stored on NFTs
		//	NFT metadata is temporarily stored on chain 
		//	'external_link' is to be provided by the user which is generated using IPFS
		//	'class_id' is created by the user
	
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_metadata(
			origin: OriginFor<T>, 
			name: Vec<u8>, 
			nick_name: Vec<u8>,
			collection: Vec<u8>, 
			external_link: Vec<u8>,
			class_id: ClassIdOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(<ProofInfoStorage<T>>::contains_key(&class_id), Error::<T>::NoneValue); 

			let bounded_name: BoundedVec<u8, T::StringLimit> = 
				name.clone().try_into().map_err(|_| Error::<T>::BadMetaData)?;
			let bounded_nick: BoundedVec<u8, T::StringLimit> = 
				name.clone().try_into().map_err(|_| Error::<T>::BadMetaData)?;
				
			//	ensure we are setting metadatas based on their respective classes 
			//	Store Proof Metadata using Sender and Class Id created from the previous function 
			ProofMetaDataStorage::<T>::insert(&sender, class_id, |proof_metadata| { 
				*proof_metadata = ProofMetaData { 
					name: bounded_name,
					nick_name: bounded_nick,
					collection: bounded_name, 
					external_link: bounded_name,
				};
			});
			Self::deposit_event(Event::NewMetaData(sender.clone(), class_id));
			Ok(().into())
		}
		
		//	Issue specified NFT class tokens here 

		//	Paremeters:
		//	'class_id' the type of NFT to be minted => ProofInfo Struct 
		//	'quantity' of tokens to be supplied to the beneficiary
		//	'beneficiary' is the amount to be credited with the minted assets 
		//	'metadata' get from fn set_metadata()		
		//	Emit 'Minted' event when successfull 
		pub fn mint(
			origin: OriginFor<T>,
			beneficiary: <T as StaticLookup>::Source,
			class_id: ClassIdOf<T>,
		) -> DispatchResultWithInfo {
			let sender = ensure_origin(origin);
			let owner = T::Lookup::lookup(beneficiary)?;
			
			ensure!(
				<ProofInfoStorage<T>>::contains_key(&sender, &class_id),
				Error::<T>::NoneValue
			);
			//	Class Id value
			let proof_specification = <ProofInfoStorage<T>>::try_get(class_id);
			//	Metadata for Class id value 
			let proof_metadata = <ProofMetaDataStorage<T>>::try_get(&sender, &class_id);
			//	Mint NFTs according to the specified supply in class id 
			for quantity in 0..proof_specification.supply { 
				orml_nft::Pallet::<T>::mint(
					sender.clone(),
					class_id.clone(),
					proof_metadata,
					proof_specification);
			}
			//	transfer tokens to benfiticary  
			//	Proof Info MetaData Storage
			
			//	Generate the token id 

			//	transfer to the original sender with no fees 


			Self::deposit_event(Event::Minted(sender, class_id));

			Ok(().into())
		}
		
		
		
		
		pub fn clear_metadata() -> DispatchResultWithPostInfo {}
		pub fn transfer() -> DispatchResultWithPostInfo {}


	}
}
