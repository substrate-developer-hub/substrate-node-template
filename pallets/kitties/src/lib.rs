#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		sp_runtime::traits::{Hash, Zero},
		traits::{Currency, ExistenceRequirement, Randomness},
	};
	use frame_system::pallet_prelude::*;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, Default, PartialEq)]
	pub struct Kitty<Hash, Balance> {
		id: Hash,
		dna: Hash,
		price: Balance,
		gender: Gender,
	}

	// Set Gender type in Kitty struct.
	#[derive(Encode, Decode, Debug, Clone, PartialEq)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self {
			Gender::Male
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: pallet_balances::Config + frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The type of Random we want to specify for runtime.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		/// Nonce has overflowed past u64 limits
		NonceOverflow,
	}

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Created(T::AccountId, T::Hash),
		PriceSet(T::AccountId, T::Hash, T::Balance),
		Transferred(T::AccountId, T::AccountId, T::Hash),
		Bought(T::AccountId, T::AccountId, T::Hash, T::Balance),
	}

	// Storage items.

	// Keeps track of the Nonce used in the randomness generator.
	#[pallet::storage]
	#[pallet::getter(fn get_nonce)]
	pub(super) type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	// Stores a Kitty: it's unique traits and price.
	#[pallet::storage]
	#[pallet::getter(fn kitty)]
	pub(super) type Kitties<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, Kitty<T::Hash, T::Balance>, ValueQuery>;

	// Keeps track of what accounts own what Kitty.
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type KittyOwner<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, Option<T::AccountId>, ValueQuery>;

	// An index to track of all Kitties.
	#[pallet::storage]
	#[pallet::getter(fn kitty_by_index)]
	pub(super) type AllKittiesArray<T: Config> =
		StorageMap<_, Twox64Concat, u64, T::Hash, ValueQuery>;

	// Stores the total amount of Kitties in existence.
	#[pallet::storage]
	#[pallet::getter(fn all_kitties_count)]
	pub(super) type AllKittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	// Keeps track of all the Kitties.
	#[pallet::storage]
	pub(super) type AllKittiesIndex<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, u64, ValueQuery>;

	// Keep track of who a Kitty is owned by.
	#[pallet::storage]
	#[pallet::getter(fn kitty_of_owner_by_index)]
	pub(super) type OwnedKittiesArray<T: Config> =
		StorageMap<_, Twox64Concat, (T::AccountId, u64), T::Hash, ValueQuery>;

	// Keeps track of the total amount of Kitties owned.
	#[pallet::storage]
	#[pallet::getter(fn owned_kitty_count)]
	pub(super) type OwnedKittiesCount<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, u64, ValueQuery>;

	// Keeps track of all owned Kitties by index.
	#[pallet::storage]
	pub(super) type OwnedKittiesIndex<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, u64, ValueQuery>;

	// Our pallet's genesis configuration.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, T::Hash, T::Balance)>,
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
			for &(ref acct, hash, balance) in &self.kitties {
				let k = Kitty { id: hash, dna: hash, price: balance, gender: Gender::Male };

				let _ = <Pallet<T>>::mint(acct.clone(), hash, k);
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
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let random_hash = Self::random_hash(&sender);

			let new_kitty = Kitty {
				id: random_hash,
				dna: random_hash,
				price: 0u8.into(),
				gender: Kitty::<T, T>::gender(random_hash),
			};

			Self::mint(sender, random_hash, new_kitty)?;
			Self::increment_nonce()?;

			Ok(().into())
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
			new_price: T::Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// Make sure the Kitty exists.
			ensure!(<Kitties<T>>::contains_key(kitty_id), "This cat does not exist");

			// Check that the Kitty has an owner (i.e. if it exists).
			let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;

			// Make sure the owner matches the corresponding owner.
			ensure!(owner == sender, "You do not own this cat");

			// Set the Kitty price.
			let mut kitty = Self::kitty(kitty_id);
			kitty.price = new_price;

			// Update new Kitty infomation to storage.
			<Kitties<T>>::insert(kitty_id, kitty);

			// Deposit a "PriceSet" event.
			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));

			Ok(().into())
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
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// Verify Kitty owner: must be the account invoking this transaction.
			let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
			ensure!(owner == sender, "You do not own this kitty");

			// Transfer.
			Self::transfer_from(sender, to, kitty_id)?;

			Ok(().into())
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
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// Check if the Kitty exists.
			ensure!(<Kitties<T>>::contains_key(kitty_id), "This cat does not exist");

			// Check that the Kitty has an owner.
			let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;

			// Check that account buying the Kitty doesn't already own it.
			ensure!(owner != sender, "You can't buy your own cat");

			// Get the price of the Kitty.
			let mut kitty = Self::kitty(kitty_id);
			let kitty_price = kitty.price;

			// Check if the Kitty is for sale.
			ensure!(!kitty_price.is_zero(), "This Kitty is not for sale!");
			ensure!(kitty_price <= ask_price, "This Kitty is out of your budget!");

			// Update Balances using Currency trait.
			<pallet_balances::Pallet<T> as Currency<_>>::transfer(
				&sender,
				&owner,
				kitty_price,
				ExistenceRequirement::KeepAlive,
			)?;

			// Transfer ownership of Kitty.
			Self::transfer_from(owner.clone(), sender.clone(), kitty_id).expect(
				"`owner` is shown to own the kitty; \
                `owner` must have greater than 0 kitties, so transfer cannot cause underflow; \
                `all_kitty_count` shares the same type as `owned_kitty_count` \
                and minting ensure there won't ever be more than `max()` kitties, \
                which means transfer cannot cause an overflow; \
                qed",
			);

			// Set the price of the Kitty to the new price it was sold at.
			kitty.price = ask_price.into();
			<Kitties<T>>::insert(kitty_id, kitty);

			Self::deposit_event(Event::Bought(sender, owner, kitty_id, kitty_price));

			Ok(().into())
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
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			ensure!(<Kitties<T>>::contains_key(kitty_id_1), "This cat 1 does not exist");
			ensure!(<Kitties<T>>::contains_key(kitty_id_2), "This cat 2 does not exist");

			let random_hash = Self::random_hash(&sender);
			let kitty_1 = Self::kitty(kitty_id_1);
			let kitty_2 = Self::kitty(kitty_id_2);

			let mut final_dna = kitty_1.dna;
			for (i, (dna_2_element, r)) in
				kitty_2.dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate()
			{
				if r % 2 == 0 {
					final_dna.as_mut()[i] = *dna_2_element;
				}
			}

			let new_kitty = Kitty {
				id: random_hash,
				dna: final_dna,
				price: 0u8.into(),
				gender: Kitty::<T, T>::gender(final_dna),
			};

			Self::mint(sender, random_hash, new_kitty)?;
			Self::increment_nonce()?;

			Ok(().into())
		}
	}

	//** These are all our **//
	//** helper functions. **//

	impl<T: Config> Kitty<T, T> {
		pub fn gender(dna: T::Hash) -> Gender {
			if dna.as_ref()[0] % 2 == 0 {
				Gender::Male
			} else {
				Gender::Female
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Safely increment the nonce, with error on overflow
		fn increment_nonce() -> DispatchResult {
			<Nonce<T>>::try_mutate(|nonce| {
				let next = nonce.checked_add(1).ok_or(Error::<T>::NonceOverflow)?;
				*nonce = next;

				Ok(().into())
			})
		}

		/// Generate a random hash, using the nonce as part of the hash
		fn random_hash(sender: &T::AccountId) -> T::Hash {
			let nonce = <Nonce<T>>::get();
			let seed = T::KittyRandomness::random_seed();

			T::Hashing::hash_of(&(seed, &sender, nonce))
		}

		// Helper to mint a Kitty.
		fn mint(
			to: T::AccountId,
			kitty_id: T::Hash,
			new_kitty: Kitty<T::Hash, T::Balance>,
		) -> DispatchResult {
			ensure!(!<KittyOwner<T>>::contains_key(kitty_id), "Kitty already contains_key");

			// Update total Kitty counts.
			let owned_kitty_count = Self::owned_kitty_count(&to);
			let new_owned_kitty_count = owned_kitty_count
				.checked_add(1)
				.ok_or("Overflow adding a new kitty to account balance")?;

			let all_kitties_count = Self::all_kitties_count();
			let new_all_kitties_count = all_kitties_count
				.checked_add(1)
				.ok_or("Overflow adding a new kitty to total supply")?;

			// Update storage with new Kitty.
			<Kitties<T>>::insert(kitty_id, new_kitty);
			<KittyOwner<T>>::insert(kitty_id, Some(&to));

			// Write Kitty counting information to storage.
			<AllKittiesArray<T>>::insert(new_all_kitties_count, kitty_id);
			<AllKittiesCount<T>>::put(new_all_kitties_count);
			<AllKittiesIndex<T>>::insert(kitty_id, new_all_kitties_count);

			// Write Kitty counting information to storage.
			<OwnedKittiesArray<T>>::insert((to.clone(), new_owned_kitty_count), kitty_id);
			<OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count);
			<OwnedKittiesIndex<T>>::insert(kitty_id, new_owned_kitty_count);

			// Deposit our "Created" event.
			Self::deposit_event(Event::Created(to, kitty_id));

			Ok(())
		}
		// Helper to handle transferring a Kitty from one account to another.
		fn transfer_from(
			from: T::AccountId,
			to: T::AccountId,
			kitty_id: T::Hash,
		) -> DispatchResult {
			// Verify that the owner is the rightful owner of this Kitty.
			let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
			ensure!(owner == from, "'from' account does not own this kitty");

			// Address to send from.
			let owned_kitty_count_from = Self::owned_kitty_count(&from);

			// Address to send to.
			let owned_kitty_count_to = Self::owned_kitty_count(&to);

			// Increment the amount of owned Kitties by 1.
			let new_owned_kitty_count_to = owned_kitty_count_to
				.checked_add(1)
				.ok_or("Transfer causes overflow of 'to' kitty balance")?;

			// Increment the amount of owned Kitties by 1.
			let new_owned_kitty_count_from = owned_kitty_count_from
				.checked_sub(1)
				.ok_or("Transfer causes underflow of 'from' kitty balance")?;

			// Get current Kitty index.
			let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);

			// Update storage items that require updated index.
			if kitty_index != new_owned_kitty_count_from {
				let last_kitty_id =
					<OwnedKittiesArray<T>>::get((from.clone(), new_owned_kitty_count_from));
				<OwnedKittiesArray<T>>::insert((from.clone(), kitty_index), last_kitty_id);
				<OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
			}

			// Write new Kitty ownership to storage items.
			<KittyOwner<T>>::insert(&kitty_id, Some(&to));
			<OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);

			<OwnedKittiesArray<T>>::remove((from.clone(), new_owned_kitty_count_from));
			<OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

			<OwnedKittiesCount<T>>::insert(&from, new_owned_kitty_count_from);
			<OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);

			Self::deposit_event(Event::Transferred(from, to, kitty_id));

			Ok(())
		}
	}
}
