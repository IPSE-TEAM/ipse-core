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

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;
extern crate pallet_balances as balances;
extern crate pallet_babe as babe;
use crate::constants::time::MILLISECS_PER_BLOCK;

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, DispatchError,}, debug,
	traits::{Get, Currency, ReservableCurrency, OnUnbalanced, LockableCurrency, LockIdentifier, WithdrawReason},
    weights::Weight,
	StorageMap, StorageValue,
	decl_error, ensure,
};

use pallet_staking as staking;

use sp_std::result;

use system::{ensure_signed};
use sp_runtime::{traits::{SaturatedConversion, Saturating, CheckedDiv, CheckedAdd, CheckedSub}, Percent};
use sp_std::vec::Vec;
use sp_std::vec;
use node_primitives::GIB;
use crate::ipse_traits::PocHandler;
use sp_std::{collections::btree_set::BTreeSet};

const Staking_ID: LockIdentifier = *b"pocstake";

type BalanceOf<T> =
	<<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait + timestamp::Trait + balances::Trait + babe::Trait + staking::Trait {

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type ChillDuration: Get<Self::BlockNumber>;

	type StakingCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId>;

	type StakingDeposit: Get<BalanceOf<Self>>;

	type PocStakingMinAmount: Get<BalanceOf<Self>>;

	type StakingSlash: OnUnbalanced<NegativeImbalanceOf<Self>>;

	type StakerMaxNumber: Get<usize>;

	type PocHandler: PocHandler<Self::AccountId>;

	type StakingLockExpire: Get<Self::BlockNumber>;

	type RecommendLockExpire: Get<Self::BlockNumber>;

	type RecommendMaxNumber: Get<usize>;

}


#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineInfo<BlockNumber, AccountId> {

	pub plot_size: GIB,

	pub numeric_id: u128,

	pub update_time: BlockNumber,

	pub is_stop: bool,

	pub reward_dest: AccountId,

}



#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct StakingInfo<AccountId, Balance> {

	pub miner: AccountId,

	pub miner_proportion: Percent,

	pub total_staking: Balance,

	pub others: Vec<(AccountId, Balance, Balance)>,
}



#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Operate {

	Add,

	Sub,
}

impl Default for Operate {
	fn default() -> Self {
		Self::Add
	}
}


decl_storage! {
    trait Store for Module<T: Trait> as IpseStakingModule {

		/// the machine info of miners.
		pub DiskOf get(fn disk_of): map hasher(twox_64_concat) T::AccountId => Option<MachineInfo<T::BlockNumber, T::AccountId>>;

		/// is in the chill time(only miners can update their info).
		pub IsChillTime get(fn is_chill_time): bool = true;

		/// the staking info of miners.
		pub StakingInfoOf get(fn staking_info_of): map hasher(twox_64_concat) T::AccountId => Option<StakingInfo<T::AccountId, BalanceOf<T>>>;

		/// the miners of the user that help stake.
		pub MinersOf get(fn miners_of): map hasher(twox_64_concat) T::AccountId => Vec<T::AccountId>;

		/// whose plot id?.
		pub AccountIdOfPid get(fn accouont_id_of_pid): map hasher(twox_64_concat) u128 => Option<T::AccountId>;

		/// exposed miners(hope someone to stake him).
		pub RecommendList get(fn recommend_list): Vec<(T::AccountId, BalanceOf<T>)>;

		/// the total declared capacity in the entire network.
		pub DeclaredCapacity get(fn declared_capacity): u64;

		/// minsers that already registered
		pub Miners get(fn miners): BTreeSet<T::AccountId>;

		/// miners whom is mining.
		pub MiningMiners get(fn mining_miners): BTreeSet<T::AccountId>;

		/// the total number of mining miners.
		pub MiningNum get(fn mining_num): u64;

		/// locks.
		pub Locks get(fn locks): map hasher(twox_64_concat) T::AccountId => Option<Vec<(T::BlockNumber, BalanceOf<T>)>>;

		/// The chill time  (start, end).
		pub ChillTime get(fn chill_time): (T::BlockNumber, T::BlockNumber);


    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId,
	Balance = <<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance,
    {

        UpdatePlotSize(AccountId, GIB),
        Register(AccountId, u64),
        StopMining(AccountId),
        RemoveStaker(AccountId, AccountId),
        Staking(AccountId, AccountId, Balance),
        UpdateProportion(AccountId, Percent),
		UpdateStaking(AccountId, Balance),
		ExitStaking(AccountId, AccountId),
		UpdateNumericId(AccountId, u128),
		RequestUpToList(AccountId, Balance),
		RequestDownFromList(AccountId),
		Unlock(AccountId),
		RestartMining(AccountId),
		UpdateRewardDest(AccountId, AccountId),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
     	/// how many block that the chill time.
     	const ChillDuration: T::BlockNumber = T::ChillDuration::get();
     	/// how much LT you should deposit when staking.
     	const StakingDeposit: BalanceOf<T> = T::StakingDeposit::get();
		/// the min amount of staking.
     	const PocStakingMinAmount: BalanceOf<T> = T::PocStakingMinAmount::get();
     	/// the max users number that can help miner stake.
     	const StakerMaxNumber: u32 = T::StakerMaxNumber::get() as u32;
     	/// how many blocks that can unlock when you not stake.
     	const StakingLockExpire: T::BlockNumber = T::StakingLockExpire::get();
     	/// how many blocks that can unlock when you down the recommend list.
     	const RecommendLockExpire: T::BlockNumber = T::RecommendLockExpire::get();
     	/// the max miners number of the recommend list.
     	const RecommendMaxNumber: u32 = T::RecommendMaxNumber::get() as u32;


     	type Error = Error<T>;

        fn deposit_event() = default;


		/// register.
		#[weight = 10_000]
		fn register(origin, plot_size: GIB, numeric_id: u128, miner_proportion: u32, reward_dest: Option<T::AccountId>) {

			let miner_proportion = Percent::from_percent(miner_proportion as u8);

			let miner = ensure_signed(origin)?;

			let kib = plot_size;

			let pid = numeric_id;

			ensure!(kib != 0 as GIB, Error::<T>::PlotSizeIsZero);

			let disk = kib.checked_mul((1024 * 1024 * 1024) as GIB).ok_or(Error::<T>::Overflow)?;

			ensure!(!Self::is_register(miner.clone()), Error::<T>::AlreadyRegister);

			ensure!(!<AccountIdOfPid<T>>::contains_key(pid), Error::<T>::NumericIdInUsing);

			<DeclaredCapacity>::mutate(|h| *h += disk);

			let dest: T::AccountId;
			if reward_dest.is_some() {
				dest = reward_dest.unwrap();
			}
			else {
				dest = miner.clone();
			}

			let now = Self::now();
			<DiskOf<T>>::insert(miner.clone(), MachineInfo {
        		plot_size: disk,
        		numeric_id: pid,
        		update_time: now,
        		is_stop: false,
        		reward_dest: dest,

        	});

        	<StakingInfoOf<T>>::insert(&miner,
        		StakingInfo {

        			miner: miner.clone(),
        			miner_proportion: miner_proportion,
        			total_staking: <BalanceOf<T>>::from(0u32),
        			others: vec![],
        		}
        	);

        	<AccountIdOfPid<T>>::insert(pid, miner.clone());

        	<Miners<T>>::mutate(|h| h.insert(miner.clone()));

        	<MiningMiners<T>>::mutate(|h| h.insert(miner.clone()));

        	Self::deposit_event(RawEvent::Register(miner, disk));

		}


		/// request to expose in recommend list.
		#[weight = 10_000]
		fn request_up_to_list(origin, amount: BalanceOf<T>) {

			let miner = ensure_signed(origin)?;

			ensure!(Self::is_can_mining(miner.clone())?, Error::<T>::NotRegister);

			Self::sort_account_by_amount(miner.clone(), amount)?;

			Self::deposit_event(RawEvent::RequestUpToList(miner, amount));

		}


		/// request to down from the recommended list
		#[weight = 10_000]
		fn request_down_from_list(origin) {
			let miner = ensure_signed(origin)?;
			let mut list = <RecommendList<T>>::get();
			if let Some(pos) = list.iter().position(|h| h.0 == miner) {
				let amount = list.remove(pos).1;

				T::StakingCurrency::unreserve(&miner, amount);

				let now = Self::now();
				let expire = now.saturating_add(T::RecommendLockExpire::get());
				Self::lock_add_amount(miner.clone(), amount, expire);

				<RecommendList<T>>::put(list);
			}
			else {
				return Err(Error::<T>::NotInList)?;
			}

			Self::deposit_event(RawEvent::RequestDownFromList(miner));

		}

		/// the miner modify income address.
		#[weight = 10_000]
		fn update_reward_dest(origin, dest: T::AccountId) {
			let miner = ensure_signed(origin)?;
			ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);
			<DiskOf<T>>::mutate(miner.clone(), |h| if let Some(i) = h {
				i.reward_dest = dest.clone();

			}
			);

			Self::deposit_event(RawEvent::UpdateRewardDest(miner, dest));

		}


		/// the miner modify plot id.
		#[weight = 10_000]
		fn update_numeric_id(origin, numeric_id: u128) {
			let miner = ensure_signed(origin)?;

			let pid = numeric_id;

			ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);

			ensure!(!(<AccountIdOfPid<T>>::contains_key(pid) && <AccountIdOfPid<T>>::get(pid).unwrap() != miner.clone()) , Error::<T>::NumericIdInUsing);

			let old_pid = <DiskOf<T>>::get(miner.clone()).unwrap().numeric_id;

			<AccountIdOfPid<T>>::remove(old_pid);

			<DiskOf<T>>::mutate(miner.clone(), |h| if let Some(i) = h {
				i.numeric_id = pid;

			}
			);

			// T::PocHandler::remove_history(miner.clone());

			<AccountIdOfPid<T>>::insert(pid, miner.clone());

			Self::deposit_event(RawEvent::UpdateNumericId(miner, pid));

		}


		/// the miner modify the plot size.
        #[weight = 10_000]
        fn update_plot_size(origin, plot_size: GIB) {

        	let miner = ensure_signed(origin)?;

        	let kib = plot_size;

			let disk = kib.checked_mul((1024 * 1024 * 1024) as GIB).ok_or(Error::<T>::Overflow)?;

			ensure!(disk != 0 as GIB, Error::<T>::PlotSizeIsZero);

			ensure!(Self::is_chill_time(), Error::<T>::ChillTime);

			T::PocHandler::remove_history(miner.clone());

        	let now = Self::now();

        	ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);

        	<DiskOf<T>>::mutate(miner.clone(), |h| if let Some(i) = h {
        		if i.is_stop == false {
        			<DeclaredCapacity>::mutate(|h| *h -= i.plot_size);
					i.plot_size = disk;
					<DeclaredCapacity>::mutate(|h| *h += i.plot_size);
					i.update_time = now;

        		}
        		else {
        			i.plot_size = disk;
        			i.update_time = now;
        		}

        	}
        	);

        	Self::deposit_event(RawEvent::UpdatePlotSize(miner, disk));

        }


		/// the miner stop the machine.
		#[weight = 10_000]
        fn stop_mining(origin) {

        	let miner = ensure_signed(origin)?;

        	Self::is_can_mining(miner.clone())?;

			<DiskOf<T>>::mutate(miner.clone(), |h| {
				if let Some(x) = h {
					x.is_stop = true;
					<DeclaredCapacity>::mutate(|h| *h -= x.plot_size);
					<MiningMiners<T>>::mutate(|h| h.remove(&miner));
				}
			});

			Self::deposit_event(RawEvent::StopMining(miner));
		}


		/// the miner restart mining.
		#[weight = 10_000]
		fn restart_mining(origin) {
			let miner = ensure_signed(origin)?;

			ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);

			ensure!(<DiskOf<T>>::get(miner.clone()).unwrap().is_stop == true, Error::<T>::MiningNotStop);
			<DiskOf<T>>::mutate(miner.clone(), |h| {
				if let Some(x) = h {
					let now = Self::now();
					x.update_time = now;
					x.is_stop = false;
					<DeclaredCapacity>::mutate(|h| *h += x.plot_size);
					<MiningMiners<T>>::mutate(|h| h.insert(miner.clone()));
				}
			});
			T::PocHandler::remove_history(miner.clone());

			Self::deposit_event(RawEvent::RestartMining(miner));
		}


        /// the delete him staker.
        #[weight = 10_000]
        fn remove_staker(origin, staker: T::AccountId) {

			let miner = ensure_signed(origin)?;

			Self::update_staking_info(miner.clone(), staker.clone(), Operate::Sub, None, true)?;

			Self::staker_remove_miner(staker.clone(), miner.clone());

			Self::deposit_event(RawEvent::RemoveStaker(miner, staker));
        }


		/// the user stake for miners.
        #[weight = 10_000]
        fn staking(origin, miner: T::AccountId, amount: BalanceOf<T>) {

        	let who = ensure_signed(origin)?;

			Self::is_can_mining(miner.clone())?;

			ensure!(!<IsChillTime>::get(), Error::<T>::ChillTime);

			ensure!(amount >= T::PocStakingMinAmount::get(), Error::<T>::StakingAmountooLow);

			if Self::staker_pos(miner.clone(), who.clone()).is_some() {

				return Err(Error::<T>::AlreadyStaking)?;
			}

			let bond = amount.checked_add(&T::StakingDeposit::get()).ok_or(Error::<T>::Overflow)?;

			let staker_info = (who.clone(), amount.clone(), T::StakingDeposit::get());

			let mut staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();

			ensure!(staking_info.others.len() < T::StakerMaxNumber::get(), Error::<T>::StakerNumberToMax);

			let total_amount = staking_info.clone().total_staking;

			let now_amount = total_amount.checked_add(&amount).ok_or(Error::<T>::Overflow)?;

			T::StakingCurrency::reserve(&who, bond)?;

			staking_info.total_staking = now_amount;

			staking_info.others.push(staker_info);

			<StakingInfoOf<T>>::insert(miner.clone(), staking_info);

			<MinersOf<T>>::mutate(who.clone(), |h| h.push(miner.clone()));

			Self::deposit_event(RawEvent::Staking(who, miner, amount));

        }


		/// users update their staking amount.
        #[weight = 10_000]
        fn update_staking(origin, miner: T::AccountId, operate: Operate , amount: BalanceOf<T>) {

        	let staker = ensure_signed(origin)?;

			Self::update_staking_info(miner, staker.clone(), operate, Some(amount), false)?;

			Self::deposit_event(RawEvent::UpdateStaking(staker, amount));

        }


        /// unlock
        #[weight = 10_000]
        fn unlock(origin) {
        	let staker = ensure_signed(origin)?;
        	Self::lock_sub_amount(staker.clone());
        	Self::deposit_event(RawEvent::Unlock(staker));

        }


        /// the user exit staking.
        #[weight = 10_000]
        fn exit_Staking(origin, miner: T::AccountId) {
        	let staker = ensure_signed(origin)?;
        	Self::update_staking_info(miner.clone(), staker.clone(), Operate ::Sub, None, false)?;
        	Self::staker_remove_miner(staker.clone(), miner.clone());
        	Self::deposit_event(RawEvent::ExitStaking(staker, miner));

        }


		/// miners update their mining reward proportion.
        #[weight = 10_000]
        fn update_proportion(origin, proportion: Percent) {

        	let miner = ensure_signed(origin)?;

        	ensure!(<IsChillTime>::get(), Error::<T>::NotChillTime);

        	Self::is_can_mining(miner.clone())?;

        	let mut staking_info = <StakingInfoOf<T>>::get(miner.clone()).unwrap();

        	staking_info.miner_proportion = proportion.clone();

        	<StakingInfoOf<T>>::insert(miner.clone(), staking_info);

			Self::deposit_event(RawEvent::UpdateProportion(miner, proportion));
        }


		fn on_initialize(n: T::BlockNumber) -> Weight {
			// debug::info!("staking_poc----当前打印的高度是:{:?}", Self::now());
			let _ = Self::update_chill();
			0

       }

       fn on_finalize(n: T::BlockNumber) {
       		let num = <MiningMiners<T>>::get().len() as u64;
       		<MiningNum>::put(num);
       }

     }
}

impl<T: Trait> Module<T> {


	fn current_epoch_start() -> result::Result<u64, DispatchError> {

		let time = <babe::Module<T>>::current_epoch_start();
		let block_number = time.checked_div(MILLISECS_PER_BLOCK).ok_or((Error::<T>::Overflow))?;
		Ok(block_number)

	}


	pub fn now() -> T::BlockNumber {

		<system::Module<T>>::block_number()
	}


	fn staker_pos(miner: T::AccountId, staker: T::AccountId) -> Option<usize> {
		let staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();
		let others = staking_info.others;
		let pos = others.iter().position(|h| h.0 == staker);
		pos
	}


	fn update_chill() -> DispatchResult {

		let now = Self::now();

		let era_start_time = <staking::Module<T>>::era_start_block_number();

		let chill_duration = T::ChillDuration::get();  // 一个session区块数

		let era = chill_duration * 6.saturated_into::<T::BlockNumber>();  // 一个era区块数

		// 获取时代消耗的区块
		let num = now % era;
		let num1 = now / era;

		if num < chill_duration {
			let start = num1 * era;
			let end = num1 * era + chill_duration;
			<ChillTime<T>>::put((start, end));
			<IsChillTime>::put(true);
		}
		else {
			let start = (num1 + 1.saturated_into::<T::BlockNumber>()) * era;
			let end = (num1 + 1.saturated_into::<T::BlockNumber>()) * era + chill_duration;
			<ChillTime<T>>::put((start, end));

			<IsChillTime>::put(false);
		}

		Ok(())

	}


	fn is_register(miner: T::AccountId) -> bool {

		if <DiskOf<T>>::contains_key(&miner) && <StakingInfoOf<T>>::contains_key(&miner) {
			true
		}

		else {
			false
		}

	}


	pub fn is_can_mining(miner: T::AccountId) -> result::Result<bool, DispatchError> {
		ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);

		ensure!(!<DiskOf<T>>::get(&miner).unwrap().is_stop, Error::<T>::AlreadyStopMining);

		Ok(true)
	}


	fn staker_remove_miner(staker: T::AccountId, miner: T::AccountId) {

		<MinersOf<T>>::mutate(staker.clone(), |miners|  {
			miners.retain(|h| h != &miner);

		});

	}


	fn sort_after(miner: T::AccountId, amount: BalanceOf<T>, index: usize, mut old_list: Vec<(T::AccountId, BalanceOf<T>)>) -> result::Result<(), DispatchError> {

		if index < T::RecommendMaxNumber::get() {

			T::StakingCurrency::reserve(&miner, amount)?;

			old_list.insert(index, (miner, amount));

		}

		if old_list.len() >= T::RecommendMaxNumber::get() {
			let abandon = old_list.split_off(T::RecommendMaxNumber::get());

			for i in abandon {
				T::StakingCurrency::unreserve(&i.0, i.1);
				let now = Self::now();
				let expire = now.saturating_add(T::RecommendLockExpire::get());

				Self::lock_add_amount(i.0, i.1, expire);
			}
		}

		<RecommendList<T>>::put(old_list);

		if index >= T::RecommendMaxNumber::get() {
			return Err(Error::<T>::AmountTooLow)?;
		}

		Ok(())

	}


	fn lock_add_amount(who: T::AccountId, amount: BalanceOf<T>, expire: T::BlockNumber) {

		Self::lock(who.clone(), Operate ::Add, amount);
		let locks_opt = <Locks<T>>::get(who.clone());
		if locks_opt.is_some() {
			let mut locks = locks_opt.unwrap();
			locks.push((expire, amount));
			<Locks<T>>::insert(who, locks);
		}

		else {
			let mut locks = vec![(expire, amount)];
			<Locks<T>>::insert(who, locks);
		}
	}


	fn lock_sub_amount(who: T::AccountId) {
		let now = Self::now();
		<Locks<T>>::mutate(who.clone(), |h_opt| if let Some(h) = h_opt {
			h.retain(|i|
				if i.0 <= now {
					Self::lock(who.clone(), Operate ::Sub, i.1);
					false
					}
				else {
					true
				}
			);
		});

	}


	fn lock(who: T::AccountId, operate: Operate , amount: BalanceOf<T>) {

		let locks_opt = <Locks<T>>::get(who.clone());
		let reasons = WithdrawReason::Transfer | WithdrawReason::Reserve;
		match operate {
			Operate ::Sub => {
				if locks_opt.is_none() {

				}
				//
				else{
					T::StakingCurrency::lock_sub_amount(Staking_ID, &who, amount, reasons);
				}

			},

			Operate ::Add => {
				if locks_opt.is_none() {
					T::StakingCurrency::set_lock(Staking_ID, &who, amount, reasons);
				}
				//
				else{
					T::StakingCurrency::lock_add_amount(Staking_ID, &who, amount, reasons);
				}
			},
		};

	}


	fn sort_account_by_amount(miner: T::AccountId, mut amount: BalanceOf<T>) -> result::Result<(), DispatchError> {

		let mut old_list = <RecommendList<T>>::get();

		let mut miner_old_info: Option<(T::AccountId, BalanceOf<T>)> = None;

		if let Some(pos) = old_list.iter().position(|h| h.0 == miner.clone()) {

			miner_old_info = Some(old_list.remove(pos));

		}

		if miner_old_info.is_some() {

			let old_amount = miner_old_info.clone().unwrap().1;

			ensure!(T::StakingCurrency::can_reserve(&miner, amount), Error::<T>::AmountNotEnough);

			T::StakingCurrency::unreserve(&miner, old_amount);

			amount = amount + old_amount;

		}

		if old_list.len() == 0 {

			Self::sort_after(miner, amount, 0, old_list)?;
		}

		else {
			let mut index = 0;
			for i in old_list.iter() {
				if i.1 >= amount {
					index += 1;
				}
				else {
					break;
				}
			}

			Self::sort_after(miner, amount, index, old_list)?;

		}

		Ok(())

	}


	fn update_staking_info(miner: T::AccountId, staker: T::AccountId, operate: Operate , amount_opt: Option<BalanceOf<T>>, is_slash: bool) -> DispatchResult {

		ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);

		let mut amount: BalanceOf<T>;


		if let Some(pos) = Self::staker_pos(miner.clone(), staker.clone()) {

			let mut staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();

			let mut staker_info = staking_info.others.remove(pos);

			if amount_opt.is_none() {
				amount = staker_info.1.clone()
			}
			else {
				amount = amount_opt.unwrap()
			}

			match  operate {

				Operate ::Add => {
					let bond = staker_info.1.clone();
					let now_bond = bond.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
					let total_staking = staking_info.total_staking;
					let now_staking = total_staking.checked_add(&amount).ok_or(Error::<T>::Overflow)?;
					T::StakingCurrency::reserve(&staker, amount)?;

					staker_info.1 = now_bond;

					staking_info.total_staking = now_staking;
				},

				_ => {
					let bond = staker_info.1.clone();
					let now_bond = bond.checked_sub(&amount).ok_or(Error::<T>::Overflow)?;
					let total_staking = staking_info.total_staking;
					let now_staking = total_staking.checked_sub(&amount).ok_or(Error::<T>::Overflow)?;

					T::StakingCurrency::unreserve(&staker, amount);

					let now = Self::now();
					let expire = now.saturating_add(T::StakingLockExpire::get());
					Self::lock_add_amount(staker.clone(), amount, expire);

					staker_info.1 = now_bond;

					staking_info.total_staking = now_staking;

				},

			}

			if staker_info.1 == <BalanceOf<T>>::from(0u32) {
				if is_slash {

					T::StakingSlash::on_unbalanced(T::StakingCurrency::slash_reserved(&staker, staker_info.2.clone()).0);
				}

				else {
					T::StakingCurrency::unreserve(&staker, staker_info.2.clone());

				}
				Self::staker_remove_miner(staker.clone(), miner.clone());

			}

			else{
				staking_info.others.push(staker_info);

			}

			<StakingInfoOf<T>>::insert(&miner, staking_info);


		} else {
			return Err(Error::<T>::NotYourStaker)?;

		}

		Ok(())

	}


}


decl_error! {
    /// Error for the ipse module.
    pub enum Error for Module<T: Trait> {
    	/// the numeric id is in using.
    	NumericIdInUsing,
    	/// the miner already register.
		AlreadyRegister,
		/// the miner is not register.
		NotRegister,
		/// plot size should not 0.
		PlotSizeIsZero,
		/// in chill time.
		ChillTime,
		/// not in chill time.
		NotChillTime,
		/// miner already stop mining.
		AlreadyStopMining,
		/// not the staker of this miner.
		NotYourStaker,
		/// the user already staking.
		AlreadyStaking,
		/// over flow.
		Overflow,
		/// the satkers number of this miner is up the max value.
		StakerNumberToMax,
		/// amount not enough.
		AmountNotEnough,
		/// not in the recommend list.
		NotInList,
		/// you did not stop mining.
		MiningNotStop,
		/// you should add the amount.
		AmountTooLow,
		/// you staking amount too low
		StakingAmountooLow,
	}
}
