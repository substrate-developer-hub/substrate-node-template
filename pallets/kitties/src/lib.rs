#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

mod mock;
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::{
		sp_runtime::traits::Hash,
		traits::{ Randomness, Currency, tokens::ExistenceRequirement },
	};
	use sp_io::hashing::blake2_128;

	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq)]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],   // Using 16 bytes to represent a kitty DNA
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}

	// Set Gender type in Kitty struct.
	#[derive(Encode, Decode, Debug, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		/// The type of Random we want to specify for runtime.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		KittyCntOverflow,
		ExceedMaxKittyOwned,
		BuyerIsKittyOwner,
		KittyNotExist,
		NotKittyOwner,
		KittyNotOnSale,
		KittyBidPriceTooLow,
		NotEnoughBalance,
		KittyNotFound,
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created(T::AccountId, T::Hash),
		PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
		Transferred(T::AccountId, T::AccountId, T::Hash),
		Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
	}

	// Storage items.

	// Keeps track of the KittyCount
	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	// Stores a Kitty: it's unique traits and price.
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, T::Hash, Kitty<T>>;

	// Keeps track of what accounts own what Kitty.
	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub(super) type KittyOwned<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::Hash, T::MaxKittyOwned>, ValueQuery>;

	// Our pallet's genesis configuration.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub genesis: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	// Required to implement default for GenesisConfig.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { genesis: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// When building a kitty from genesis config, we require the dna and gender to be supplied
			//   because we cannot call RandomnessCollectiveFlip at block number = 0
			for (acct, dna, gender) in &self.genesis {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1)]
		pub fn hello_world(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			log::info!("Hello World! Transaction received from: {:?}", sender);
			Ok(())
		}

		/// Create a new unique kitty.
		///
		/// Provides the new Kitty details to the 'mint()'
		/// helper function (sender, kitty hash, Kitty struct).
		///
		/// Calls mint() and increment_nonce().
		///
		/// Weight: `O(1)`
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let kitty_id = Self::mint(&sender, None, None)?;

			// Deposit our "Created" event.
			Self::deposit_event(Event::Created(sender, kitty_id));
			Ok(())
		}

		/// Set the price for a Kitty.
		///
		/// Updates Kitty price and updates storage.
		///
		/// Weight: `O(1)`
		#[pallet::weight(100)]
		pub fn set_price(origin: OriginFor<T>, kitty_id: T::Hash, new_price: Option<BalanceOf<T>>)
			-> DispatchResult
		{
			let sender = ensure_signed(origin)?;

			// Make sure the owner matches the corresponding owner.
			// Also check if kitty exists.
			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, <Error<T>>::NotKittyOwner);

			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

			kitty.price = new_price.clone();
			<Kitties<T>>::insert(&kitty_id, kitty);

			// Deposit a "PriceSet" event.
			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));

			Ok(())
		}

		/// Transfer a Kitty.
		///
		/// Any account that holds a Kitty can send it to another Account.
		///
		/// Weight: `O(1)`
		#[pallet::weight(100)]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, kitty_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure the Kitty exists.
			// Verify Kitty owner: must be the account invoking this transaction.
			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, <Error<T>>::NotKittyOwner);

			// Verify the `to` has the capacity to receive one more kitty
			let to_owned = <KittyOwned<T>>::get(&to);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);

			Self::transfer_kitty_to(&kitty_id, &to, false)?;

			Self::deposit_event(Event::Transferred(sender, to, kitty_id));

			Ok(())
		}

		/// Buy a Kitty by asking a price. Ask price must be more than
		/// current price.
		///
		/// Check that the Kitty exists and is for sale. Update
		/// the price in storage and Balance of owner and sender.
		///
		/// Weight: `O(1)`
		#[pallet::weight(100)]
		pub fn buy_kitty(origin: OriginFor<T>, kitty_id: T::Hash, bid_price: BalanceOf<T>)
			-> DispatchResult
		{
			let sender = ensure_signed(origin)?;

			// Check: Buyer is not current kitty owner
			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

			ensure!(kitty.owner != sender, <Error<T>>::BuyerIsKittyOwner);

			// Check: Kitty is on sale and the kitty ask price <= bid_price
			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				Err(<Error<T>>::KittyNotOnSale)?;
			}

			// Check: buyer has enough free balance
			ensure!(T::Currency::free_balance(&sender) >= bid_price, <Error<T>>::NotEnoughBalance);

			// Check: Verify buyer has the capacity to receive one more kitty
			let to_owned = <KittyOwned<T>>::get(&sender);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);

			let prev_kitty_owner = kitty.owner.clone();

			// Transfer the amount
			T::Currency::transfer(&sender, &prev_kitty_owner, bid_price, ExistenceRequirement::KeepAlive)?;

			// Transfer the kitty
			Self::transfer_kitty_to(&kitty_id, &sender, true)?;

			Self::deposit_event(Event::Bought(sender, prev_kitty_owner, kitty_id, bid_price));

			Ok(())
		}

		/// Breed a Kitty.
		///
		/// Breed two kitties to create a new generation
		/// of Kitties.
		///
		/// Weight: `O(1)`
		#[pallet::weight(100)]
		pub fn breed_kitty(origin: OriginFor<T>, kid1: T::Hash, kid2: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check: Verify `sender` owns both kitties (and both kitties exist).
			ensure!(Self::is_kitty_owner(&kid1, &sender)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&kid2, &sender)?, <Error<T>>::NotKittyOwner);

			let new_dna = Self::breed_dna(&kid1, &kid2)?;
			Self::mint(&sender, Some(new_dna), None)?;

			Ok(())
		}
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		fn breed_dna(kid1: &T::Hash, kid2: &T::Hash) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::kitties(kid1).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(kid2).ok_or(<Error<T>>::KittyNotExist)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}

		// Helper to mint a Kitty.
		fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>,
		) -> Result<T::Hash, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
			};

			let kitty_id = T::Hashing::hash_of(&kitty);

			// Performs this operation first because as it may fail
			let new_cnt = Self::kitty_cnt().checked_add(1)
				.ok_or(<Error<T>>::KittyCntOverflow)?;

			// Performs this operation first because as it may fail
			<KittyOwned<T>>::try_mutate(&owner, |kitty_vec| {
				kitty_vec.try_push(kitty_id)
			}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			<Kitties<T>>::insert(kitty_id, kitty);
			<KittyCnt<T>>::put(new_cnt);
			Ok(kitty_id)
		}

		fn is_kitty_owner(kitty_id: &T::Hash, acct: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(<Error<T>>::KittyNotExist)
			}
		}

		fn transfer_kitty_to(
			kitty_id: &T::Hash,
			to: &T::AccountId,
			reset_price: bool,
		) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

			let prev_owner = kitty.owner.clone();

			// Remove `kitty_id` from the KittyOwned vector of `prev_kitty_owner`
			<KittyOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(ind);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::KittyNotFound)?;

			// Update the kitty owner
			kitty.owner = to.clone();
			if reset_price {
				kitty.price = None;
			}
			<Kitties<T>>::insert(kitty_id, kitty);

			<KittyOwned<T>>::try_mutate(to, |vec| {
				vec.try_push(*kitty_id)
			}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			Ok(())
		}
	}
}
