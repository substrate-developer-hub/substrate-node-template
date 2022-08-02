use super::*;
use frame_support::traits::{ExistenceRequirement, Get};

impl<T: Config> Pallet<T> {
	/// Get the `asset` balance of `who`   
	/// **Note:** this is a wrapper function for handling native and custom asset balances.
	pub(super) fn balance(id: AssetIdOf<T>, who: &AccountIdOf<T>) -> BalanceOf<T> {
		// Return balance of native currency if supplied asset id matches configured native asset id
		if id == T::NativeAssetId::get() {
			T::NativeCurrency::total_balance(who)
		} else {
			// Otherwise use asset balance
			T::Assets::balance(id, &who)
		}
	}

	/// Calculates the output amount of asset `other`, given an input `amount` and `asset` type.
	/// # Arguments
	/// * `amount` - An amount to be valued.
	/// * `asset` - The asset type of the amount.
	/// * `other` - The required asset type.
	pub fn price(
		amount: BalanceOf<T>,
		asset: AssetIdOf<T>,
		other: AssetIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		ensure!(amount > <BalanceOf<T>>::default(), Error::<T>::InvalidAmount);
		<LiquidityPool<T>>::price((amount, asset), other)
	}

	/// Transfer funds from one account into another. The default implementation uses `mint_into`
	/// and `burn_from` and may generate unwanted events.
	/// **Note:** this is a wrapper function for handling native and custom asset transfers.
	pub(super) fn transfer(
		asset: AssetIdOf<T>,
		source: &AccountIdOf<T>,
		dest: &AccountIdOf<T>,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		// Use native currency if supplied asset id matches configured native asset id
		if asset == T::NativeAssetId::get() {
			T::NativeCurrency::transfer(source, dest, amount, ExistenceRequirement::AllowDeath)
		} else {
			// Otherwise use asset transfer.
			T::Assets::teleport(asset, source, dest, amount).map(|_| ())
		}
	}
}
