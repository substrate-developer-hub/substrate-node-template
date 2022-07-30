use super::*;
use frame_support::traits::Get;

impl<T: Config> Pallet<T> {
	// todo: document
	pub(super) fn exists(id: AssetIdOf<T>) -> bool {
		// todo: improve to query storage for key only (contains_key)
		T::Assets::total_issuance(id) > BalanceOf::<T>::default()
	}

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
