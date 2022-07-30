#![cfg_attr(not(feature = "std"), no_std)]
use codec::HasCompact;
use frame_support::{dispatch::DispatchResult, traits::fungibles::Inspect, traits::Currency};
use frame_system::pallet_prelude::OriginFor;
pub use pallet::*;
pub use types::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod functions;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AssetIdOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type NativeBalanceOf<T> =
	<<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type PriceOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::fungibles::Mutate;
	use frame_support::traits::ReservableCurrency;
	use frame_support::traits::UnixTime;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// Identifier type for a fungible asset
		type AssetId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ PartialOrd;

		// Balance inspection for fungible assets
		type Assets: Mutate<
			Self::AccountId,
			AssetId = Self::AssetId,
			Balance = NativeBalanceOf<Self>, // Constrain balance type to same as native currency
		>;

		// Native currency: for swaps between native token and other assets
		type NativeCurrency: ReservableCurrency<Self::AccountId>;

		/// Identifier of the native asset identifier (proxy between native token and asset)
		#[pallet::constant]
		type NativeAssetId: Get<Self::AssetId>;

		type Time: UnixTime;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Genesis liquidity pools: ((amount, asset), (amount, asset), liquidity provider)
		pub liquidity_pools:
			Vec<((BalanceOf<T>, AssetIdOf<T>), (BalanceOf<T>, AssetIdOf<T>), AccountIdOf<T>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { liquidity_pools: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (amount_0, amount_1, liquidity_provider) in &self.liquidity_pools {
				let pair = Pair::<T>::new(*amount_0, *amount_1);
				let key = pair.key();
				assert!(
					!LiquidityPools::<T>::contains_key(key),
					"Liquidity pool id already in use"
				);
				assert!(pair.0.amount > BalanceOf::<T>::default(), "Amount should not be zero");
				assert!(pair.1.amount > BalanceOf::<T>::default(), "Amount should not be zero");

				// Create liquidity pool and add liquidity

				let mut liquidity_pool = LiquidityPool::<T>::new((pair.0.asset, pair.1.asset));
				let price = liquidity_pool
					.add_liquidity(&pair.0, &pair.1, &liquidity_provider)
					.expect("Expect to be able to add liquidity during genesis.");
				LiquidityPools::<T>::insert(key, liquidity_pool);
				Price::<T>::insert(key, price);
			}
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Stores liquidity pools based on composite key of asset pair
	#[pallet::storage]
	pub(super) type LiquidityPools<T: Config> =
		StorageMap<_, Twox64Concat, (super::AssetIdOf<T>, super::AssetIdOf<T>), LiquidityPool<T>>;

	/// Stores liquidity pool price based on composite key of asset pair
	#[pallet::storage]
	pub(super) type Price<T: Config> =
		StorageMap<_, Twox64Concat, (super::AssetIdOf<T>, super::AssetIdOf<T>), PriceOf<T>>;

	// // The pallet's runtime storage items.
	// // https://docs.substrate.io/v3/runtime/storage
	// #[pallet::storage]
	// #[pallet::getter(fn something)]
	// // Learn more about declaring storage items:
	// // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	// pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// todo: add description [amount_0, asset_0, amount_1, asset_1]
		LiquidityAdded(BalanceOf<T>, AssetIdOf<T>, BalanceOf<T>, AssetIdOf<T>, PriceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		NoPool,
		InsufficientOutputAmount,
		InsufficientBalance,
		InvalidAsset,
		DeadlinePassed,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// todo: document
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			amount_0: BalanceOf<T>,
			asset_0: AssetIdOf<T>,
			amount_1: BalanceOf<T>,
			asset_1: AssetIdOf<T>,
			deadline: u64,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let liquidity_provider = ensure_signed(origin)?;

			ensure!(deadline > T::Time::now().as_secs(), Error::<T>::DeadlinePassed);

			// Create pair from amounts
			let pair = Pair::<T>::new((amount_0, asset_0), (amount_1, asset_1));

			// Ensure assets exists
			ensure!(
				Self::exists(pair.0.asset) && Self::exists(pair.1.asset),
				Error::<T>::InvalidAsset
			);

			// Get/create liquidity pool
			let key = pair.key();
			let mut pool =
				<LiquidityPools<T>>::get(key).unwrap_or_else(|| <LiquidityPool<T>>::new(key));

			// Add liquidity to pool
			let price = pool.add_liquidity(&pair.0, &pair.1, &liquidity_provider)?;

			// Update
			<LiquidityPools<T>>::set(key, Some(pool));
			<Price<T>>::set(key, Some(price));

			// Emit an event.
			Self::deposit_event(Event::LiquidityAdded(
				pair.0.amount,
				pair.0.asset,
				pair.1.amount,
				pair.1.asset,
				price,
			));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// todo: document
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn swap(
			origin: OriginFor<T>,
			amount_0: BalanceOf<T>,
			asset_0: AssetIdOf<T>,
			amount_1: BalanceOf<T>,
			asset_1: AssetIdOf<T>,
		) -> DispatchResult {
			// Ensure signed
			let who = ensure_signed(origin)?;

			// Create pair from amounts
			let pair = Pair::<T>::new((amount_0, asset_0), (amount_1, asset_1));

			// Verify the output amounts
			ensure!(
				pair.0.amount != BalanceOf::<T>::default()
					|| pair.1.amount != BalanceOf::<T>::default(),
				Error::<T>::InsufficientOutputAmount
			);

			// Verify sender has sufficient balance of each asset
			let balance_0 = Self::balance(pair.0.asset, &who);
			ensure!(balance_0 >= pair.0.amount, Error::<T>::InsufficientBalance);
			let balance_1 = Self::balance(pair.1.asset, &who);
			ensure!(balance_1 >= pair.1.amount, Error::<T>::InsufficientBalance);

			match <LiquidityPools<T>>::get((pair.0.asset, pair.1.asset)) {
				None => return Err(DispatchError::from(Error::<T>::NoPool)),
				Some(pool) => {
					//pool.swap()
				},
			}

			// Update storage.
			//<Something<T>>::put(something);

			// Emit an event.
			//			Self::deposit_event(Event::SomethingStored(something, who));

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// /// An example dispatchable that may throw a custom error.
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;
		//
		// 	// Read a value from storage.
		// 	match <Something<T>>::get() {
		// 		// Return an error if the value has not been set.
		// 		None => return Err(Error::<T>::NoneValue.into()),
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
		// 			<Something<T>>::put(new);
		// 			Ok(())
		// 		},
		// 	}
		// }
	}

	impl<T: Config> Swap<T::AccountId> for Pallet<T> {
		type AssetId = T::AssetId;
		type Balance = <<T as pallet::Config>::Assets as Inspect<
			<T as frame_system::Config>::AccountId,
		>>::Balance;

		fn swap(
			origin: T::AccountId,
			amount_0: Self::Balance,
			asset_0: Self::AssetId,
			amount_1: Self::Balance,
			asset_1: Self::AssetId,
		) -> DispatchResult {
			todo!()
		}
	}
}

// NOTE: Should be defined in a separate crate for loose coupling
pub trait Swap<AccountId> {
	type AssetId;
	type Balance;
	fn swap(
		origin: AccountId,
		amount_0: Self::Balance,
		asset_0: Self::AssetId,
		amount_1: Self::Balance,
		asset_1: Self::AssetId,
	) -> DispatchResult;
}
