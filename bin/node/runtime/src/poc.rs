// Copyright 2021 IPSE  Developer.
// This file is part of IPSE

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;
use crate::constants::time::HOURS;
use crate::poc_staking as staking;
use crate::poc_staking::AccountIdOfPid;
use crate::poc_staking::DeclaredCapacity;
use node_primitives::GiB;
use num_traits::Zero;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::convert::{Into, TryFrom, TryInto};

use codec::{Decode, Encode};
use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, Get, Imbalance, OnUnbalanced, ReservableCurrency},
	weights::Weight,
	IterableStorageMap, StorageMap, StorageValue,
};
use pallet_treasury as treasury;
use sp_runtime::{
	traits::{CheckedAdd, CheckedDiv, CheckedSub, SaturatedConversion, Saturating},
	Percent,
};
use sp_std::result;
use sp_std::vec;
use sp_std::vec::Vec;
use system::{ensure_root, ensure_signed};

use crate::ipse_traits::PocHandler;

use conjugate_poc::{
	nonce::noncegen_rust,
	poc_hashing::{calculate_scoop, find_best_deadline_rust},
};

use crate::constants::{
	currency::DOLLARS,
	time::{DAYS, MILLISECS_PER_BLOCK, MONTHS},
};

/// block numbers of a year
pub const YEAR: u32 = 365 * DAYS;

pub const GIB: u64 = 1024 * 1024 * 1024;

/// you should not modify the SPEED and the MiningExpire
pub const SPEED: u64 = 11;
pub const MiningExpire: u64 = 2;

type BalanceOf<T> =
	<<T as staking::Trait>::StakingCurrency as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> = <<T as staking::Trait>::StakingCurrency as Currency<
	<T as system::Trait>::AccountId,
>>::PositiveImbalance;

pub trait Trait: system::Trait + timestamp::Trait + treasury::Trait + staking::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type PocAddOrigin: OnUnbalanced<PositiveImbalanceOf<Self>>;

	/// GENESIS_BASE_TARGET
	type GENESIS_BASE_TARGET: Get<u64>;

	type TotalMiningReward: Get<BalanceOf<Self>>;

	type ProbabilityDeviationValue: Get<Percent>;

	type MaxDeadlineValue: Get<u64>;
}

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MiningInfo<AccountId> {
	// when miner is None, it means Treasury
	pub miner: Option<AccountId>,
	pub best_dl: u64,
	pub block: u64,
}

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MiningHistory<Balance, BlockNumber> {
	total_num: u64,
	history: Vec<(BlockNumber, Balance)>,
}

#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct Difficulty {
	pub base_target: u64,
	pub net_difficulty: u64,
	// the block height of adjust difficulty
	pub block: u64,
}

decl_storage! {
	trait Store for Module<T: Trait> as PoC {

		/// difficulties of some duration(50 blocks).
		pub TargetInfo get(fn target_info): Vec<Difficulty>;

		/// deadlines of the mining.
		pub DlInfo get(fn dl_info): Vec<MiningInfo<T::AccountId>>;

		/// the mining history of miners.
		pub History get(fn history): map hasher(twox_64_concat) T::AccountId => Option<MiningHistory<BalanceOf<T>, T::BlockNumber>>;

		/// the reward history of users.
		pub UserRewardHistory get(fn user_reward_history): map hasher(twox_64_concat) T::AccountId => Vec<(T::BlockNumber, BalanceOf<T>)>;

		/// the net power(how much capacity)
		pub NetPower get(fn net_power): u64;

		/// how much capacity that one difficulty.
		pub CapacityOfPerDifficulty get(fn capacity_of_per_difficult): u64 = 5;

		/// how often to adjust difficulty.
		pub AdjustDifficultyDuration get(fn adjust_difficulty_duration): u64 = 50;

		/// how much IPSE that one Gib should staking.
		pub CapacityPrice get(fn capacity_price): BalanceOf<T> = 10.saturated_into::<BalanceOf<T>>() * DOLLARS.saturated_into::<BalanceOf<T>>();

		/// active miners (now_count, [account_id..], last_count, [account_id..])
		pub ActiveMiners get(fn active_miners): (u32, BTreeSet<T::AccountId>, u32, BTreeSet<T::AccountId>);

	}
}

decl_event! {
pub enum Event<T>
	where
	AccountId = <T as system::Trait>::AccountId,
	Balance = <<T as staking::Trait>::StakingCurrency as Currency<<T as system::Trait>::AccountId>>::Balance,
	{
		Minning(AccountId, u64),
		Verify(AccountId, bool),
		// RewardTreasury(AccountId, Balance),
		SetDifficulty(u64),
		SetCapacityOfPerDifficulty(u64),
		SetAdjustDifficultyDuration(u64),
		SetCapacityPrice(Balance),
	}
}

decl_module! {

	 pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		 type Error = Error<T>;

		fn deposit_event() = default;

		const GENESIS_BASE_TARGET: u64 = T::GENESIS_BASE_TARGET::get();

		/// poc mining total reward.
		const TotalMiningReward: BalanceOf<T> = T::TotalMiningReward::get();

		/// the max deviation value of the mining probability。
		const ProbabilityDeviationValue: Percent = T::ProbabilityDeviationValue::get();

		/// max deadine(you should not submit the value up this value).
		const MaxDeadlineValue: u64 = T::MaxDeadlineValue::get();


		/// set the difficulty
		#[weight = 10_000]
		fn set_difficulty(origin, difficulty: u64) {

			ensure_root(origin)?;

			ensure!(difficulty != 0u64, Error::<T>::DifficultyIsZero);

			ensure!(difficulty <= T::GENESIS_BASE_TARGET::get(), Error::<T>::DifficultyIsTooLarge);

			let base_target = T::GENESIS_BASE_TARGET::get() / difficulty;

			let now = <staking::Module<T>>::now().saturated_into::<u64>();
			Self::append_target_info(Difficulty{
					block: now,
					base_target: base_target,
					net_difficulty: T::GENESIS_BASE_TARGET::get() / base_target,
				});

			Self::deposit_event(RawEvent::SetDifficulty(base_target));

		}


		/// how often to adjust the difficulty.
		#[weight = 10_000]
		fn set_adjust_difficulty_duration(origin, block_num: u64) {
			ensure_root(origin)?;
			ensure!(block_num > 0u64, Error::<T>::DurationIsZero);
			<AdjustDifficultyDuration>::put(block_num);
			Self::deposit_event(RawEvent::SetAdjustDifficultyDuration(block_num));
		}

		/// how much IPSE that one Gib should staking.
		#[weight = 10_000]
		fn set_capacity_price(origin, price: BalanceOf<T>) {
			ensure_root(origin)?;
			<CapacityPrice<T>>::put(price);
			Self::deposit_event(RawEvent::SetCapacityPrice(price));

		}


		/// how much capacity that one difficulty.
		#[weight = 10_000]
		fn set_capacity_of_per_difficulty(origin, capacity: GiB) {
			ensure_root(origin)?;
			ensure!(capacity != 0u64, Error::<T>::CapacityIsZero);
			<CapacityOfPerDifficulty>::put(capacity);

			Self::deposit_event(RawEvent::SetCapacityOfPerDifficulty(capacity));
		}


		/// submit deadline.
		#[weight = 50_000_000 as Weight + T::DbWeight::get().reads(8 as Weight).saturating_add(T::DbWeight::get().writes(3 as Weight))]
		fn mining(origin, numeric_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {

			let miner = ensure_signed(origin)?;

			<ActiveMiners<T>>::mutate(|h| if h.1.insert(miner.clone()) {
				h.0 += 1;
			});

			debug::info!("miner: {:?},  submit deadline!, height = {}, deadline = {}", miner.clone(), height, deadline);

			ensure!(deadline <= T::MaxDeadlineValue::get(), Error::<T>::DeadlineTooLarge);

			ensure!(<staking::Module<T>>::is_can_mining(miner.clone())?, Error::<T>::NotRegister);

			let real_pid = <staking::Module<T>>::disk_of(&miner).unwrap().numeric_id;

			ensure!(real_pid == numeric_id.into(), Error::<T>::PidErr);

			let current_block = <system::Module<T>>::block_number().saturated_into::<u64>();

			debug::info!("starting Verify Deadline !!!");

			if !(current_block / MiningExpire == height / MiningExpire && current_block >= height)
			{
				debug::info!("expire! ：{:?}, off chain get info block: {:?}, deadline is: {:?}", height, current_block, deadline);

				return Err(Error::<T>::HeightNotInDuration)?;
			}

			let dl = Self::dl_info();
			let (block, best_dl) = if let Some(dl_info) = dl.iter().last() {
				(dl_info.clone().block, dl_info.best_dl)
			} else {
				(0, core::u64::MAX)
			};


			// Someone(miner) has mined a better deadline at this mining cycle before.
			if best_dl <= deadline && current_block / MiningExpire == block / MiningExpire {

				debug::info!("not best deadline! best_dl = {}, submit deadline = {}!", best_dl, deadline);

				return Err(Error::<T>::NotBestDeadline)?;
			}

			let verify_ok = Self::verify_dl(numeric_id, height, sig, nonce, deadline);

			if verify_ok.0 {
				debug::info!("verify is ok!, deadline = {}", deadline);

				if current_block / MiningExpire == block / MiningExpire {

					DlInfo::<T>::mutate(|dl| dl.pop());

				}

				Self::append_dl_info(MiningInfo{
						miner: Some(miner.clone()),
						best_dl: deadline,
						block: current_block,
					});

				Self::deposit_event(RawEvent::Minning(miner, deadline));

			}

			else {
				debug::info!("verify failed! deadline = {:?}, target = {:?}, base_target = {:?}", verify_ok.1 / verify_ok.2, verify_ok.1, verify_ok.2);
				return Err(Error::<T>::VerifyFaile)?;
			}

			Ok(())
		}


		fn on_initialize(n: T::BlockNumber) -> Weight{

			if n == T::BlockNumber::from(1u32) {
			Self::append_target_info(Difficulty{
						base_target: T::GENESIS_BASE_TARGET::get(),
						net_difficulty: 1,
						block: 1,
					});
			}
			0
		}


		fn on_finalize(n: T::BlockNumber) {

			if (n % ( 8 * HOURS).saturated_into::<T::BlockNumber>()).is_zero() {
				<ActiveMiners<T>>::mutate(|h| {
					h.2 = h.0.clone();
					h.3 = h.1.clone();
					h.1.clear();
					h.0 = 0u32;
				});
			}
			let current_block = n.saturated_into::<u64>();
			let last_mining_block = Self::get_last_mining_block();

			debug::info!("current-block = {}, last-mining-block = {}", current_block, last_mining_block);

			let reward_result = Self::get_reward_amount();

			let mut reward: BalanceOf<T>;

			if reward_result.is_ok() {
				reward = reward_result.unwrap();
			}
			else {
				return
			}

			if (current_block + 1) % MiningExpire == 0 {

				if current_block / MiningExpire == last_mining_block / MiningExpire {

					if let Some(miner_info) = Self::dl_info().last() {
						let miner: Option<T::AccountId> = miner_info.clone().miner;
						if miner.is_some() {
							Self::reward(miner.unwrap(), reward);
							debug::info!("<<REWARD>> miner on block {}, last_mining_block {}", current_block, last_mining_block);
						}

					}

				}

				else {
					Self::treasury_minning(current_block);
					Self::reward_treasury(reward);
				}

			}

			if current_block % <AdjustDifficultyDuration>::get() == 0 {
				Self::adjust_difficulty(current_block);
			}

			Self::get_total_capacity();

		}

	 }
}

impl<T: Trait> Module<T> {
	fn adjust_difficulty(block: u64) {
		debug::info!("[ADJUST] difficulty on block {}", block);

		let last_base_target = Self::get_last_base_target().0;
		let last_net_difficulty = Self::get_last_base_target().1;

		let ave_deadline = Self::get_ave_deadline().1;
		let mining_count = Self::get_ave_deadline().0;

		if (ave_deadline < 2000 && mining_count > 0) {
			let mut new = last_base_target.saturating_mul(10) / SPEED;
			if new == 0 {
				new = 1;
			}

			debug::info!("[DIFFICULTY] make more difficult, base_target = {:?}", new);
			Self::append_target_info(Difficulty {
				block,
				base_target: new,
				net_difficulty: T::GENESIS_BASE_TARGET::get() / new,
			});
		} else if ave_deadline > 3000 {
			let new = last_base_target.saturating_mul(SPEED) / 10;
			Self::append_target_info(Difficulty {
				block,
				base_target: new,
				net_difficulty: T::GENESIS_BASE_TARGET::get() / new,
			});
			debug::info!("[DIFFICULTY] make easier,  base_target = {}", new);
		} else {
			let new = last_base_target;
			debug::info!("[DIFFICULTY] use avg,  base_target = {}", new);
			Self::append_target_info(Difficulty {
				block,
				base_target: new,
				net_difficulty: T::GENESIS_BASE_TARGET::get() / new,
			});
		}
	}

	fn treasury_minning(current_block: u64) {
		Self::append_dl_info(MiningInfo {
			miner: None,
			best_dl: T::MaxDeadlineValue::get(),
			block: current_block,
		});
		debug::info!("<<REWARD>> treasury on block {}", current_block);
	}

	fn get_current_base_target() -> u64 {
		let ti = Self::target_info();
		ti.iter().last().unwrap().base_target
	}

	fn get_last_mining_block() -> u64 {
		let dl = Self::dl_info();
		if let Some(dl) = dl.iter().last() {
			dl.block
		} else {
			0
		}
	}

	fn get_last_base_target() -> (u64, u64) {
		let ti = Self::target_info();

		if let Some(info) = ti.iter().last() {
			(info.base_target, info.net_difficulty)
		} else {
			(T::GENESIS_BASE_TARGET::get(), 1u64)
		}
	}

	fn get_ave_deadline() -> (u64, u64) {
		let dl = Self::dl_info();
		let mut iter = dl.iter().rev();
		let mut count = 0_u64;
		let mut real_count = 0_u64;
		let mut deadline = 0_u64;

		while let Some(dl) = iter.next() {
			if count == <AdjustDifficultyDuration>::get() / MiningExpire {
				break
			}
			if dl.miner.is_some() {
				real_count += 1;
				deadline += dl.best_dl;
			}

			count += 1;
		}

		if real_count == 0 {
			(0, 0u64)
		} else {
			(real_count, deadline / real_count)
		}
	}

	fn verify_dl(
		account_id: u64,
		height: u64,
		sig: [u8; 32],
		nonce: u64,
		deadline: u64,
	) -> (bool, u64, u64) {
		let scoop_data = calculate_scoop(height, &sig) as u64;
		debug::info!("scoop_data: {:?}", scoop_data);
		debug::info!("sig: {:?}", sig);

		let mut cache = vec![0_u8; 262144];

		noncegen_rust(&mut cache[..], account_id, nonce, 1);

		let mirror_scoop_data = Self::gen_mirror_scoop_data(scoop_data, cache);

		let (target, _) = find_best_deadline_rust(mirror_scoop_data.as_ref(), 1, &sig);
		debug::info!("target: {:?}", target);
		let base_target = Self::get_current_base_target();
		let deadline_ = target / base_target;
		debug::info!("deadline: {:?}", deadline_);
		(deadline == target / base_target, target, base_target)
	}

	fn gen_mirror_scoop_data(scoop_data: u64, cache: Vec<u8>) -> Vec<u8> {
		let addr = 64 * scoop_data as usize;
		let mirror_scoop = 4095 - scoop_data as usize;
		let mirror_addr = 64 * mirror_scoop as usize;
		let mut mirror_scoop_data = vec![0_u8; 64];
		mirror_scoop_data[0..32].clone_from_slice(&cache[addr..addr + 32]);
		mirror_scoop_data[32..64].clone_from_slice(&cache[mirror_addr + 32..mirror_addr + 64]);
		mirror_scoop_data
	}

	fn get_treasury_id() -> T::AccountId {
		<treasury::Module<T>>::account_id()
	}

	fn get_reward_amount() -> result::Result<BalanceOf<T>, DispatchError> {
		let now = <staking::Module<T>>::now();

		let sub_half_reward_time = 2u32;

		let year = now.checked_div(&T::BlockNumber::from(YEAR)).ok_or(Error::<T>::DivZero)?;
		let duration = year / T::BlockNumber::from(sub_half_reward_time);

		let duration = <<T as system::Trait>::BlockNumber as TryInto<u32>>::try_into(duration)
			.map_err(|_| Error::<T>::ConvertErr)? +
			1u32;

		let n_opt = sub_half_reward_time.checked_pow(duration);

		let reward: BalanceOf<T>;

		if n_opt.is_some() {
			let n = <BalanceOf<T>>::from(n_opt.unwrap());

			reward = T::TotalMiningReward::get() /
				n / 2.saturated_into::<BalanceOf<T>>() /
				Self::block_convert_to_balance(T::BlockNumber::from(YEAR))?;

			Ok(reward * MiningExpire.saturated_into::<BalanceOf<T>>())
		} else {
			Ok(<BalanceOf<T>>::from(0u32))
		}
	}

	fn block_convert_to_balance(n: T::BlockNumber) -> result::Result<BalanceOf<T>, DispatchError> {
		let n_u = <<T as system::Trait>::BlockNumber as TryInto<u32>>::try_into(n)
			.map_err(|_| Error::<T>::ConvertErr)?;
		let n_b =
			<BalanceOf<T> as TryFrom<u32>>::try_from(n_u).map_err(|_| Error::<T>::ConvertErr)?;
		Ok(n_b)
	}

	fn reward_treasury(reward: BalanceOf<T>) {
		let account_id = Self::get_treasury_id();
		T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(&account_id, reward));
	}

	fn reward_black_hold(reward: BalanceOf<T>) {
		let account_id = T::AccountId::default();
		T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(&account_id, reward));
	}

	fn reward(miner: T::AccountId, mut reward: BalanceOf<T>) -> DispatchResult {
		let all_reward = reward.clone();

		let machine_info = <staking::Module<T>>::disk_of(&miner).ok_or(Error::<T>::NotRegister)?;
		let disk = machine_info.clone().plot_size;
		let update_time = machine_info.clone().update_time;

		let mut miner_mining_num = match <History<T>>::get(&miner) {
			Some(h) => h.total_num + 1u64,
			None => 1u64,
		};

		let now = <staking::Module<T>>::now();

		let staking_info_opt = <staking::Module<T>>::staking_info_of(&miner);

		if staking_info_opt.is_some() {
			let total_staking = staking_info_opt.unwrap().total_staking;

			let miner_should_staking_amount =
				disk.saturated_into::<BalanceOf<T>>().saturating_mul(<CapacityPrice<T>>::get()) /
					GIB.saturated_into::<BalanceOf<T>>();

			if miner_should_staking_amount <= total_staking {
				debug::info!("miner's staking enough！staking enough = {:?} ", total_staking);

				let mut net_mining_num = (now - update_time).saturated_into::<u64>() / MiningExpire;

				if net_mining_num < miner_mining_num {
					net_mining_num = miner_mining_num
				}

				debug::info!(
					"miner: {:?}, mining probability: {:?} / {:?}",
					miner.clone(),
					miner_mining_num,
					net_mining_num
				);

				let net_should_staking_total_amount = Self::get_total_capacity()
					.saturated_into::<BalanceOf<T>>()
					.saturating_mul(<CapacityPrice<T>>::get()) /
					GIB.saturated_into::<BalanceOf<T>>();

				if (miner_mining_num
					.saturated_into::<BalanceOf<T>>()
					.saturating_mul(net_should_staking_total_amount) >
					(net_mining_num.saturated_into::<BalanceOf<T>>() *
						miner_should_staking_amount)
						.saturating_add(
							T::ProbabilityDeviationValue::get() *
								(net_mining_num.saturated_into::<BalanceOf<T>>() *
									miner_should_staking_amount),
						)) || ((net_mining_num.saturated_into::<BalanceOf<T>>() *
					miner_should_staking_amount)
					.saturating_sub(
						T::ProbabilityDeviationValue::get() *
							net_mining_num.saturated_into::<BalanceOf<T>>() *
							miner_should_staking_amount,
					) > miner_mining_num
					.saturated_into::<BalanceOf<T>>()
					.saturating_mul(net_should_staking_total_amount))
				{
					debug::info!("Miners: {:?} have a high probability of mining, and you should increase the disk space", miner.clone());

					reward = Percent::from_percent(10) * reward;

					Self::reward_staker(miner.clone(), reward);

					Self::reward_default(Percent::from_percent(90) * all_reward);
				} else {
					debug::info!("Get all reward.");
					reward = reward;
					Self::reward_staker(miner.clone(), reward);
				}
			} else {
				debug::info!("Get 10% reward.");
				reward = Percent::from_percent(10) * reward;
				Self::reward_staker(miner.clone(), reward);
				Self::reward_default(Percent::from_percent(90) * all_reward);
			}
		} else {
			debug::info!("miner have no staking info.");
			reward = Percent::from_percent(10) * reward;
			Self::reward_staker(miner.clone(), reward);
			Self::reward_default(Percent::from_percent(90) * all_reward);
		}

		let history_opt = <History<T>>::get(&miner);

		if history_opt.is_some() {
			let mut his = history_opt.unwrap();
			his.total_num = miner_mining_num;
			his.history.push((now, reward));

			if his.history.len() >= 300 {
				let mut old_history = his.history.clone();
				let new_history = old_history.split_off(1);
				his.history = new_history;
			}
			<History<T>>::insert(miner.clone(), his);
		} else {
			let history = vec![(now, reward)];
			<History<T>>::insert(
				miner.clone(),
				MiningHistory { total_num: miner_mining_num, history },
			);
		}

		Ok(())
	}

	fn reward_staker(miner: T::AccountId, reward: BalanceOf<T>) -> DispatchResult {
		let now = <staking::Module<T>>::now();

		let staking_info =
			<staking::Module<T>>::staking_info_of(&miner).ok_or(Error::<T>::NotRegister)?;
		let stakers = staking_info.clone().others;
		if stakers.len() == 0 {
			Self::reward_miner(miner.clone(), reward, now);
		} else {
			let miner_reward = staking_info.clone().miner_proportion * reward;
			Self::reward_miner(miner.clone(), miner_reward, now);

			let stakers_reward = reward - miner_reward;
			let total_staking = staking_info.clone().total_staking;
			for staker_info in stakers.iter() {
				let staker_reward = stakers_reward
					.saturating_mul(staker_info.clone().1)
					.checked_div(&total_staking)
					.ok_or(Error::<T>::DivZero)?;

				if staker_info.clone().0 == miner.clone() {
					Self::reward_miner(miner.clone(), staker_reward, now);
				} else {
					T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(
						&staker_info.clone().0,
						staker_reward,
					));
					Self::update_reword_history(staker_info.clone().0, staker_reward, now);
				}
			}
		}

		Ok(())
	}

	fn reward_default(reward: BalanceOf<T>) {
		let now = <staking::Module<T>>::now();
		if now >= T::BlockNumber::from(3 * MONTHS) {
			Self::reward_treasury(reward);
		} else {
			// The amount lost by the user is put into the black hole address for the convenience of
			// preliminary testing
			Self::reward_black_hold(reward);
		}
	}

	fn get_total_capacity() -> u64 {
		let mut old_target_info_vec = <TargetInfo>::get();
		let len = old_target_info_vec.len();
		if len > 6 {
			let new_target_info = old_target_info_vec.split_off(len - 6);
			old_target_info_vec = new_target_info;
		}
		let len = old_target_info_vec.len() as u64;

		let mut total_difficulty = 0u64;

		for i in old_target_info_vec.iter() {
			total_difficulty += i.net_difficulty;
		}

		let mut avg_difficulty = 0;
		if len == 0 {
			avg_difficulty = 0;
		} else {
			avg_difficulty = total_difficulty / len;
		}

		let capacity = avg_difficulty.saturating_mul(GIB * <CapacityOfPerDifficulty>::get());

		<NetPower>::put(capacity);

		return capacity
	}

	fn reward_miner(miner: T::AccountId, amount: BalanceOf<T>, now: T::BlockNumber) {
		let disk = <staking::Module<T>>::disk_of(&miner).unwrap();
		let reward_dest = disk.reward_dest;
		if reward_dest == miner.clone() {
			T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(
				&reward_dest,
				amount,
			));
			Self::update_reword_history(reward_dest.clone(), amount, now);
		} else {
			let miner_reward = Percent::from_percent(10) * amount;
			T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(
				&miner,
				miner_reward,
			));
			Self::update_reword_history(miner, miner_reward, now);

			let dest_reward = amount.saturating_sub(miner_reward);
			T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(
				&reward_dest,
				dest_reward,
			));
			Self::update_reword_history(reward_dest, dest_reward, now);
		}
	}

	fn update_reword_history(
		account_id: T::AccountId,
		amount: BalanceOf<T>,
		block_num: T::BlockNumber,
	) {
		let mut reward_history = <UserRewardHistory<T>>::get(account_id.clone());

		reward_history.push((block_num, amount));

		if reward_history.len() >= 300 {
			let mut old_history = reward_history.clone();
			let new_history = old_history.split_off(1);
			<UserRewardHistory<T>>::insert(account_id, new_history);
		} else {
			<UserRewardHistory<T>>::insert(account_id, reward_history);
		}
	}

	fn append_dl_info(dl_info: MiningInfo<T::AccountId>) {
		let mut old_dl_info_vec = <DlInfo<T>>::get();
		let len = old_dl_info_vec.len();

		old_dl_info_vec.push(dl_info);

		if len >= 2000 {
			let new_dl_info = old_dl_info_vec.split_off(len - 2000);
			old_dl_info_vec = new_dl_info;
		}

		<DlInfo<T>>::put(old_dl_info_vec);
	}

	fn append_target_info(difficulty: Difficulty) {
		let mut old_target_info_vec = <TargetInfo>::get();
		let len = old_target_info_vec.len();
		old_target_info_vec.push(difficulty);
		if len >= 50 {
			let new_target_info = old_target_info_vec.split_off(len - 50);
			old_target_info_vec = new_target_info;
		}

		<TargetInfo>::put(old_target_info_vec);
	}
}

impl<T: Trait> PocHandler<T::AccountId> for Module<T> {
	fn remove_history(miner: T::AccountId) {
		<History<T>>::remove(miner);
	}
}

decl_error! {
	/// Error for the ipse module.
	pub enum Error for Module<T: Trait> {
		/// 0 can't be a divisor.
		DivZero,
		/// not register.
		NotRegister,
		/// not your plot id.
		PidErr,
		/// data type conversion error
		ConvertErr,
		/// submit deadline too delay.
		HeightNotInDuration,
		/// not best deadline
		NotBestDeadline,
		/// deadline verify failed.
		VerifyFaile,
		/// the capacity should not empty.
		CapacityIsZero,
		/// submit deadline up max value.
		DeadlineTooLarge,
		/// the block number should not zero.
		DurationIsZero,
		/// the difficulty should not zero.
		DifficultyIsZero,
		/// the difficulty up max value.
		DifficultyIsTooLarge,
	}
}
