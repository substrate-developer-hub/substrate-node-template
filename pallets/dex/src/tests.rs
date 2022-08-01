use crate::{mock::*, Error, LiquidityPools, Price};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;
use std::time::{SystemTime, UNIX_EPOCH};

const ADMIN: u64 = 1;
const ASSET_0: u32 = 1;
const ASSET_1: u32 = 2;
const DEADLINE: u64 = u64::MAX;
const INVALID_ASSET: u32 = 21762531;
const LIQUIDITY_PROVIDER: u64 = 123;
const MIN_BALANCE: u128 = 1;

#[test]
fn add_liquidity_ensures_signed() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::add_liquidity(Origin::none(), 0, ASSET_0, 0, ASSET_1, DEADLINE),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn add_liquidity_ensure_assets_unique() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				0,
				ASSET_0,
				0,
				ASSET_0,
				DEADLINE
			),
			Error::<Test>::IdenticalAssets
		);
	});
}

#[test]
fn add_liquidity_ensure_amount_0_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				0,
				ASSET_0,
				10,
				ASSET_1,
				DEADLINE
			),
			Error::<Test>::InvalidAmount
		);
	});
}

#[test]
fn add_liquidity_ensure_amount_1_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				1,
				ASSET_0,
				0,
				ASSET_1,
				DEADLINE
			),
			Error::<Test>::InvalidAmount
		);
	});
}

#[test]
fn add_liquidity_ensure_asset_0_valid() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				10,
				INVALID_ASSET,
				20,
				ASSET_0,
				DEADLINE
			),
			Error::<Test>::InvalidAsset
		);
	});
}

#[test]
fn add_liquidity_ensure_asset_1_valid() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				10,
				ASSET_0,
				20,
				INVALID_ASSET,
				DEADLINE
			),
			Error::<Test>::InvalidAsset
		);
	});
}

#[test]
fn add_liquidity_ensure_asset_0_balance_sufficient() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				10,
				ASSET_1,
				20,
				ASSET_0,
				DEADLINE
			),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn add_liquidity_ensure_asset_1_balance_sufficient() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LIQUIDITY_PROVIDER, 100));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				10,
				ASSET_0,
				20,
				ASSET_1,
				DEADLINE
			),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn add_liquidity_ensure_within_deadline() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LIQUIDITY_PROVIDER, 100));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LIQUIDITY_PROVIDER, 100));

		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
		Timestamp::set_timestamp(now);

		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				10,
				ASSET_0,
				20,
				ASSET_1,
				now - 10
			),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn add_liquidity_ensure_liquidity_pool_id() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LIQUIDITY_PROVIDER, 100));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LIQUIDITY_PROVIDER, 100));

		// Create liquidity pool asset in advance
		assert_ok!(Assets::create(Origin::signed(1), u32::MAX, ADMIN, MIN_BALANCE));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LIQUIDITY_PROVIDER),
				10,
				ASSET_0,
				20,
				ASSET_1,
				DEADLINE
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn adds_liquidity() {
	new_test_ext().execute_with(|| {
		// Create assets and fund
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LIQUIDITY_PROVIDER, 100));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LIQUIDITY_PROVIDER, 100));

		// Add liquidity to pool
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LIQUIDITY_PROVIDER),
			10,
			ASSET_1,
			20,
			ASSET_0, // Intentionally placed lower id in second position to test ordering
			DEADLINE
		));

		// Ensure liquidity pool (and token) token created as asset
		let pool = LiquidityPools::<Test>::get((ASSET_0, ASSET_1)).unwrap();
		assert!(Assets::maybe_total_supply(pool.id).is_some());

		// Check resulting balances and price
		assert_eq!(Assets::balance(ASSET_0, &LIQUIDITY_PROVIDER), 80);
		assert_eq!(Assets::balance(ASSET_1, &LIQUIDITY_PROVIDER), 90);
		assert_eq!(Assets::balance(pool.id, &LIQUIDITY_PROVIDER), 20);
		assert_eq!(Price::<Test>::get((ASSET_0, ASSET_1)).unwrap(), 10 * 20);
	});
}
