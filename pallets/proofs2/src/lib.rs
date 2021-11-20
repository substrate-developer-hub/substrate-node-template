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


//	This Struct is for setting custom classes for NFT
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProofInfo<AccountId, BoundedString> { 
	pub class_name: BoundedString,
	pub class_creator: AccountId,
	pub metadata: ProofInfoMetaData<BoundedString>
}
//	This struct is for storing the metadata of NFT
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProofMetaData<AccountId, ProofIdOf<T>, BoundedString, ClassidOf<T>> { 
	pub symbol: BoundedString,
	//	Name to represent NFT on marketplaces 
	pub image: BoundedString,
	//	The URL linking to the NFT artwork's image file
	pub name: BoundedString,
	//	Name of the art work 
	pub description: BoundedString,
	//	Description for NFT
	pub animation_url: BoundedString,
	//	URL linking to the animation 
	pub copyright_transfer: bool,
	//	Whether the copyright is transferred to the buyer
	pub tokenid: ProofIdOf<T>,
	//	Token address on the chain 
	pub resellable: bool,
	//	Whether the artwork can be sold 
	pub original_creator: AccountId,
	//	NFT creator's address on chain 
	pub edition_number: u8,
	//	Edition number of the artwork 
	pub class_category: ClassIdOf<T>,
	//	Class id of the Artwork 
	pub edition_total: u32
	//	Total number of editions of the artwork 
}

//	Set default values if no ProofInfo is found 

pub type ProofIdOf<T> = <T as orml_nft::Config>::TokenId;
//	Class id 
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;
//	Balance Of AccountId Based on Currency Pallet 	
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
//	Type to call AccountId
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;



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

	//	Class_id 
	//	Keys: AccountId, Value: Class id
	//	Return None if there no Class Id made by user 
	#[pallet::storage]
	#[pallet::getter(fn class_id)]
	pub type ClassId<T: Config> = StorageMap<
		_,
		Blake2_128Concant, 
		T::AccountId, 
		ClassIdOf<T>,
		OptionQuery,
	>;

	//	Proof info Class Storage: keys: Class_id => val: Class_info struct 
	#[pallet::storage]
	#[pallet::getter(fn info)]
	pub type ProofInfoStorage<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		ClassIdOf<T>,
		ProofInfo<T::AccountId, BoundedVec<u8, T::StringLimit>>,
		ValueQuery,
	>;
	//	Proof MetaData Storage: keys: [class_id], [accountId] => val: metadata
	#[pallet::storage]
	#[pallet::getter(fn metadata)]
	pub type MetaData<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ClassIdOf<T>,
		Blake2_128Concat, 
		T::AccountId,
		ProofMetaData<BoundedVec<u8, T::StringLimit>>,
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
			class_name: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_origin(origin)?;
			
			let bounded_name: BoundedVec<u8, T::StringLimit> = 
				name.clone().try_into().map_err(|_| Error::<T>::BadMetaData)?;

			let info = ProofInfo { 
				class_name: bounded_name,
				class_creator: sender.clone(),
				_,
				//	No Metadata Set into Place 
			};
			let class_id = orml_nft::Pallet::<T>::create_class(&sender, _, info);
			//	Insert ClassId under AccountId 
			ClassId::<T>::insert(&sender, class_id);
			//	Storage Associated CLassid with Class Information 
			ProofInfoStorage::<T>::insert(&class_id, info);
			Self::deposit_event(Event::ClassDefined(sender, _, class_id));

			Ok(().into())
		}
		//	Setting Custom Metadata stored on NFTs
		//	NFT metadata is temporarily stored on chain 
		//	'external_link' is to be provided by the user which is generated using IPFS
		//	'class_id' is created by the user
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_metadata(
			origin: OriginFor<T>, 
			symbol: Vec<u8>,
			image: Vec<u8>,
			name: Vec<u8>,
			description: Vec<u8>
			animation_url: Vec<u8>,
			copyright_transfer: bool,
			resellable: bool,
			original_creator: <T::Lookup as StaticLookup>::Source,
			edition_total: u32,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			//	Get class_id assciated to the user 
			let class_id = ClassId::<T>::take(sender);
			let creator = T::Lookup::lookup(original_creator);
			let bounded_name: BoundedVec<u8, T::StringLimit> = 
				name.clone().try_into().map_err(|_| Error::<T>::BadMetaData)?;
			
			let bounded_symbol: BoundedVec<u8, T::StringLimit> = 
				name.clone().try_into().map_err(|_| Error::<T>::BadMetaData)?;
			
			let bounded_description: BoundedVec<u8, T::StringLimit> = 
				name.clone().try_into().map_err(|_| Error::<T>::BadMetaData)?;

			let metadata = ProofInfo { 
				symbol: bounded_symbol,
				image,
				name: bounded_name,
				description: bounded_description,
				animation_url,
				copyright_transfer,
				token_id: (),
				resellable,
				original_creator: creator,
				edition_number: class_id,
				class_category: class_id,
				edition_total,
			};
			//	Insert into onchain storage
			Metadata::<T>::insert(class_id, sender, metadata);
			
			Self::deposit_event(Event::NewMetaData(sender.clone(), class_id));
			Ok(().into())

		//	Issue specified NFT class tokens here 

		//	Paremeters:
		//	'class_id' the type of NFT to be minted => ProofInfo Struct 
		//	'quantity' of tokens to be supplied to the beneficiary
		//	'beneficiary' is the amount to be credited with the minted assets 
		//	'metadata' get from fn set_metadata()		
		//	Emit 'Minted' event when successfull 
		pub fn mint(
			origin: OriginFor<T>,
		) -> DispatchResultWithInfo {
			let sender = ensure_origin(origin);
			let class_id = ClassId::<T>::take(sender);
			//	Get Storage items
			let metadata = Metadata::<T>::get(&class_id, &sender);
			//	ORML Mint  
			let token_id = orml_nft::<T>::mint(
				&sender,
				class_id,
				_,
				metadata
			);	

			Self::deposit_event(Event::Minted(sender, class_id));
			Ok(().into())
		}
		
		
		
		
		pub fn clear_metadata() -> DispatchResultWithPostInfo {}
		pub fn transfer() -> DispatchResultWithPostInfo {}


	}
}
