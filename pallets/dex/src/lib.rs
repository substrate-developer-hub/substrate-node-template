#![cfg_attr(not(feature = "std"), no_std)]
use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult, pallet_prelude::*, traits::fungibles::Create,
	traits::fungibles::Inspect, traits::fungibles::Mutate,
	traits::tokens::fungibles::metadata::Mutate as MutateMetadata, traits::Currency,
	traits::ReservableCurrency, traits::Time, PalletId,
};
use frame_system::pallet_prelude::OriginFor;
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::{traits::AtLeast32BitUnsigned, traits::Bounded};
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
type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// Identifier type for a fungible asset
		type AssetId: AtLeast32BitUnsigned
			+ Bounded
			+ HasCompact
			+ MaybeSerializeDeserialize
			+ Member
			+ TypeInfo
			+ Member
			+ Parameter
			+ Default
			+ Copy
			+ MaxEncodedLen
			+ PartialOrd;
		// Balance inspection for fungible assets
		type Assets: Create<Self::AccountId>
			+ Mutate<
				Self::AccountId,
				AssetId = Self::AssetId,
				Balance = NativeBalanceOf<Self>, // Constrain balance type to same as native currency
			> + MutateMetadata<
				Self::AccountId,
				AssetId = Self::AssetId,
				Balance = NativeBalanceOf<Self>,
			> + StorageInfoTrait;
		// The minimum balance of the liquidity pool token (must be non-zero)
		type LiquidityPoolTokenMinimumBalance: Get<
			<Self::Assets as Inspect<<Self as frame_system::Config>::AccountId>>::Balance,
		>;
		// The number of decimals used for the liquidity pool token
		type LiquidityPoolTokenDecimals: Get<u8>;
		// The minimum level of liquidity in a pool
		type MinimumLiquidity: Get<u32>;
		// Native currency: for swaps between native token and other assets
		type NativeCurrency: ReservableCurrency<Self::AccountId>;
		/// Identifier of the native asset identifier (proxy between native token and asset)
		#[pallet::constant]
		type NativeAssetId: Get<Self::AssetId>;
		/// The DEX's pallet id, used for deriving its sovereign account
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// The units used when determining the swap fee (e.g. 1,000)
		type SwapFeeUnits: Get<
			<Self::Assets as Inspect<<Self as frame_system::Config>::AccountId>>::Balance,
		>;
		/// The value used to determine the swap fee rate (e.g. 1,000 - 997 = 0.03%)
		type SwapFeeValue: Get<
			<Self::Assets as Inspect<<Self as frame_system::Config>::AccountId>>::Balance,
		>;
		// A provider of time
		type Time: Time;
		// Call out to runtime to have it provide result
		// NOTE: no easy way to determine if an asset exists via loose-coupling, so this provides a simple layer of
		// indirection to work around this without tight coupling to the assets pallet
		fn exists(id: Self::AssetId) -> bool;
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
				let pair = <Pair<T>>::from_values(amount_0.0, amount_0.1, amount_1.0, amount_1.1);
				let key = (pair.0.asset, pair.1.asset);
				assert!(
					!LiquidityPools::<T>::contains_key(key),
					"Liquidity pool id already in use"
				);
				assert!(pair.0.value > <BalanceOf<T>>::default(), "Amount should not be zero");
				assert!(pair.1.value > <BalanceOf<T>>::default(), "Amount should not be zero");

				// Create liquidity pool and add liquidity
				let liquidity_pool = LiquidityPool::<T>::new(key)
					.expect("Expect to be able to create a new liquidity pool during genesis.");
				let price = liquidity_pool
					.add((pair.0.value, pair.1.value), &liquidity_provider)
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
		StorageMap<_, Twox64Concat, (AssetIdOf<T>, AssetIdOf<T>), LiquidityPool<T>>;

	/// Stores a counter for liquidity pool token identifiers (starting at max and counting down).
	#[pallet::storage]
	pub(super) type LiquidityPoolTokenIdGenerator<T: Config> = StorageValue<_, AssetIdOf<T>>;

	/// Stores liquidity pool price based on composite key of asset pair
	#[pallet::storage]
	pub(super) type Price<T: Config> =
		StorageMap<_, Twox64Concat, (AssetIdOf<T>, AssetIdOf<T>), PriceOf<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Liquidity pool created [asset_0, asset_1]
		LiquidityPoolCreated(AssetIdOf<T>, AssetIdOf<T>),
		// Liquidity has been added to the pool [amount_0, asset_0, amount_1, asset_1]
		LiquidityAdded(BalanceOf<T>, AssetIdOf<T>, BalanceOf<T>, AssetIdOf<T>),
		// Liquidity pool price changed [asset_0, asset_1, price]
		PriceChanged(AssetIdOf<T>, AssetIdOf<T>, PriceOf<T>),
		// Liquidity has been removed from the pool [amount_0, asset_0, amount_1, asset_1]
		LiquidityRemoved(BalanceOf<T>, AssetIdOf<T>, BalanceOf<T>, AssetIdOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// Identical assets provided.
		IdenticalAssets,
		// An invalid amount was provided.
		InvalidAmount,
		// The asset identifier already exists.
		AssetAlreadyExists,

		// todo:
		NoPool,
		InsufficientOutputAmount,
		InsufficientBalance,
		InvalidAsset,
		DeadlinePassed,
	}

	// Dispatchable functions which materialize as "extrinsics"
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adds liquidity to a pool, with liquidity pool (LP) tokens being minted for the liquidity provider
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			amount_0: BalanceOf<T>,
			asset_0: AssetIdOf<T>,
			amount_1: BalanceOf<T>,
			asset_1: AssetIdOf<T>,
			deadline: MomentOf<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let liquidity_provider = ensure_signed(origin)?;

			// Check inputs
			ensure!(asset_0 != asset_1, Error::<T>::IdenticalAssets); // Check if same asset supplied
			ensure!(
				amount_0 != <BalanceOf<T>>::default() && amount_1 != <BalanceOf<T>>::default(),
				Error::<T>::InvalidAmount
			); // Check if either amount invalid
			ensure!(T::exists(asset_0) && T::exists(asset_1), Error::<T>::InvalidAsset); // Ensure assets exists
			ensure!(
				Self::balance(asset_0, &liquidity_provider) >= amount_0
					&& Self::balance(asset_1, &liquidity_provider) >= amount_1,
				Error::<T>::InsufficientBalance
			); // Ensure sufficient balance
			ensure!(deadline > T::Time::now(), Error::<T>::DeadlinePassed); // Check whether deadline passed

			// Create pair from supplied values
			let pair = <Pair<T>>::from_values(amount_0, asset_0, amount_1, asset_1);

			// Get/create liquidity pool
			let key = (pair.0.asset, pair.1.asset);
			let pool = match <LiquidityPools<T>>::get(key) {
				Some(pool) => Result::<LiquidityPool<T>, DispatchError>::Ok(pool), // Type couldnt be inferred
				None => {
					// Create new pool, save and emit event
					let pool = <LiquidityPool<T>>::new(key)?;
					<LiquidityPools<T>>::set(key, Some(pool.clone()));
					Self::deposit_event(Event::LiquidityPoolCreated(pair.0.asset, pair.1.asset));
					Ok(pool)
				},
			}?;

			// Add liquidity to pool and emit event
			let price = pool.add((pair.0.value, pair.1.value), &liquidity_provider)?;
			Self::deposit_event(Event::LiquidityAdded(
				pair.0.value,
				pair.0.asset,
				pair.1.value,
				pair.1.asset,
			));

			// Finally update price oracle, emit event and return success
			<Price<T>>::set(key, Some(price));
			Self::deposit_event(Event::PriceChanged(pair.0.asset, pair.1.asset, price));
			Ok(())
		}

		// todo: document
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			asset_0: AssetIdOf<T>,
			asset_1: AssetIdOf<T>,
			deadline: MomentOf<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let liquidity_provider = ensure_signed(origin)?;

			// Check inputs
			ensure!(asset_0 != asset_1, Error::<T>::IdenticalAssets); // Check if same asset supplied
			ensure!(amount != <BalanceOf<T>>::default(), Error::<T>::InvalidAmount); // Check if amount invalid
			ensure!(deadline > T::Time::now(), Error::<T>::DeadlinePassed); // Check whether deadline passed

			// Get liquidity pool
			let pair = <Pair<T>>::from(asset_0, asset_1);
			let mut pool = match <LiquidityPools<T>>::get(pair) {
				Some(pool) => Result::<LiquidityPool<T>, DispatchError>::Ok(pool), // Type couldnt be inferred
				None => Err(DispatchError::from(Error::<T>::NoPool)),
			}?;

			// Remove liquidity from pool and emit event
			let (amount, price) = pool.remove(amount, &liquidity_provider)?;
			Self::deposit_event(Event::LiquidityRemoved(amount.0, pair.0, amount.1, pair.1));

			// Finally update price oracle, emit event and return success
			let event = Event::PriceChanged(pair.0, pair.1, price);
			<Price<T>>::set((pair.0, pair.1), Some(price));
			Self::deposit_event(event);
			Ok(())
		}

		// todo: document
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn swap(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			asset: AssetIdOf<T>,
			counter: AssetIdOf<T>,
		) -> DispatchResult {
			// Ensure signed
			let who = ensure_signed(origin)?;

			// Verify the amounts
			ensure!(amount != <BalanceOf<T>>::default(), Error::<T>::InsufficientOutputAmount);

			// Verify sender has sufficient balance of asset
			let balance = Self::balance(asset, &who);
			ensure!(balance >= amount, Error::<T>::InsufficientBalance);

			let pair = <Pair<T>>::from(asset, counter);
			match <LiquidityPools<T>>::get(pair) {
				None => return Err(DispatchError::from(Error::<T>::NoPool)),
				Some(_pool) => {
					todo!()
				},
			}

			todo!()

			// Update storage.
			//<Something<T>>::put(something);

			// Emit an event.
			//			Self::deposit_event(Event::SomethingStored(something, who));

			// Return a successful DispatchResultWithPostInfo
			//Ok(())
		}
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
