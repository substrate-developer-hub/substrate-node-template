#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::{
		sp_runtime::traits::Hash,
		traits::Randomness,
	};
	use sp_io::hashing::blake2_128;
	use sp_runtime::{
		traits::{ MaybeSerializeDeserialize }
	};
	use codec::Codec;


	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq)]
	pub struct Kitty<AccountId, Balance> {
		dna: [u8; 16],   // Using 16 bytes to represent a kitty DNA
		price: Option<Balance>,
		gender: Gender,
		owner: AccountId,
	}

	// Set Gender type in Kitty struct.
	#[derive(Encode, Decode, Debug, Clone, PartialEq)]
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

		type Balance: Parameter + From<u64> + Codec + MaybeSerializeDeserialize;

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
		KittyNotExist,
		KittyNotFound,
		NotKittyOwner,
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created(T::AccountId, T::Hash),
		PriceSet(T::AccountId, T::Hash, Option<T::Balance>),
		Transferred(T::AccountId, T::AccountId, T::Hash),
		Bought(T::AccountId, T::AccountId, T::Hash, T::Balance),
	}

	// Storage items.

	// Keeps track of the KittyCount
	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	// Stores a Kitty: it's unique traits and price.
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, Kitty<T::AccountId, T::Balance>>;

	// Keeps track of what accounts own what Kitty.
	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub(super) type KittyOwned<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::Hash, T::MaxKittyOwned>, ValueQuery>;

	// Our pallet's genesis configuration.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<T::AccountId>,
	}

	// Required to implement default for GenesisConfig.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for acct in &self.kitties {
				let _ = <Pallet<T>>::mint(acct);
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

			let kitty_id = Self::mint(&sender)?;

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
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			new_price: Option<T::Balance>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Retrieve the kitty. If not existed, return error
			let mut kitty = <Kitties<T>>::try_get(&kitty_id)
				.map_err(|_| <Error<T>>::KittyNotExist)?;

			// Make sure the owner matches the corresponding owner.
			ensure!(Self::kitty_owned_by(&sender, &kitty_id), <Error<T>>::NotKittyOwner);

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
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Make sure the Kitty exists.
			ensure!(<Kitties<T>>::contains_key(&kitty_id), <Error<T>>::KittyNotExist);

			// Verify Kitty owner: must be the account invoking this transaction.
			ensure!(Self::kitty_owned_by(&sender, &kitty_id), <Error<T>>::NotKittyOwner);

			// Verify the `to` has the capacity to receive one more kitty
			let to_owned = <KittyOwned<T>>::get(&to);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);

			let _ = Self::transfer_owner(&kitty_id, &to);

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
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::Hash,
			ask_price: T::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Ok(())

			// // Check if the Kitty exists.
			// ensure!(<Kitties<T>>::contains_key(kitty_id), "This cat does not exist");

			// // Check that the Kitty has an owner.
			// let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;

			// // Check that account buying the Kitty doesn't already own it.
			// ensure!(owner != sender, "You can't buy your own cat");

			// // Get the price of the Kitty.
			// let mut kitty = Self::kitty(kitty_id);
			// let kitty_price = kitty.price;

			// // Check if the Kitty is for sale.
			// ensure!(!kitty_price.is_zero(), "This Kitty is not for sale!");
			// ensure!(kitty_price <= ask_price, "This Kitty is out of your budget!");

			// // Update Balances using Currency trait.
			// <pallet_balances::Pallet<T> as Currency<_>>::transfer(
			// 	&sender,
			// 	&owner,
			// 	kitty_price,
			// 	ExistenceRequirement::KeepAlive,
			// )?;

			// // Transfer ownership of Kitty.
			// Self::transfer_from(owner.clone(), sender.clone(), kitty_id).expect(
			// 	"`owner` is shown to own the kitty; \
			//    `owner` must have greater than 0 kitties, so transfer cannot cause underflow; \
			//    `all_kitty_count` shares the same type as `owned_kitty_count` \
			//    and minting ensure there won't ever be more than `max()` kitties, \
			//    which means transfer cannot cause an overflow; \
			//    qed",
			// );

			// // Set the price of the Kitty to the new price it was sold at.
			// kitty.price = ask_price.into();
			// <Kitties<T>>::insert(kitty_id, kitty);

			// Self::deposit_event(Event::Bought(sender, owner, kitty_id, kitty_price));

			// Ok(().into())
		}

		/// Breed a Kitty.
		///
		/// Breed two kitties to create a new generation
		/// of Kitties.
		///
		/// Weight: `O(1)`
		#[pallet::weight(100)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			kitty_id_1: T::Hash,
			kitty_id_2: T::Hash,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Ok(())

			// ensure!(<Kitties<T>>::contains_key(kitty_id_1), "This cat 1 does not exist");
			// ensure!(<Kitties<T>>::contains_key(kitty_id_2), "This cat 2 does not exist");

			// let random_hash = Self::random_hash(&sender);
			// let kitty_1 = Self::kitty(kitty_id_1);
			// let kitty_2 = Self::kitty(kitty_id_2);

			// let mut final_dna = kitty_1.dna;
			// for (i, (dna_2_element, r)) in
			// 	kitty_2.dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate()
			// {
			// 	if r % 2 == 0 {
			// 		final_dna.as_mut()[i] = *dna_2_element;
			// 	}
			// }

			// let new_kitty = Kitty {
			// 	id: random_hash,
			// 	dna: final_dna,
			// 	price: 0u8.into(),
			// 	gender: Kitty::<T, T>::gender(final_dna),
			// };

			// Self::mint(sender, random_hash, new_kitty)?;
			// Self::increment_nonce()?;

			// Ok(().into())
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

		// Helper to mint a Kitty.
		fn mint(owner: &T::AccountId) -> Result<T::Hash, Error<T>> {

			let kitty = Kitty::<T::AccountId, T::Balance> {
				dna: Self::gen_dna(),
				price: None,
				gender: Self::gen_gender(),
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

		fn kitty_owned_by(acct: &T::AccountId, kitty_id: &T::Hash) -> bool {
			match Self::kitties(kitty_id) {
				Some(kitty) => kitty.owner == *acct,
				None => false
			}
		}

		fn transfer_owner(kitty_id: &T::Hash, to: &T::AccountId) -> Result<(), Error<T>> {
			let mut kitty = <Kitties<T>>::try_get(kitty_id)
				.map_err(|_| <Error<T>>::KittyNotExist)?;

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
			<Kitties<T>>::insert(kitty_id, kitty);

			<KittyOwned<T>>::try_mutate(to, |vec| {
				vec.try_push(*kitty_id)
			}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			Ok(())
		}
	}
}
