#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::types::{
	U32F32
};
use codec::{
    Decode,
    Encode,
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
use sp_std::prelude::*; // Imports Vec
#[macro_use]
extern crate alloc; // Required to use Vec

// #[cfg(test)]
// mod tests;

pub trait Config: frame_system::Config + pallet_balances::Config + pallet_timestamp::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId>;
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Debug, Default, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive())]
pub struct RewardDailyData<U, V, W> {
	pub requestor_account_id: U,
	pub total_amt: V,
    pub rewarded_block: W,
}

type DailyData<T> = RewardDailyData<
	<T as frame_system::Config>::AccountId,
	BalanceOf<T>,
    <T as frame_system::Config>::BlockNumber,
>;

// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Config> as CalculatorModule {
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items

		/// Returns reward data that has been distributed for a given day
		pub RewardsPerDay get(fn rewards_daily):
			map hasher(opaque_blake2_256) T::Moment =>
				Option<Vec<RewardDailyData<
					<T as frame_system::Config>::AccountId,
					BalanceOf<T>,
					<T as frame_system::Config>::BlockNumber,
				>>>;
	}
}

decl_event!(
    pub enum Event<T> where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        Created(AccountId),
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
		fn update_fixed_daily_rewards(origin, reward_amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// // get the current block & current date/time
			// let current_block = <frame_system::Module<T>>::block_number();
			// let timestamp_sent = <pallet_timestamp::Module<T>>::get();

			// // convert the current date/time to the start of the current day date/time.
			// // i.e. 21 Apr @ 1420 -> 21 Apr @ 0000
			// let milliseconds_per_day = 86400000u64;

			// let timestamp_sent_as_u64;
			// if let Some(_timestamp_sent_as_u64) = TryInto::<u64>::try_into(timestamp_sent).ok() {
			// 	timestamp_sent_as_u64 = _timestamp_sent_as_u64;
			// } else {
			// 	return Err(DispatchError::Other("Unable to convert Moment to u64 for timestamp_sent"));
			// }

			// let _current_block_as_u64_try = TryInto::<u64>::try_into(current_block).ok();
			// let current_block_as_u64;
			// if let Some(_current_block_as_u64) = _current_block_as_u64_try {
			// 	current_block_as_u64 = _current_block_as_u64;
			// } else {
			// 	return Err(DispatchError::Other("Unable to convert current_block BlockNumber to u64"));
			// }

			// let timestamp_sent_at_day_start_as_u64 = timestamp_sent_as_u64.clone() -(current_block_as_u64.clone() * MILLISECS_PER_BLOCK.clone());

			// let timestamp_sent_at_day_start_as_moment;
			// if let Some(_timestamp_sent_at_day_start_as_moment) =
			// 	TryInto::<<T as pallet_timestamp::Config>::Moment>::try_into(
			// 		timestamp_sent_at_day_start_as_u64
			// 	).ok() {

			// 	timestamp_sent_at_day_start_as_moment = _timestamp_sent_at_day_start_as_moment;
			// } else {
			// 	return Err(DispatchError::Other("Unable to convert u64 to Moment"));
			// }
			// debug::info!("Timestamp sent start day as Moment: {:?}",      timestamp_sent_at_day_start_as_moment.clone());
			// // let days_since_unix_epoch = U32F32::from_num(timestamp_sent_as_u64 / milliseconds_per_day);
			// // let days_since_unix_epoch_as_u64 = days_since_unix_epoch.to_num::<u64>();

			// let milliseconds_since_genesis_as_u64 = U32F32::from_num(current_block_as_u64 * MILLISECS_PER_BLOCK);
			// let milliseconds_since_epoch_as_u64 = timestamp_sent_as_u64;
			// let mut days_since_genesis = U32F32::from_num(0);
			// if (milliseconds_since_genesis_as_u64 >= milliseconds_per_day) {
			// 	days_since_genesis = U32F32::from_num(milliseconds_since_genesis_as_u64 / milliseconds_per_day);
			// }
			// let days_since_genesis_as_u64 = days_since_genesis.to_num::<u64>();
			// debug::info!("Days since genesis u64: {:?}", days_since_genesis_as_u64.clone());
			// let days_since_genesis_ceil = days_since_genesis.ceil().to_num::<u64>();
			// debug::info!("Days since genesis u64 ceil: {:?}", days_since_genesis_ceil.clone());
			// // Initialise to start of day 1 since genesis since otherwise overflow when subtract
			// let mut days_since_genesis_round_down = 0u64;
			// if days_since_genesis_ceil >= 1 {
			// 	days_since_genesis_round_down = days_since_genesis_ceil - 1u64;
			// }
			// // convert that value in days back to a timestamp value in milliseconds to
			// // correspond to the start of the day
			// let milliseconds_since_genesis_at_day_start = days_since_genesis_round_down * milliseconds_per_day;

			// // convert the current block number to the block number at the start of the current day.
			// assert!(milliseconds_since_genesis_at_day_start % MILLISECS_PER_BLOCK == 0);
			// let mut block_at_day_start = 0u64;
			// if milliseconds_since_genesis_at_day_start >= MILLISECS_PER_BLOCK {
			// 	block_at_day_start = milliseconds_since_genesis_at_day_start / MILLISECS_PER_BLOCK
			// }

			// let block_at_day_start_as_blocknumber;
			// if let Some(_block_at_day_start_as_blocknumber) =
			// 	TryInto::<<T as frame_system::Config>::BlockNumber>::try_into(
			// 		block_at_day_start
			// 	).ok() {

			// 	block_at_day_start_as_blocknumber = _block_at_day_start_as_blocknumber;
			// } else {
			// 	return Err(DispatchError::Other("Unable to convert u64 to BlockNumber"));
			// }

			// debug::info!("Block at day start as u64: {:?}", block_at_day_start.clone());
			// debug::info!("Block at day start as BlockNumber: {:?}", block_at_day_start_as_blocknumber.clone());

			// debug::info!("Timestamp sent Moment: {:?}", timestamp_sent.clone());
			// debug::info!("Timestamp sent u64: {:?}", timestamp_sent_as_u64.clone());

			// let reward_amount_item: DailyData<T> = RewardDailyData {
			// 	requestor_account_id: sender.clone(),
			// 	total_amt: reward_amount.clone(),
			// 	rewarded_block: block_at_day_start_as_blocknumber.clone(),
			// };
			// let mut reward_amount_vec;

			// debug::info!("Adding reward to storage vector");

			// reward_amount_vec = Vec::new();
			// reward_amount_vec.push(reward_amount_item.clone());

			// <RewardsPerDay<T>>::insert(
			// 	timestamp_sent_at_day_start_as_moment.clone().into(),
			// 	&reward_amount_vec,
			// );

			// // <RewardsPerDay<T>>::append(
			// // 	timestamp_sent_at_day_start_as_moment.clone(),
			// // 	reward_amount_item.clone(),
			// // );

			Ok(())
		}
	}
}
