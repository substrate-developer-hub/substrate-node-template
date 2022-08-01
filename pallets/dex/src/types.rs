use super::*;
use frame_support::{
	traits::fungibles::metadata::Inspect as InspectMetadata,
	traits::fungibles::Mutate,
	traits::fungibles::{Create, Inspect},
};
use sp_runtime::{traits::AccountIdConversion, traits::Bounded};

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct LiquidityPool<T: Config> {
	/// The identifier of the liquidity pool asset
	pub(super) id: AssetIdOf<T>,
	/// The identifiers of the asset pair
	pub(super) pair: (AssetIdOf<T>, AssetIdOf<T>),
	// The account holding liquidity added to the pool
	pub(super) account: AccountIdOf<T>,
}

impl<T: Config> LiquidityPool<T> {
	// todo: document
	pub(super) fn new(pair: (AssetIdOf<T>, AssetIdOf<T>)) -> Result<Self, DispatchError> {
		let id = Self::create(pair)?;
		let account = T::PalletId::get().into_sub_account_truncating(id);
		Ok(Self { id, pair, account })
	}

	// Create asset for liquidity pool token
	// todo: document
	fn create(pair: (AssetIdOf<T>, AssetIdOf<T>)) -> Result<AssetIdOf<T>, DispatchError> {
		// Generate asset identifier of liquidity pool token
		// NOTE: Currently storing identifiers of liquidity pool tokens at end of u32 range due to time constraints
		// todo: ideally use a hash for asset id to make this easier, but seems assets pallet has a trait bound not provided by default hash type
		let id = <LiquidityPoolTokenIdGenerator<T>>::get()
			.unwrap_or_else(|| AssetIdOf::<T>::max_value());

		// Ensure asset id not already in use
		ensure!(!T::exists(id), Error::<T>::AssetAlreadyExists);

		// Create asset
		let dex: T::AccountId = T::PalletId::get().into_account_truncating();
		T::Assets::create(id, dex.clone(), true, T::LiquidityPoolTokenMinimumBalance::get())?;

		// Set asset metadata based on existing assets
		let mut asset_0 = T::Assets::symbol(pair.0);
		let asset_1 = T::Assets::symbol(pair.1);
		asset_0.extend(asset_1);
		T::Assets::set(id, &dex, asset_0.clone(), asset_0, T::LiquidityPoolTokenDecimals::get())?;

		// Set next value to be used
		// <LiquidityPoolTokenIdGenerator<T>>::set(Some(id - 1u32.into()));

		Ok(id)
	}

	// todo: document
	pub(super) fn add(
		&self,
		amount: (BalanceOf<T>, BalanceOf<T>),
		liquidity_provider: &AccountIdOf<T>,
	) -> Result<PriceOf<T>, sp_runtime::DispatchError> {
		// Simplified version of https://github.com/Uniswap/v1-contracts/blob/c10c08d81d6114f694baa8bd32f555a40f6264da/contracts/uniswap_exchange.vy#L48
		let total_issuance = T::Assets::total_issuance(self.id);
		let minted = if total_issuance == <BalanceOf<T>>::default() {
			// Use supplied amounts to initialise pool
			T::Assets::mint_into(self.id, liquidity_provider, amount.0)?;
			T::Assets::teleport(self.pair.0, liquidity_provider, &self.account, amount.0)?;
			T::Assets::teleport(self.pair.1, liquidity_provider, &self.account, amount.1)?;
		} else {
			// Determine current balances of each asset held within liquidity pool
			let balances = (
				T::Assets::balance(self.pair.0, &self.account),
				T::Assets::balance(self.pair.1, &self.account),
			);

			// Calculate amount of second token based on existing ratio
			let amount_1 = amount.0 * balances.1 / balances.0;
			let liquidity_minted = amount.0 * total_issuance / balances.0;
			// Transfer the assets from the liquidity provider to the pool and then mint their corresponding LP tokens
			T::Assets::mint_into(self.id, liquidity_provider, liquidity_minted)?;
			T::Assets::teleport(self.pair.0, liquidity_provider, &self.account, amount.0)?;
			T::Assets::teleport(self.pair.1, liquidity_provider, &self.account, amount_1)?;
		};

		// Finally return updated price
		let balance_0 = T::Assets::balance(self.pair.0, &self.account);
		let balance_1 = T::Assets::balance(self.pair.1, &self.account);
		Ok(balance_0 * balance_1)
	}

	pub(super) fn remove(
		&self,
		amount: BalanceOf<T>,
		liquidity_provider: &AccountIdOf<T>,
	) -> Result<((BalanceOf<T>, BalanceOf<T>), PriceOf<T>), DispatchError> {
		// Get the total number of liquidity pool tokens
		let total_issuance = T::Assets::total_issuance(self.id);

		// Determine current balances of each asset held within liquidity pool
		let balance_0 = T::Assets::balance(self.pair.0, &self.account);
		let balance_1 = T::Assets::balance(self.pair.1, &self.account);

		// Calculate the amount of each asset to be withdrawn
		let withdrawn_0 = balance_0 * (amount / total_issuance);
		let withdrawn_1 = balance_1 * (amount / total_issuance);

		// Transfer the assets from liquidity pool account back to liquidity provider
		T::Assets::teleport(self.pair.0, &self.account, liquidity_provider, withdrawn_0)?;
		T::Assets::teleport(self.pair.1, &self.account, liquidity_provider, withdrawn_1)?;

		// Finally return updated price based on new balances
		let balance_0 = T::Assets::balance(self.pair.0, &self.account);
		let balance_1 = T::Assets::balance(self.pair.1, &self.account);
		Ok(((withdrawn_0, withdrawn_1), balance_0 * balance_1))
	}

	pub(super) fn swap(
		&self,
		amount: (BalanceOf<T>, AssetIdOf<T>),
		who: &AccountIdOf<T>,
	) -> Result<PriceOf<T>, sp_runtime::DispatchError> {
		// Based on https://docs.uniswap.org/protocol/V1/guides/trade-tokens
		if amount.1 == self.pair.0 {
			// Sell TOKEN_0 for TOKEN_1
			let input_amount = amount.0;
			let input_reserve = T::Assets::balance(self.pair.0, &self.account);
			let output_reserve = T::Assets::balance(self.pair.1, &self.account);

			// Output amount bought
			let numerator = input_amount * output_reserve * T::SwapFeeValue::get();
			let denominator =
				input_reserve * T::SwapFeeUnits::get() + input_amount * T::SwapFeeValue::get();
			let output_amount = numerator / denominator;

			// Transfer assets
			T::Assets::teleport(self.pair.0, &who, &self.account, input_amount)?;
			T::Assets::teleport(self.pair.1, &self.account, who, output_amount)?;
		} else {
			// Sell TOKEN_1 for TOKEN_0
			let input_amount = amount.0;
			let input_reserve = T::Assets::balance(self.pair.1, &self.account);
			let output_reserve = T::Assets::balance(self.pair.0, &self.account);

			// Output amount bought
			let numerator = input_amount * output_reserve * T::SwapFeeValue::get();
			let denominator =
				input_reserve * T::SwapFeeUnits::get() + input_amount * T::SwapFeeValue::get();
			let output_amount = numerator / denominator;

			// Transfer assets
			T::Assets::teleport(self.pair.0, &self.account, who, output_amount)?;
			T::Assets::teleport(self.pair.1, &who, &self.account, input_amount)?;
		}

		// Finally return updated price
		let balance_0 = T::Assets::balance(self.pair.0, &self.account);
		let balance_1 = T::Assets::balance(self.pair.1, &self.account);
		Ok(balance_0 * balance_1)
	}
}

pub(super) struct Pair<T: Config>(PhantomData<T>);

impl<T: Config> Pair<T> {
	// todo: document
	pub(super) fn from(
		asset_0: AssetIdOf<T>,
		asset_1: AssetIdOf<T>,
	) -> (AssetIdOf<T>, AssetIdOf<T>) {
		// Sort by asset id so always in same order
		if asset_1 < asset_0 {
			(asset_1, asset_0)
		} else {
			(asset_0, asset_1)
		}
	}

	// todo: document
	pub(super) fn from_values(
		value_0: BalanceOf<T>,
		asset_0: AssetIdOf<T>,
		value_1: BalanceOf<T>,
		asset_1: AssetIdOf<T>,
	) -> (Value<T>, Value<T>) {
		let value_0 = Value { value: value_0, asset: asset_0 };
		let value_1 = Value { value: value_1, asset: asset_1 };
		// Sort by asset id so always in same order
		if value_1.asset < value_0.asset {
			(value_1, value_0)
		} else {
			(value_0, value_1)
		}
	}
}

pub(super) struct Value<T: Config> {
	pub(super) value: BalanceOf<T>,
	pub(super) asset: AssetIdOf<T>,
}
#[cfg(test)]
mod tests {
	use crate::LiquidityPool;
	use crate::{mock::*, Error, LiquidityPools, Price};
	use frame_support::assert_ok;
	use frame_support::traits::fungibles::Inspect;

	#[test]
	fn new_liquidity_pool() {
		new_test_ext().execute_with(|| {
			let pool = <LiquidityPool<Test>>::new((1, 2)).unwrap();
			assert_eq!(pool.id, u32::MAX);
			assert_eq!(pool.pair, (1, 2));
			assert_ne!(pool.account, 0);
		});
	}

	const ADMIN: u64 = 1;
	const ASSET_0: u32 = 1;
	const ASSET_1: u32 = 2;
	const BUYER: u64 = 12312;
	const DECIMALS: u128 = 10;
	const LIQUIDITY_PROVIDER: u64 = 123;
	const MIN_BALANCE: u128 = 1;

	#[test]
	fn adds_liquidity() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::mint(
				Origin::signed(ADMIN),
				ASSET_0,
				LIQUIDITY_PROVIDER,
				1000 * DECIMALS
			));
			assert_ok!(Assets::mint(
				Origin::signed(ADMIN),
				ASSET_1,
				LIQUIDITY_PROVIDER,
				1000 * DECIMALS
			));

			let pool = <LiquidityPool<Test>>::new((ASSET_0, ASSET_1)).unwrap();
			assert_eq!(
				pool.add((10 * DECIMALS, 500 * DECIMALS), &LIQUIDITY_PROVIDER).unwrap(),
				(10 * DECIMALS) * (500 * DECIMALS)
			);

			// Check pool balances
			assert_eq!(Assets::balance(ASSET_0, pool.account), 10 * DECIMALS);
			assert_eq!(Assets::balance(ASSET_1, pool.account), 500 * DECIMALS);
			assert_eq!(Assets::total_issuance(pool.id), 10 * DECIMALS);

			// Check liquidity provider balances
			assert_eq!(Assets::balance(ASSET_0, &LIQUIDITY_PROVIDER), 990 * DECIMALS);
			assert_eq!(Assets::balance(ASSET_1, &LIQUIDITY_PROVIDER), 500 * DECIMALS);
			assert_eq!(Assets::balance(pool.id, &LIQUIDITY_PROVIDER), 10 * DECIMALS);
		});
	}

	#[test]
	fn swaps() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::mint(
				Origin::signed(ADMIN),
				ASSET_0,
				LIQUIDITY_PROVIDER,
				1000 * DECIMALS
			));
			assert_ok!(Assets::mint(
				Origin::signed(ADMIN),
				ASSET_1,
				LIQUIDITY_PROVIDER,
				1000 * DECIMALS
			));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, BUYER, 10 * DECIMALS));

			let pool = <LiquidityPool<Test>>::new((1, 2)).unwrap();
			pool.add((10 * DECIMALS, 500 * DECIMALS), &LIQUIDITY_PROVIDER).unwrap();

			let price = pool.swap((5 * DECIMALS, ASSET_1), &BUYER).unwrap();

			// Check pool balances
			assert_eq!(Assets::balance(ASSET_0, pool.account), 10 * DECIMALS);
			assert_eq!(Assets::balance(ASSET_1, pool.account), 550 * DECIMALS);
			assert_eq!(Assets::total_issuance(pool.id), 10 * DECIMALS);

			// Check buyer balances
			assert_eq!(Assets::balance(ASSET_0, &BUYER), 990 * DECIMALS);
			assert_eq!(Assets::balance(ASSET_1, &BUYER), 5 * DECIMALS);
			assert_eq!(Assets::balance(pool.id, pool.account), 10 * DECIMALS);
		});
	}
}
