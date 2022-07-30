use super::*;
use frame_support::{pallet_prelude::*, traits::fungibles::Mutate};
use sp_runtime::traits::CheckedAdd;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct LiquidityPool<T: Config> {
	pub(super) balance_0: BalanceOf<T>,
	pub(super) balance_1: BalanceOf<T>,
	pub(super) liquidity_token: AssetIdOf<T>,
}

impl<T: Config> LiquidityPool<T> {
	pub(super) fn new(pair: (AssetIdOf<T>, AssetIdOf<T>)) -> Self {
		// todo: replace with hash
		let liquidity_token = AssetIdOf::<T>::default();
		Self {
			balance_0: <BalanceOf<T>>::default(),
			balance_1: <BalanceOf<T>>::default(),
			liquidity_token,
		}
	}

	// todo: document
	pub(super) fn add_liquidity(
		&mut self,
		amount_0: &Amount<T>,
		amount_1: &Amount<T>,
		liquidity_provider: &AccountIdOf<T>,
	) -> Result<PriceOf<T>, sp_runtime::DispatchError> {
		// todo: adjust balances of assets (reserving if native)
		// todo: ensure ratio
		self.balance_0.checked_add(&amount_0.amount);
		self.balance_1.checked_add(&amount_1.amount);

		// Calculate liquidity rewards and mint new tokens
		let liquidity_rewards =
			amount_0.amount * T::Assets::total_issuance(self.liquidity_token) / self.balance_0;
		T::Assets::mint_into(self.liquidity_token, liquidity_provider, liquidity_rewards)?;

		// Finally return updated price
		Ok(self.balance_0 * self.balance_1)
	}

	pub(super) fn remove_liquidity(
		&mut self,
		_amount_0: &Amount<T>,
		_amount_1: &Amount<T>,
		_liquidity_provider: &AccountIdOf<T>,
	) -> PriceOf<T> {
		// Finally return updated price
		self.balance_0 * self.balance_1
	}
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Amount<T: Config> {
	pub(super) amount: BalanceOf<T>,
	pub(super) asset: AssetIdOf<T>,
}

impl<T: Config> Amount<T> {
	fn new(amount: BalanceOf<T>, asset: AssetIdOf<T>) -> Self {
		Self { amount, asset }
	}
}

pub struct Pair<T: Config>(pub(super) Amount<T>, pub(super) Amount<T>);

impl<T: Config> Pair<T> {
	// todo: document
	pub(super) fn new(
		amount_0: (BalanceOf<T>, AssetIdOf<T>),
		amount_1: (BalanceOf<T>, AssetIdOf<T>),
	) -> Self {
		let amount_0 = Amount::<T>::new(amount_0.0, amount_0.1);
		let amount_1 = Amount::<T>::new(amount_1.0, amount_1.1);

		// Sort by asset id so always in same order
		if amount_1.asset < amount_0.asset {
			Self(amount_1, amount_0)
		} else {
			Self(amount_0, amount_1)
		}
	}

	// todo: document
	pub(super) fn key(&self) -> (AssetIdOf<T>, AssetIdOf<T>) {
		(self.0.asset, self.1.asset)
	}
}
