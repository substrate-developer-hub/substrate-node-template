#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::types::{
	U32F32
};
use frame_support::{debug, decl_module, decl_storage, decl_event, decl_error, dispatch,
	dispatch::{
		DispatchError,
		DispatchResult,
	},
	traits::{
		Currency,
		Get
	},
};
use frame_system::ensure_signed;
use core::convert::TryInto;
use module_primitives::{
	constants::time::MILLISECS_PER_BLOCK,
};

// #[cfg(test)]
// mod tests;

pub trait Config: frame_system::Config + pallet_balances::Config + pallet_timestamp::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Config> as CalculatorModule {
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items

		/// Substrate-fixed, value starts at 0 (additive identity)
		pub TotalRewardsPerDay get(fn total_rewards_daily):
			map hasher(opaque_blake2_256) u64 => Option<U32F32>;

		/// Returns reward data that has been distributed for a given day
		pub RewardsPerDay get(fn rewards_daily):
			map hasher(opaque_blake2_256) u64 =>
				// TODO - create and define generic struct instead of tuple to represent this reward data
				Option<Vec<(
					<T as frame_system::Config>::AccountId,
					U32F32, // instead of BalanceOf<T>,
					<T as frame_system::Config>::BlockNumber,
				)>>;

		/// Returns daily reward distribution block number corresponding to a given date/time
		/// This is to simplify querying storage.
		pub BlockRewardedForDay get(fn block_rewarded_for_day):
			map hasher(opaque_blake2_256) u64 =>
				Option<<T as frame_system::Config>::BlockNumber>;

		/// Returns date/time corresponding to a given daily reward distribution block number
		/// This is to simplify querying storage.
		pub DayRewardedForBlock get(fn day_rewarded_for_block):
			map hasher(opaque_blake2_256) <T as frame_system::Config>::BlockNumber =>
				Option<u64>;
	}
}

// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where
		AccountId = <T as frame_system::Config>::AccountId,
		// BalanceOf = BalanceOf<T>,
		<T as frame_system::Config>::BlockNumber,
	{
		/// Substrate-fixed total rewards for a given day has been updated.
		TotalRewardsPerDayUpdated(U32F32, u64, BlockNumber, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		NoneValue,
		/// Some math operation overflowed
		Overflow,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		fn update_fixed_daily_rewards(origin, new_reward: U32F32) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// get the current block & current date/time
			let current_block = <frame_system::Module<T>>::block_number();
			let timestamp_current = <pallet_timestamp::Module<T>>::get();

			// convert the current date/time to the start of the current day date/time.
			// i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
			let milliseconds_per_day = 86400000u64;

			let timestamp_current_as_u64;
			if let Some(_timestamp_current_as_u64) = TryInto::<u64>::try_into(timestamp_current).ok() {
				timestamp_current_as_u64 = _timestamp_current_as_u64;
			} else {
				return Err(DispatchError::Other("Unable to convert Moment to u64 for timestamp_current"));
			}

			let days_since_unix_epoch = U32F32::from_num(timestamp_current_as_u64 / milliseconds_per_day);
			// FIXME - remove hard-coded data when get a value
			// assert_eq!(days_since_unix_epoch.to_string(), "10.123");
			// round down to nearest integer days (by rounding up then subtracting one)
			let days_since_unix_epoch_ceil = days_since_unix_epoch.ceil().to_num::<u64>();
			// FIXME - remove hard-coded data
			// assert_eq!(days_since_unix_epoch_ceil, 10);
			let days_since_unix_epoch_round_down = days_since_unix_epoch_ceil - 1u64;
			// convert that value in days back to a timestamp value in milliseconds to
			// correspond to the start of the day
			let milliseconds_since_epoch_at_day_start = days_since_unix_epoch_round_down * milliseconds_per_day;

			// convert the current block number to the block number at the start of the current day.
			assert!(milliseconds_since_epoch_at_day_start % MILLISECS_PER_BLOCK == 0);
			let block_at_day_start: u64 = milliseconds_since_epoch_at_day_start / MILLISECS_PER_BLOCK;
			let block_at_day_start_as_blocknumber;
			if let Some(_block_at_day_start_as_blocknumber) =
				TryInto::<<T as frame_system::Config>::BlockNumber>::try_into(
					block_at_day_start
				).ok() {

				block_at_day_start_as_blocknumber = _block_at_day_start_as_blocknumber;
			} else {
				return Err(DispatchError::Other("Unable to convert u64 to BlockNumber"));
			}

			// check if the start of the current day date/time entry exists as a key for `rewards_daily`
			//
			// if so, retrieve the latest `rewards_daily` data stored for the start of that day date/time
			// i.e. (account_id, balance_rewarded, block_number), and add the new reward value to it.
			//
			// else just insert that as a new entry

			let new_reward_item = (
				sender.clone(),
				new_reward.clone(),
				block_at_day_start_as_blocknumber.clone(),
			);
			let mut new_reward_vec;

			match Self::rewards_daily(milliseconds_since_epoch_at_day_start.clone()) {
				None => {
					debug::info!("Creating new reward in storage vector");

					new_reward_vec = Vec::new();
					new_reward_vec.push(new_reward_item.clone());

					<RewardsPerDay<T>>::insert(
						milliseconds_since_epoch_at_day_start.clone().into(),
						&new_reward_vec,
					);
				},
				Some(_) => {
					debug::info!("Appending new rewards_per_day to existing storage vector");

					<RewardsPerDay<T>>::mutate(
						milliseconds_since_epoch_at_day_start.clone().into(),
						|ms_since_epoch_at_day_start| {
							if let Some(_ms_since_epoch_at_day_start) = ms_since_epoch_at_day_start {
								_ms_since_epoch_at_day_start.push(new_reward_item.clone());
							}
						},
					);
				},
			}

			// check if the start of the current day date/time entry exists as a key for `block_rewarded_for_day`,
			// otherwise add it, with the block number corresponding to the start of the current day as the value
			match Self::block_rewarded_for_day(milliseconds_since_epoch_at_day_start.clone()) {
				None => {
					debug::info!("Creating new mapping from timestamp at start of day to block number at start of day");

					<BlockRewardedForDay<T>>::insert(
						milliseconds_since_epoch_at_day_start.clone(),
						block_at_day_start_as_blocknumber.clone(),
					);
				},
				Some(_) => {
					debug::info!("BlockRewardedForDay entry mapping already exists. No further action required");
				}
			}

			// repeat for `day_rewarded_for_block`
			match Self::day_rewarded_for_block(block_at_day_start_as_blocknumber.clone()) {
				None => {
					debug::info!("Creating new mapping from block number at start of day to timestamp at start of day");

					<DayRewardedForBlock<T>>::insert(
						block_at_day_start_as_blocknumber.clone(),
						milliseconds_since_epoch_at_day_start.clone(),
					);
				},
				Some(old) => {
					debug::info!("DayRewardedForBlock entry mapping already exists. No further action required");
				}
			}

			// Update in storage the total rewards distributed so far for the current day
			// so users may query state and have the latest calculated total returned.
			match Self::total_rewards_daily(milliseconds_since_epoch_at_day_start.clone()) {
				None => {
					debug::info!("Creating new total rewards entry for a given day");

					<TotalRewardsPerDay>::insert(
						milliseconds_since_epoch_at_day_start.clone(),
						new_reward.clone(),
					);

					// Emit event
					Self::deposit_event(Event::TotalRewardsPerDayUpdated(
						new_reward.clone(),
						milliseconds_since_epoch_at_day_start.clone(),
						block_at_day_start_as_blocknumber.clone(),
						sender.clone(),
					));
				},
				Some(old_total_rewards_for_day) => {
					debug::info!("TotalRewardsPerDay entry mapping already exists for given day. Updating...");

					// Add, handling overflow
					let new_total_rewards_for_day =
						old_total_rewards_for_day.checked_add(new_reward.clone()).ok_or(Error::<T>::Overflow)?;
					// Write the new value to storage
					<TotalRewardsPerDay>::mutate(
						milliseconds_since_epoch_at_day_start.clone(),
						|ms_since_epoch_at_day_start| {
							if let Some(_ms_since_epoch_at_day_start) = ms_since_epoch_at_day_start {
								*_ms_since_epoch_at_day_start = new_total_rewards_for_day.clone();
							}
						},
					);

					// Emit event
					Self::deposit_event(Event::TotalRewardsPerDayUpdated(
						new_total_rewards_for_day.clone(),
						milliseconds_since_epoch_at_day_start.clone(),
						block_at_day_start_as_blocknumber.clone(),
						sender.clone(),
					));
				}
			}
			Ok(())
		}
	}
}
