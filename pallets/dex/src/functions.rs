use super::*;
use frame_support::traits::Get;

impl<T: Config> Pallet<T> {
	// todo: document
	pub(super) fn balance(id: AssetIdOf<T>, who: &AccountIdOf<T>) -> BalanceOf<T> {
		// Return balance of native currency if supplied asset id matches configured native asset id
		if id == T::NativeAssetId::get() {
			T::NativeCurrency::total_balance(who)
		} else {
			// Otherwise use asset balance
			T::Assets::balance(id, &who)
		}
	}
}
