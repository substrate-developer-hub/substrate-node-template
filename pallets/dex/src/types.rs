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
	) -> Result<PriceOf<T>, DispatchError> {
		// Simplified version of https://github.com/Uniswap/v1-contracts/blob/c10c08d81d6114f694baa8bd32f555a40f6264da/contracts/uniswap_exchange.vy#L48
		let total_issuance = T::Assets::total_issuance(self.id);
		if total_issuance == <BalanceOf<T>>::default() {
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
	) -> Result<PriceOf<T>, DispatchError> {
		// Simplified version of https://github.com/Uniswap/v1-contracts/blob/master/contracts/uniswap_exchange.vy#L83

		// Get the total number of liquidity pool tokens
		ensure!(amount > <BalanceOf<T>>::default(), Error::<T>::InvalidAmount);
		let total_issuance = T::Assets::total_issuance(self.id);
		ensure!(amount > <BalanceOf<T>>::default(), Error::<T>::EmptyPool);

		// Determine current balances of each asset held within liquidity pool
		let balances = (
			T::Assets::balance(self.pair.0, &self.account),
			T::Assets::balance(self.pair.1, &self.account),
		);

		// Calculate the amount of each asset to be withdrawn
		let amount_0 = amount * balances.0 / total_issuance;
		let amount_1 = amount * balances.1 / total_issuance;

		// Transfer the assets from liquidity pool account back to liquidity provider and then burn LP tokens
		T::Assets::teleport(self.pair.0, &self.account, liquidity_provider, amount_0)?;
		T::Assets::teleport(self.pair.1, &self.account, liquidity_provider, amount_1)?;
		T::Assets::burn_from(self.id, &liquidity_provider, amount)?;

		// Finally return updated price based on new balances
		let balance_0 = T::Assets::balance(self.pair.0, &self.account);
		let balance_1 = T::Assets::balance(self.pair.1, &self.account);
		Ok(balance_0 * balance_1)
	}

	pub(super) fn swap(
		&self,
		amount: (BalanceOf<T>, AssetIdOf<T>),
		who: &AccountIdOf<T>,
	) -> Result<PriceOf<T>, DispatchError> {
		let swap_fee_value = T::SwapFeeValue::get();
		let swap_fee_units = T::SwapFeeUnits::get();

		// Based on https://docs.uniswap.org/protocol/V1/guides/trade-tokens
		let input_amount = amount.0;
		if amount.1 == self.pair.0 {
			// Sell TOKEN_0 for TOKEN_1
			let input_reserve = T::Assets::balance(self.pair.0, &self.account) - input_amount;
			let output_reserve = T::Assets::balance(self.pair.1, &self.account);

			// Output amount bought
			let input_amount_with_fee = input_amount * swap_fee_value;
			let numerator = input_amount_with_fee * output_reserve;
			let denominator = (input_reserve * swap_fee_units) + input_amount_with_fee;
			let output_amount = numerator / denominator;

			// Transfer assets
			T::Assets::teleport(self.pair.0, &who, &self.account, input_amount)?;
			T::Assets::teleport(self.pair.1, &self.account, who, output_amount)?;
		} else {
			// Sell TOKEN_1 for TOKEN_0
			let input_reserve = T::Assets::balance(self.pair.1, &self.account) - input_amount;
			let output_reserve = T::Assets::balance(self.pair.0, &self.account);

			// Output amount bought
			let input_amount_with_fee = input_amount * swap_fee_value;
			let numerator = input_amount_with_fee * output_reserve;
			let denominator = (input_reserve * swap_fee_units) + input_amount_with_fee;
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
	use crate::mock::*;
	use crate::LiquidityPool;
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
	const UNITS: u128 = 10;
	const LP: u64 = 123;
	const MIN_BALANCE: u128 = 1;

	#[test]
	fn adds_liquidity() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 1000 * UNITS));

			let pool = <LiquidityPool<Test>>::new((ASSET_0, ASSET_1)).unwrap();
			assert_eq!(
				pool.add((10 * UNITS, 500 * UNITS), &LP).unwrap(),
				(10 * UNITS) * (500 * UNITS)
			);

			// Check pool balances
			assert_eq!(Assets::balance(ASSET_0, pool.account), 10 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, pool.account), 500 * UNITS);
			assert_eq!(Assets::total_issuance(pool.id), 10 * UNITS);

			// Check liquidity provider balances
			assert_eq!(Assets::balance(ASSET_0, &LP), 990 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, &LP), 500 * UNITS);
			assert_eq!(Assets::balance(pool.id, &LP), 10 * UNITS);
		});
	}

	#[test]
	fn removes_all_liquidity() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 1000 * UNITS));

			let pool = <LiquidityPool<Test>>::new((ASSET_0, ASSET_1)).unwrap();
			assert_ok!(pool.add((10 * UNITS, 500 * UNITS), &LP));
			let lp_tokens = Assets::balance(pool.id, &LP);
			assert_eq!(lp_tokens, (10 * UNITS));

			let result = pool.remove(lp_tokens, &LP).unwrap();

			// Check pool balances
			assert_eq!(Assets::balance(ASSET_0, pool.account), 0);
			assert_eq!(Assets::balance(ASSET_1, pool.account), 0);
			assert_eq!(Assets::total_issuance(pool.id), 0);

			// Check liquidity provider balances (back to original)
			assert_eq!(Assets::balance(ASSET_0, &LP), 1000 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, &LP), 1000 * UNITS);
			assert_eq!(Assets::balance(pool.id, &LP), 0);
		});
	}

	#[test]
	fn swaps_asset_0() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, BUYER, 100 * UNITS));

			let pool = <LiquidityPool<Test>>::new((1, 2)).unwrap();
			pool.add((10 * UNITS, 500 * UNITS), &LP).unwrap();

			assert_ok!(pool.swap((5 * UNITS, ASSET_0), &BUYER));

			// Check buyer balances
			assert_eq!(Assets::balance(ASSET_0, &BUYER), (100 - 5) * UNITS);
			assert_eq!(Assets::balance(ASSET_1, &BUYER), 2496);

			// Check pool balances
			assert_eq!(Assets::balance(ASSET_0, pool.account), 15 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, pool.account), (500 * UNITS) - 2496);
			assert_eq!(Assets::total_issuance(pool.id), 10 * UNITS);
		});
	}

	#[test]
	fn swaps_asset_1() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, BUYER, 500 * UNITS));

			let pool = <LiquidityPool<Test>>::new((1, 2)).unwrap();
			pool.add((10 * UNITS, 500 * UNITS), &LP).unwrap();

			assert_ok!(pool.swap((250 * UNITS, ASSET_1), &BUYER));

			// Check buyer balances
			assert_eq!(Assets::balance(ASSET_0, &BUYER), 49);
			assert_eq!(Assets::balance(ASSET_1, &BUYER), (500 - 250) * UNITS);

			// Check pool balances
			assert_eq!(Assets::balance(ASSET_0, pool.account), 51);
			assert_eq!(Assets::balance(ASSET_1, pool.account), 750 * UNITS);
		});
	}
}
