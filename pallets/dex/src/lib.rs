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
pub mod traits;
mod types;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AssetIdOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type NativeBalanceOf<T> =
	<<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// The configuration for the pallet.
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
		// todo: explore use of ReservableCurrency for locking native funds when adding to liquidity pool
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

		/// The value used to determine the swap fee rate (e.g. 1,000 - 997 = 0.3%)
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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Stores liquidity pools based on composite key of asset pair.
	#[pallet::storage]
	pub(super) type LiquidityPools<T: Config> =
		StorageMap<_, Twox64Concat, (AssetIdOf<T>, AssetIdOf<T>), LiquidityPool<T>>;

	/// Stores a simple counter for liquidity pool token identifiers (starting at AssetIdOf<T>::max_value() and
	/// counting down).
	#[pallet::storage]
	pub(super) type LiquidityPoolTokenIdGenerator<T: Config> = StorageValue<_, AssetIdOf<T>>;

	// The various events emitted by the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// A new liquidity pool was created [asset_0, asset_1]
		LiquidityPoolCreated(AssetIdOf<T>, AssetIdOf<T>),
		// Liquidity has been added to the pool [amount_0, asset_0, amount_1, asset_1]
		LiquidityAdded(BalanceOf<T>, AssetIdOf<T>, BalanceOf<T>, AssetIdOf<T>),
		// Liquidity has been removed from the pool [amount_0, asset_0, amount_1, asset_1, lp_tokens]
		LiquidityRemoved(BalanceOf<T>, AssetIdOf<T>, BalanceOf<T>, AssetIdOf<T>, BalanceOf<T>),
		// A swap has been completed, showing the input amount of one asset and the resulting output amount of another
		// [input_amount, input_asset, output_amount, output_asset]
		Swapped(BalanceOf<T>, AssetIdOf<T>, BalanceOf<T>, AssetIdOf<T>),
	}

	// The various errors returned by the pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// The asset identifier already exists.
		AssetAlreadyExists,
		/// The specified deadline has passed.
		DeadlinePassed,
		// The pool is empty.
		EmptyPool,
		/// Identical assets provided.
		IdenticalAssets,
		// The current balance is insufficient.
		InsufficientBalance,
		/// An invalid amount was provided.
		InvalidAmount,
		/// The asset does not exist.
		InvalidAsset,
		/// No pool could be found.
		NoPool,
	}

	// The various calls made available by the pallet (dispatchable functions which materialize as "extrinsics").
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adds liquidity to a pool in the form of a pair of asset amounts, with liquidity pool (LP) tokens being
		/// minted for the liquidity provider.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `amount_0` - The first amount of the pair.
		/// * `asset_0` - The identifier of the first asset of the pair.
		/// * `amount_1` - The other amount of the pair.
		/// * `asset_1` - The identifier of the other asset of the pair.
		/// * `deadline` - At deadline at which the transaction is no longer valid.
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
			); // Check if either amount valid
			ensure!(T::exists(asset_0) && T::exists(asset_1), Error::<T>::InvalidAsset); // Ensure assets exists
			ensure!(
				Self::balance(asset_0, &liquidity_provider) >= amount_0
					&& Self::balance(asset_1, &liquidity_provider) >= amount_1,
				Error::<T>::InsufficientBalance
			); // Ensure sufficient balance of both assets
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
			pool.add((pair.0.value, pair.1.value), &liquidity_provider)?;
			Self::deposit_event(Event::LiquidityAdded(
				pair.0.value,
				pair.0.asset,
				pair.1.value,
				pair.1.asset,
			));
			Ok(())
		}

		/// Removes liquidity from a pool by redeeming liquidity pool (LP) tokens for the corresponding assets and
		/// rewards.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `amount` - The amount of liquidity pool tokens being redeemed.
		/// * `asset_0` - The identifier of the first asset of the pair, used to identify the liquidity pool.
		/// * `asset_1` - The identifier of the other asset of the pair, used to identify the liquidity pool.
		/// * `deadline` - At deadline at which the transaction is no longer valid.
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
			ensure!(amount != <BalanceOf<T>>::default(), Error::<T>::InvalidAmount); // Check if amount valid
			ensure!(deadline > T::Time::now(), Error::<T>::DeadlinePassed); // Check whether deadline passed

			// NOTE:
			// - individual assets identifiers not checked here as we just attempt to look up pool using them below
			// - balance checked in pool.remove()

			// Get liquidity pool
			let pair = <Pair<T>>::from(asset_0, asset_1);
			let pool = match <LiquidityPools<T>>::get(pair) {
				Some(pool) => Result::<LiquidityPool<T>, DispatchError>::Ok(pool), // Type couldnt be inferred
				None => Err(DispatchError::from(Error::<T>::NoPool)),
			}?;

			// Remove liquidity from pool and emit event
			let output = pool.remove(amount, &liquidity_provider)?;
			Self::deposit_event(Event::LiquidityRemoved(
				output.0 .0,
				output.0 .1,
				output.1 .0,
				output.1 .1,
				amount,
			));
			Ok(())
		}

		/// Swaps an amount of some asset for another asset.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `amount` - The amount of the asset to be swapped.
		/// * `asset` - The identifier of the asset being swapped.
		/// * `other` - The identifier of the other asset being requested.
		/// * `deadline` - At deadline at which the transaction is no longer valid.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn swap(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			asset: AssetIdOf<T>,
			other: AssetIdOf<T>,
			deadline: MomentOf<T>,
		) -> DispatchResult {
			// Ensure signed and get the buyer
			let buyer = ensure_signed(origin)?;

			// Check inputs (note: remaining checked in swap implementation)
			ensure!(deadline > T::Time::now(), Error::<T>::DeadlinePassed); // Check whether deadline passed

			// Forward to trait implementation (below)
			<Self as traits::Swap<T::AccountId>>::swap(amount, asset, other, buyer)
		}
	}

	/// Trait for exposing asset swapping to other pallets.  
	impl<T: Config> traits::Swap<T::AccountId> for Pallet<T> {
		type AssetId = T::AssetId;
		type Balance = <<T as pallet::Config>::Assets as Inspect<
			<T as frame_system::Config>::AccountId,
		>>::Balance;

		/// Performs a swap of an `amount` of the specified `asset` to the `other` asset.  
		/// # Arguments
		/// * `amount` - An amount to be swapped.
		/// * `asset` - The identifier of the asset type to be swapped.
		/// * `other` - The identifier of the other asset type.
		/// * `buyer` - The identifier of the account initiating the swap.
		fn swap(
			amount: Self::Balance,
			asset: Self::AssetId,
			other: Self::AssetId,
			buyer: T::AccountId,
		) -> DispatchResult {
			// Check inputs
			ensure!(amount != <BalanceOf<T>>::default(), Error::<T>::InvalidAmount); // Verify the amounts
			ensure!(Self::balance(asset, &buyer) >= amount, Error::<T>::InsufficientBalance); // Verify sender has sufficient balance of asset

			// NOTE: individual assets identifiers not checked here as we just attempt to look up pool using them below

			// Get liquidity pool
			let pair = <Pair<T>>::from(asset, other);
			let pool = match <LiquidityPools<T>>::get(pair) {
				Some(pool) => Ok(pool),
				None => Err(DispatchError::from(Error::<T>::NoPool)),
			}?;

			// Finally perform swap and emit event
			let output = pool.swap((amount, asset), &buyer)?;
			Self::deposit_event(Event::Swapped(amount, asset, output.0, output.1));
			Ok(())
		}
	}

	/// Configuration of the DEX state at genesis
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
				let pool = LiquidityPool::<T>::new(key)
					.expect("Expect to be able to create a new liquidity pool during genesis.");
				pool.add((pair.0.value, pair.1.value), &liquidity_provider)
					.expect("Expect to be able to add liquidity during genesis.");
				LiquidityPools::<T>::insert(key, pool);
			}
		}
	}
}
