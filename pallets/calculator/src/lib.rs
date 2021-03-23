#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::types::{
	U16F16
};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, dispatch::DispatchResult, traits::Get};
use frame_system::ensure_signed;

// #[cfg(test)]
// mod tests;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Config> as TemplateModule {
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;

		/// Substrate-fixed accumulator, value starts at 1 (multiplicative identity)
		FixedAccumulator get(fn fixed_value): U16F16 = U16F16::from_num(1);

		pub FixedRewardsPerDay get(fn rewards_daily):
			map hasher(opaque_blake2_256) T::Moment =>
				Option<Vec<(
					<T as frame_system::Trait>::AccountId,
					BalanceOf<T>,
					<T as frame_system::Trait>::BlockNumber
				>>;

		/// Returns daily reward distribution block number corresponding to a given date/time
		pub BlockRewardedForDay get(fn block_rewarded_for_day):
			map hasher(opaque_blake2_256) T::Moment =>
				Option<<T as frame_system::Trait>::BlockNumber>;

		/// Returns date/time corresponding to a given daily reward distribution block number
		pub DayRewardedForBlock get(fn day_rewarded_for_block):
			map hasher(opaque_blake2_256) <T as frame_system::Trait>::BlockNumber =>
				Option<T::Moment>;
	}
}

// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
		SomethingStored(u32, AccountId),
		/// Substrate-fixed accumulator has been updated.
		FixedUpdated(U16F16, U16F16),
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

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn update_fixed_daily_rewards(origin, new_factor: U16F16) -> DispatchResult {
			ensure_signed(origin)?;

			let milliseconds_per_day = 86400000u64;

			// get the current block

			// get the current date/time
			let timestamp_current = <pallet_timestamp::Module<T>>::get();

			// convert the curreht block number to the block number at the start of the current day.

			// convert the current date/time to the start of the current day date/time. i.e. 21 Apr @ 1420 -> 21 Apr @ 0000

			// check if the start of the current day date/time entry exists as a key for `rewards_daily`
			//
			// if so, retrieve the latest `rewards_daily` data stored for the start of that day date/time
			// i.e. (account_id, balance_rewarded, block_number), and add the new reward value to it.
			//
			// else just insert that as a new entry
			//
			// also, check if the start of the current day date/time entry exists as a key for `block_rewarded_for_day`,
			// otherwise add it, with the block number corresponding to the start of the current day as the value
			//
			// repeat for `day_rewarded_for_block`
			//
			// lastly, update in storage the total rewards distributed so far for the current day
			// so users may query state to get the calculated total.


			// let old_accumulated = Self::fixed_value();

			// // Multiply, handling overflow
			// let new_product = old_accumulated.checked_mul(new_factor)
			// 	.ok_or(Error::<T>::Overflow)?;

			// // Write the new value to storage
			// FixedAccumulator::put(new_product);

			// // Emit event
			// Self::deposit_event(RawEvent::FixedUpdated(new_factor, new_product));
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;

			Something::put(something);

			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			match Something::get() {
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					let new = old.checked_add(1).ok_or(Error::<T>::Overflow)?;
					Something::put(new);
					Ok(())
				},
			}
		}
	}
}
