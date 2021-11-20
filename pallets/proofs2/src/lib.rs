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


//	This Struct is for setting custom classes for NFT
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProofInfo<AccountId, BoundedString> { 
	pub class_name: BoundedString,
	pub class_creator: AccountId,
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

	//	ProofIDStorage
	#[pallet::storage]
	#[pallet::getter(fn proof_id)]
	pub type ProofId<T: Config> = StorageMap<
		_,
		Blake2_128Concant, 
		T::AccountId, 
		ProofIdOf<T>,
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
	pub type MetaData<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		ClassIdOf<T>, // Class Id of NFT
		Blake2_128Concat, 
		T::AccountId,	// User Account 
		ProofMetaData<BoundedVec<u8, T::StringLimit>>,
		ValueQuery,
	>;

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> { 
		//	Genesis Classes [Accounts, BoundedVec]
		pub class_type: Vec<(T::AccountId, Vec<u8>)>,
		//	Genesis Metadata [Accounts, token id, BoundedVec, classid ]
		pub metadata: Vec<(T::AccountId, ProofIdOf<T>, Vec<u8>, ClassIdOf<T>)>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> { 
		fn default() -> Self { 
			Self {
				class_type: Default::default(),
				metadata: Default::default(),
			}
		}
	}
	//	Todo
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> { 
		fn build(&self) {}
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		//	User created a unique class id 
		ClassDefined { T::AccountId, ProofIdOf<T> },
		//	Minted using class_id and metadata specified by the user 
		Minted {
			T::AccountId, 
			T::AccountId, 
			ClassIdOf<T>, 
			u32
		},
		//	User Created a Metadata
		NewMetaData	{ T::AccoundId, ClassIdOf<T>},
		Transferred {
			from: T::AccountId, 
			to: T::AccountId, 
			id: ProofIdOf<T>, 
			class: ClassIdOf<T>, 
		},
		Burned {
			executed_by: T::AccountId,
			class_id : ClassIdOf<T>, 
			id: ProofIdOf<T>
		},
		Frozen { T::AccountId, ClassIdOf<T>, ProofIdOf<T>},
		//	Metadata has been clearead for an asset class 
		ClassMetadataCleared {
			class: ClassIdOf<T>,
		}

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		BadMetaData,
		//	Bad Metadata added
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
		}
		//	Issue specified NFT class tokens here 

		//	Parameters:
		//	'class_id' the type of NFT to be minted => ProofInfo Struct 
		//	'quantity' of tokens to be supplied to the beneficiary
		//	'metadata' get from fn set_metadata()		
		//	Emit 'Minted' event when successfull 
		#[pallet::weight(10_000)]
		pub fn mint(
			origin: OriginFor<T>,
		) -> DispatchResultWithInfo {
			let sender = ensure_origin(origin);
			let class_id = ClassId::<T>::take(sender);
			//	Get Storage items
			let metadata = Metadata::<T>::try_get(&class_id, &sender);
			
			for _ in metadata.edition_total { 
				let token_id = orml_nft::Pallet::<T>::mint(&sender, class_id, _ , metadata.clone());
				ProofId::<T>::insert(sender, token_id);
			}
			Self::deposit_event(Event::Minted(sender, class_id));
			Ok(().into())
		}
		//	Input
		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] id: ProofIndex<T> 
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;
	
			let class_id = ClassIdOf::<T>::get(sender);
			let token_info = ProofId::<T>::get(owner);
			let token = (class_id, token_info);

			orml_nft::Pallet::<T>::transfer(&sender, &to, token)?;

			Self::deposit_event(Event::Transferred(sender, to, id, class_id));

			Ok(().into())
		}
		//	Clear Metadata for users without destroying class_ids from onchain storage 
		//	Parameters:
		//	- Get 'class_id' associated under account on chain storage 
		//	- Use 'class_id' to remove the metadata on the storage NOT on the NFT itself 

		#[pallet::weight(10_000)]
		pub fn clear_metadata(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			let class_id = ClassIdOf::<T>::get(owner);
			Metadata::<T>::remove_prefix(&class_id);

			Self::deposit_event(Event::ClassMetadataCleared(class_id));
			Ok(().into())	
		}
		//	Burn without destroying class_id stored within onchain data storage 
		//	This one probably doesn't work properly 
		#[pallet::weight(10_000)]
		pub fn burn(
			origin: OriginFor<T>,
			
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			//	let class_id = ClassId::<T>::get(owner);
			let class_info = ClassId::<T>::get(owner);
			let token_info = ProofId::<T>::get(owner);
			
			let token = (class_info, token_info);
			//	Burn tokens
			orml_nft::Pallet::<T>::burn(&owner, token)?;
			
			Self::deposit_event(Event::Burned());

			Ok(().into())
		} 
	}
}




