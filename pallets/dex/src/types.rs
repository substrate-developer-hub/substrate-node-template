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
		<LiquidityPoolTokenIdGenerator<T>>::set(Some(id - 1u32.into()));

		Ok(id)
	}

	// todo: document
	// Simplified version of https://github.com/Uniswap/v1-contracts/blob/c10c08d81d6114f694baa8bd32f555a40f6264da/contracts/uniswap_exchange.vy#L48
	pub(super) fn add(
		&self,
		amount: (BalanceOf<T>, BalanceOf<T>),
		liquidity_provider: &AccountIdOf<T>,
	) -> DispatchResult {
		let total_issuance = T::Assets::total_issuance(self.id);
		if total_issuance == <BalanceOf<T>>::default() {
			// Use supplied amounts to initialise pool
			T::Assets::mint_into(self.id, liquidity_provider, amount.0)?;
			<Pallet<T>>::transfer(self.pair.0, liquidity_provider, &self.account, amount.0)?;
			<Pallet<T>>::transfer(self.pair.1, liquidity_provider, &self.account, amount.1)?;
		} else {
			// Determine current balances of each asset held within liquidity pool
			let balances = (
				<Pallet<T>>::balance(self.pair.0, &self.account),
				<Pallet<T>>::balance(self.pair.1, &self.account),
			);

			// Calculate amount of second token based on existing ratio
			let amount_1 = amount.0 * balances.1 / balances.0;
			let liquidity_minted = amount.0 * total_issuance / balances.0;
			// Transfer the assets from the liquidity provider to the pool and then mint their corresponding LP tokens
			T::Assets::mint_into(self.id, liquidity_provider, liquidity_minted)?;
			<Pallet<T>>::transfer(self.pair.0, liquidity_provider, &self.account, amount.0)?;
			<Pallet<T>>::transfer(self.pair.1, liquidity_provider, &self.account, amount_1)?;
		};

		Ok(())
	}

	/// Calculates the output amount of an asset, given an input amount and the amount in each reserve.   
	/// # Arguments
	/// * `input_amount` - An amount to be traded.
	/// * `input_reserve` - The reserve amount held of the input asset.
	/// * `output_reserve` - The reserve amount held of the output asset.
	fn calculate(
		input_amount: BalanceOf<T>,
		input_reserve: BalanceOf<T>,
		output_reserve: BalanceOf<T>,
	) -> BalanceOf<T> {
		// Ensure value inputs
		if input_reserve == <BalanceOf<T>>::default() || output_reserve == <BalanceOf<T>>::default()
		{
			return <BalanceOf<T>>::default();
		}

		// Subtract fee from input amount, so liquidity providers can accrue rewards
		let input_amount_with_fee = input_amount * T::SwapFeeValue::get();
		let numerator = input_amount_with_fee * output_reserve;
		let denominator = (input_reserve * T::SwapFeeUnits::get()) + input_amount_with_fee;
		numerator / denominator
	}

	fn calculate_price(&self, amount: (BalanceOf<T>, AssetIdOf<T>)) -> BalanceOf<T> {
		// Determine other asset from pair
		let (input_amount, asset) = amount;
		let other = if asset == self.pair.0 { self.pair.1 } else { self.pair.0 };

		// Get balances of each asset in the pool and calculate the price
		let input_reserve = <Pallet<T>>::balance(asset, &self.account);
		let output_reserve = <Pallet<T>>::balance(other, &self.account);
		Self::calculate(input_amount, input_reserve, output_reserve)
	}

	/// Calculates the output amount of asset `other`, given an input `amount` and asset.   
	/// # Arguments
	/// * `amount` - An amount to be valued.
	/// * `other` - The reserve amount held of the output asset.
	pub(super) fn price(
		amount: (BalanceOf<T>, AssetIdOf<T>),
		other: AssetIdOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let (amount, asset) = amount;
		ensure!(amount > <BalanceOf<T>>::default(), Error::<T>::InvalidAmount);

		// Get liquidity pool
		let pair = <Pair<T>>::from(asset, other);
		<LiquidityPools<T>>::get(pair).map_or_else(
			// Return error if not found
			|| Err(DispatchError::from(Error::<T>::NoPool)),
			// Otherwise calculate price of supplied amount
			// todo: make use of storage for pricing optimisation
			|pool| Ok(pool.calculate_price((amount, asset))),
		)
	}

	/// Removes liquidity from the pool, by specifying the `amount` of liquidity tokens.  
	/// # Arguments
	/// * `amount` - The amount of liquidity tokens to be redeemed.
	/// * `liquidity_provider` - The account identifier of the liquidity provider.
	/// **Note:** A very simple version of https://github.com/Uniswap/v1-contracts/blob/master/contracts/uniswap_exchange.vy#L83
	pub(super) fn remove(
		&self,
		amount: BalanceOf<T>,
		liquidity_provider: &AccountIdOf<T>,
	) -> DispatchResult {
		// Get the total number of liquidity pool tokens
		ensure!(amount > <BalanceOf<T>>::default(), Error::<T>::InvalidAmount);
		let total_issuance = T::Assets::total_issuance(self.id);
		ensure!(amount > <BalanceOf<T>>::default(), Error::<T>::EmptyPool);

		// Determine current balances of each asset held within liquidity pool
		let balances = (
			<Pallet<T>>::balance(self.pair.0, &self.account),
			<Pallet<T>>::balance(self.pair.1, &self.account),
		);

		// Calculate the amount of each asset to be withdrawn (which includes rewards from providing liquidity)
		let amount_0 = amount * balances.0 / total_issuance;
		let amount_1 = amount * balances.1 / total_issuance;

		// Transfer the assets from liquidity pool account back to liquidity provider and then burn LP tokens
		<Pallet<T>>::transfer(self.pair.0, &self.account, liquidity_provider, amount_0)?;
		<Pallet<T>>::transfer(self.pair.1, &self.account, liquidity_provider, amount_1)?;
		T::Assets::burn_from(self.id, &liquidity_provider, amount)?;

		Ok(())
	}

	/// Performs a swap of an amount of the specified asset type, returning the resulting amount of the corresponding
	/// asset. The other side of the pair is inferred by the pool.  
	/// # Arguments
	/// * `amount` - An amount (and asset) to be swapped.
	/// * `who` - The identifier of the account initiating the swap.
	pub(super) fn swap(
		&self,
		amount: (BalanceOf<T>, AssetIdOf<T>),
		who: &AccountIdOf<T>,
	) -> Result<(BalanceOf<T>, AssetIdOf<T>), DispatchError> {
		// todo: needs refactoring to simplify
		// Based on https://docs.uniswap.org/protocol/V1/guides/trade-tokens
		let input_amount = amount.0;
		if amount.1 == self.pair.0 {
			// Sell ASSET_0 for ASSET_1
			let input_reserve = <Pallet<T>>::balance(self.pair.0, &self.account) - input_amount;
			let output_reserve = <Pallet<T>>::balance(self.pair.1, &self.account);
			let output_amount =
				<LiquidityPool<T>>::calculate(input_amount, input_reserve, output_reserve);

			// Transfer assets
			<Pallet<T>>::transfer(self.pair.0, &who, &self.account, input_amount)?;
			<Pallet<T>>::transfer(self.pair.1, &self.account, who, output_amount)?;

			Ok((output_amount, self.pair.1))
		} else {
			// Sell ASSET_1 for ASSET_0
			let input_reserve = <Pallet<T>>::balance(self.pair.1, &self.account) - input_amount;
			let output_reserve = <Pallet<T>>::balance(self.pair.0, &self.account);
			let output_amount =
				<LiquidityPool<T>>::calculate(input_amount, input_reserve, output_reserve);

			// Transfer assets
			<Pallet<T>>::transfer(self.pair.0, &self.account, who, output_amount)?;
			<Pallet<T>>::transfer(self.pair.1, &who, &self.account, input_amount)?;

			Ok((output_amount, self.pair.0))
		}
	}
}

/// A generic 'pair'.
pub(super) struct Pair<T: Config>(PhantomData<T>);

impl<T: Config> Pair<T> {
	/// Creates a pair from two asset identifiers, `asset_0` and `asset_1`.  
	/// **Note:** The supplied identifiers are sorted so the resulting pair always has the 'lowest' asset identifier as
	/// the first item of the pair.
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

	/// Creates a pair from a pair of asset values.  
	/// **Note:** The supplied asset value pairs are sorted by asset identifier, so the resulting pair always has the
	/// value with the 'lowest' asset identifier as the first item of the pair.
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

/// A `value` of a particular `asset`.
pub(super) struct Value<T: Config> {
	pub(super) value: BalanceOf<T>,
	pub(super) asset: AssetIdOf<T>,
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::LiquidityPool;
	use frame_support::assert_ok;
	use frame_support::traits::fungible::Inspect as InspectFungible;
	use frame_support::traits::fungibles::Inspect as InspectFungibles;

	const ADMIN: u64 = 1;
	const NATIVE_TOKEN: u32 = 0;
	const ASSET_0: u32 = 1;
	const ASSET_1: u32 = 2;
	const BUYER: u64 = 12312;
	const UNITS: u128 = 1_000; // Minor precision for now, should ideally increase
	const LP: u64 = 123;
	const MIN_BALANCE: u128 = 1;

	#[test]
	fn new_liquidity_pool() {
		new_test_ext().execute_with(|| {
			let pool = <LiquidityPool<Test>>::new((ASSET_0, ASSET_1)).unwrap();
			assert_eq!(pool.id, u32::MAX);
			assert_eq!(pool.pair, (ASSET_0, ASSET_1));
			assert_ne!(pool.account, 0);
		});
	}

	#[test]
	fn additional_liquidity_pools_decrements_id() {
		new_test_ext().execute_with(|| {
			assert_eq!(<LiquidityPool<Test>>::new((1, 2)).unwrap().id, u32::MAX);
			assert_eq!(<LiquidityPool<Test>>::new((2, 3)).unwrap().id, u32::MAX - 1);
			assert_eq!(<LiquidityPool<Test>>::new((3, 4)).unwrap().id, u32::MAX - 2);
		});
	}

	#[test]
	fn new_liquidity_pool_with_native_currency() {
		new_test_ext().execute_with(|| {
			let pool = <LiquidityPool<Test>>::new((NATIVE_TOKEN, ASSET_1)).unwrap();
			assert_eq!(pool.id, u32::MAX);
			assert_eq!(pool.pair, (NATIVE_TOKEN, ASSET_1));
			assert_ne!(pool.account, 0);
		});
	}

	#[test]
	fn adds_liquidity() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 1000 * UNITS));

			let pool = <LiquidityPool<Test>>::new((ASSET_0, ASSET_1)).unwrap();
			assert_ok!(pool.add((10 * UNITS, 500 * UNITS), &LP));

			// Check prices
			assert_eq!(<LiquidityPool<Test>>::calculate(1 * UNITS, 10 * UNITS, 500 * UNITS), 45330);
			assert_eq!(pool.calculate_price((1 * UNITS, ASSET_0)), 45330);
			assert_eq!(pool.calculate_price((50 * UNITS, ASSET_1)), 906);

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
	fn adds_liquidity_with_native_currency() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(
				Origin::root(),
				NATIVE_TOKEN,
				ADMIN,
				true,
				MIN_BALANCE
			));
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Balances::transfer(Origin::signed(ADMIN), LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 1000 * UNITS));

			let pool = <LiquidityPool<Test>>::new((NATIVE_TOKEN, ASSET_1)).unwrap();
			assert_ok!(pool.add((10 * UNITS, 500 * UNITS), &LP));

			// Check price
			assert_eq!(<LiquidityPool<Test>>::calculate(1 * UNITS, 10 * UNITS, 500 * UNITS), 45330);
			assert_eq!(pool.calculate_price((1 * UNITS, NATIVE_TOKEN)), 45330);
			assert_eq!(pool.calculate_price((50 * UNITS, ASSET_1)), 906);

			// Check pool balances
			assert_eq!(Balances::balance(&pool.account), 10 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, pool.account), 500 * UNITS);
			assert_eq!(Assets::total_issuance(pool.id), 10 * UNITS);

			// Check liquidity provider balances
			assert_eq!(Balances::balance(&LP), 990 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, &LP), 500 * UNITS);
			assert_eq!(Assets::balance(pool.id, &LP), 10 * UNITS);
		});
	}

	#[test]
	fn calculates_zero_input_amount() {
		assert_eq!(<LiquidityPool<Test>>::calculate(0, 321, 12345), 0);
	}

	#[test]
	fn calculates_zero_input_reserve() {
		assert_eq!(<LiquidityPool<Test>>::calculate(10, 0, 100), 0);
	}

	#[test]
	fn calculates_zero_output_reserve() {
		assert_eq!(<LiquidityPool<Test>>::calculate(10, 100, 0), 0);
	}

	#[test]
	fn calculates() {
		// https://hackmd.io/@HaydenAdams/HJ9jLsfTz?type=view#Example-ETH-%E2%86%92-OMG
		// Result not exactly the same as test mock sets 0.3% fee rather than 0.25% in example
		assert_eq!(<LiquidityPool<Test>>::calculate(1 * UNITS, 10 * UNITS, 500 * UNITS), 45330);
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

			assert_ok!(pool.remove(lp_tokens, &LP));

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
	fn removes_all_liquidity_with_native_currency() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Balances::transfer(Origin::signed(ADMIN), LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 1000 * UNITS));

			let pool = <LiquidityPool<Test>>::new((NATIVE_TOKEN, ASSET_1)).unwrap();
			assert_ok!(pool.add((10 * UNITS, 500 * UNITS), &LP));
			let lp_tokens = Assets::balance(pool.id, &LP);
			assert_eq!(lp_tokens, (10 * UNITS));

			assert_ok!(pool.remove(lp_tokens, &LP));

			// Check pool balances
			assert_eq!(Balances::balance(&pool.account), 0);
			assert_eq!(Assets::balance(ASSET_1, pool.account), 0);
			assert_eq!(Assets::total_issuance(pool.id), 0);

			// Check liquidity provider balances (back to original)
			assert_eq!(Balances::balance(&LP), 1000 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, &LP), 1000 * UNITS);
			assert_eq!(Assets::balance(pool.id, &LP), 0);
		});
	}

	#[test]
	fn swaps_with_native_currency() {
		new_test_ext().execute_with(|| {
			assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
			assert_ok!(Balances::transfer(Origin::signed(ADMIN), LP, 1000 * UNITS));
			assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 1000 * UNITS));
			assert_ok!(Balances::transfer(Origin::signed(ADMIN), BUYER, 100 * UNITS));

			let pool = <LiquidityPool<Test>>::new((NATIVE_TOKEN, ASSET_1)).unwrap();
			pool.add((10 * UNITS, 500 * UNITS), &LP).unwrap();

			let output = pool.swap((5 * UNITS, NATIVE_TOKEN), &BUYER).unwrap();
			assert_eq!(output.0, 249624);
			assert_eq!(output.1, ASSET_1);

			// Check buyer balances
			assert_eq!(Balances::balance(&BUYER), (100 - 5) * UNITS);
			assert_eq!(Assets::balance(ASSET_1, &BUYER), 249624);

			// Check pool balances
			assert_eq!(Balances::balance(&pool.account), 15 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, pool.account), (500 * UNITS) - 249624);
			assert_eq!(Assets::total_issuance(pool.id), 10 * UNITS);
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

			let pool = <LiquidityPool<Test>>::new((ASSET_0, ASSET_1)).unwrap();
			pool.add((10 * UNITS, 500 * UNITS), &LP).unwrap();

			let output = pool.swap((5 * UNITS, ASSET_0), &BUYER).unwrap();
			assert_eq!(output.0, 249624);
			assert_eq!(output.1, ASSET_1);

			// Check buyer balances
			assert_eq!(Assets::balance(ASSET_0, &BUYER), (100 - 5) * UNITS);
			assert_eq!(Assets::balance(ASSET_1, &BUYER), 249624);

			// Check pool balances
			assert_eq!(Assets::balance(ASSET_0, pool.account), 15 * UNITS);
			assert_eq!(Assets::balance(ASSET_1, pool.account), (500 * UNITS) - 249624);
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

			let pool = <LiquidityPool<Test>>::new((ASSET_0, ASSET_1)).unwrap();
			pool.add((10 * UNITS, 500 * UNITS), &LP).unwrap();

			let output = pool.swap((250 * UNITS, ASSET_1), &BUYER).unwrap();
			assert_eq!(output.0, 4992);
			assert_eq!(output.1, ASSET_0);

			// Check buyer balances
			assert_eq!(Assets::balance(ASSET_0, &BUYER), 4992);
			assert_eq!(Assets::balance(ASSET_1, &BUYER), (500 - 250) * UNITS);

			// Check pool balances
			assert_eq!(Assets::balance(ASSET_0, pool.account), 5008);
			assert_eq!(Assets::balance(ASSET_1, pool.account), 750 * UNITS);
		});
	}
}
