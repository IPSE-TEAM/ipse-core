#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;
use crate::poc_staking as staking;
use crate::poc_staking::AccountIdOfPid;
use node_primitives::KIB;
// use num_traits::CheckedDiv;
use sp_std::convert::{TryInto,TryFrom, Into};

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage, decl_error,
    ensure,
    dispatch::{DispatchResult, DispatchError}, debug,
    weights::Weight, traits::{Get, Currency, Imbalance, OnUnbalanced, ReservableCurrency},
    IterableStorageMap,
    StorageMap, StorageValue,
};
use system::{ensure_signed};
use sp_runtime::{traits::{SaturatedConversion, Saturating, CheckedDiv, CheckedAdd, CheckedSub}, Percent};
use sp_std::vec::Vec;
use sp_std::vec;
use sp_std::result;
use pallet_treasury as treasury;

use crate::ipse_traits::PocHandler;

use conjugate_poc::{poc_hashing::{calculate_scoop, find_best_deadline_rust}, nonce::noncegen_rust};

use crate::constants::{time::{MILLISECS_PER_BLOCK, DAYS}};

pub const YEAR: u32 = 365*DAYS;

type BalanceOf<T> =
	<<T as staking::Trait>::StakingCurrency as Currency<<T as system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> =
	<<T as staking::Trait>::StakingCurrency as Currency<<T as system::Trait>::AccountId>>::PositiveImbalance;

pub trait Trait: system::Trait + timestamp::Trait + treasury::Trait + staking::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// 多少个区块高度挖一次矿
    type MiningDuration: Get<u64>;

    type PocAddOrigin: OnUnbalanced<PositiveImbalanceOf<Self>>;

    /// GENESIS_BASE_TARGET tib为单位
    type GENESIS_BASE_TARGET: Get<u64>;

    type TotalMiningReward: Get<BalanceOf<Self>>;

}


#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MiningInfo<AccountId> {
    // when miner is None, it means Treasury
    pub miner: Option<AccountId>,
    pub best_dl: u64,
    pub mining_time: u64,
    // the block height of mining success
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
        ///timestamp of last mining
//         pub LastMiningTs get(fn lts): u64;
        /// info of base_target and difficulty
        pub TargetInfo get(fn target_info): Vec<Difficulty>;

        /// deadline info of mining success
        pub DlInfo get(fn dl_info): Vec<MiningInfo<T::AccountId>>;

        /// 矿工的挖矿记录
        pub History get(fn history): map hasher(twox_64_concat) T::AccountId => Option<MiningHistory<BalanceOf<T>, T::BlockNumber>>;

        /// (block_num, account_id，deadline, target, base_target)
        pub Test get(fn test): Vec<(T::BlockNumber)>;

        /// 用户的详细奖励记录
		pub UserRewardHistory get(fn user_reward_history): map hasher(twox_64_concat) T::AccountId => Vec<(T::BlockNumber, BalanceOf<T>)>;



    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId
    {
        Minning(AccountId, bool),
        Verify(AccountId, bool),
        HeightTooLow(AccountId, u64, u64, u64),
        NotBestDeadline(AccountId, u64, u64, u64),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {

     	type Error = Error<T>;

        fn deposit_event() = default;
        /// 多少个区块poc挖一次矿
        const MiningDuration: u64 = T::MiningDuration::get();
        const GENESIS_BASE_TARGET: u64 = T::GENESIS_BASE_TARGET::get();
        /// poc总共挖矿奖励
        const TotalMiningReward: BalanceOf<T> = T::TotalMiningReward::get();


		/// 验证
        #[weight = 1000]
        fn verify_deadline(origin, account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {

            let miner = ensure_signed(origin)?;

            ensure!(<AccountIdOfPid<T>>::contains_key(account_id as u128), Error::<T>::PidErr);

            let is_ok = Self::verify_dl(account_id, height, sig, nonce, deadline).0;
            Self::deposit_event(RawEvent::Verify(miner, is_ok));
            Ok(())
        }


		/// 挖矿
		#[weight = 10_000]
        fn mining(origin, account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {

            let miner = ensure_signed(origin)?;

            //必须是注册过的矿工才能挖矿
            ensure!(<staking::Module<T>>::is_can_mining(miner.clone())?, Error::<T>::NotRegister);

			let real_pid = <staking::Module<T>>::disk_of(&miner).unwrap().numeric_id;

			ensure!(real_pid == account_id.into(), Error::<T>::PidErr);

            let current_block = <system::Module<T>>::block_number().saturated_into::<u64>();

            debug::info!("starting Verify Deadline !!!");

			// 必须在同一周期 并且提交的时间比处理的时间迟
            if !(current_block - height <= T::MiningDuration::get() && current_block >= height)
            {
                debug::info!("请求数据的区块是：{:?}, 提交挖矿的区块是: {:?}, 提交的deadline是: {:?}", height, current_block, deadline);

				Self::deposit_event(RawEvent::HeightTooLow(miner.clone(), current_block, height, deadline));

				return Err(Error::<T>::HeightNotInDuration)?;
            }

			// 获取区块高度和最佳的deadline
            let dl = Self::dl_info();
            let (block, best_dl) = if let Some(dl_info) = dl.iter().last() {
                (dl_info.clone().block, dl_info.best_dl)
            } else {
                (0, core::u64::MAX)
            };


            // Someone(miner) has mined a better deadline at this mining cycle before.
            // 如果这个块已经有比较好的deadline 那么就终止执行
            if best_dl <= deadline && current_block == block {

                debug::info!("Some miner has mined a better deadline at this mining cycle.  height = {} !", height);

                Self::deposit_event(RawEvent::NotBestDeadline(miner.clone(), current_block, height, deadline));

                return Err(Error::<T>::NotBestDeadline)?;
            }

            #[cfg(feature = "std")]
            use std::time::{Duration, SystemTime};

			#[cfg(feature = "std")]
            let start = SystemTime::now();

            let verify_ok = Self::verify_dl(account_id, height, sig, nonce, deadline);

			#[cfg(feature = "std")]
            let end = SystemTime::now();

            #[cfg(feature = "std")]
            debug::info!("挖矿验证开始的时间是: {:?}, 结束的时间是： {:?}", start, end);

            if verify_ok.0 {
                // delete the old deadline in this mining cycle
                // append a better deadline

                let last_block = Self::get_last_miner_mining_block();

                let mining_time: u64;
                if last_block == 0 {
                	mining_time =  MILLISECS_PER_BLOCK;
                	}

                else {
                	mining_time = (current_block - last_block) * MILLISECS_PER_BLOCK;
                	}

                // 这里保证了这个块的dl_info的最后一个总是最优解
                if current_block == block {

                    DlInfo::<T>::mutate(|dl| dl.pop());

                }

                // 上次出块与本次出块的时间间隔
                Self::append_dl_info(MiningInfo{
                        miner: Some(miner.clone()),
                        best_dl: deadline,
                        block: current_block,
                        mining_time
                    });
//                 DlInfo::<T>::mutate(|dl| dl.push(
//                     MiningInfo{
//                         miner: Some(miner.clone()),
//                         best_dl: deadline,
//                         block: current_block,
//                         mining_time
//                     }));

//                 LastMiningTs::mutate( |ts| *ts = current_block * MILLISECS_PER_BLOCK);
            }

            else {
				debug::info!("验证没有通过! deadline = {:?}, target = {:?}, base_target = {:?}", verify_ok.1 / verify_ok.2, verify_ok.1, verify_ok.2);
            }

            debug::info!("verify result: {}", verify_ok.0);
            Self::deposit_event(RawEvent::Minning(miner, verify_ok.0));

            Ok(())
        }

        fn on_initialize(n: T::BlockNumber) -> Weight{

            if n == T::BlockNumber::from(1u32) {
            	let now = Self::get_now_ts();

//                LastMiningTs::put(now);

               TargetInfo::mutate(|target| target.push(
                    Difficulty{
                        base_target: T::GENESIS_BASE_TARGET::get(),
                        net_difficulty: 1,
                        block: 1,
                    }));
            }
            0
        }

        fn on_finalize(n: T::BlockNumber) {
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

			debug::info!("本次挖矿总奖励是： {:?}", reward);

			// 10个块调整挖矿难度
            if current_block%10 == 0 {
                Self::adjust_difficulty(current_block);
            }


				// 每个块都有出块机会
//             if current_block%Self::get_mining_duration().unwrap() == 0 {

				// 如果这个块有poc出块 那么就说明有用户挖矿
            	if current_block == last_mining_block {

             		if let Some(miner_info) = Self::dl_info().last() {
 						let miner: Option<T::AccountId> = miner_info.clone().miner;
 						if miner.is_some() {
 							Self::reward_miner(miner.unwrap(), reward);
 							debug::info!("<<REWARD>> miner on block {}, last_mining_block {}", current_block, last_mining_block);
 						}

             		}

            	}

            	else {
            		Self::treasury_minning(current_block);
            		Self::reward_treasury(reward);
            	}

//             }
        }

     }
}

impl<T: Trait> Module<T> {

    fn adjust_difficulty(block: u64) {
        debug::info!("[ADJUST] difficulty on block {}", block);
        let base_target_avg = Self::get_base_target_avg();
//         let mining_time_avg = Self::get_mining_time_avg();
        let mining_time_avg = MILLISECS_PER_BLOCK;
        debug::info!("BASE_TARGET_AVG = {},  MINING_TIME_AVG = {}", base_target_avg, mining_time_avg);
        // base_target跟出块的平均时间成正比
        if mining_time_avg >= MILLISECS_PER_BLOCK * 2 * 2 / 3 {
            let new = base_target_avg.saturating_mul(2);
            debug::info!("[DIFFICULTY] make easier = {}", new);
            TargetInfo::mutate(|target| target.push(
                Difficulty{
                    block,
                    base_target: new,
                    net_difficulty: T::GENESIS_BASE_TARGET::get() / new,

                }));
        }

        else if mining_time_avg <= MILLISECS_PER_BLOCK * 2 / 3 {
            let new = base_target_avg / 2;
            debug::info!("[DIFFICULTY] make more difficult = {}", new);
            TargetInfo::mutate(|target| target.push(
                Difficulty{
                    block,
                    base_target: new,
                    net_difficulty: T::GENESIS_BASE_TARGET::get() / new,
                }));
        }

        else {
        	let new = base_target_avg;
            debug::info!("[DIFFICULTY]  = {}", new);
            TargetInfo::mutate(|target| target.push(
                Difficulty{
                    block,
                    base_target: new,
                    net_difficulty: T::GENESIS_BASE_TARGET::get() / new,
                }));
        }


    }


	fn treasury_minning(current_block: u64) {

		Self::append_dl_info(MiningInfo{
				miner: None,
				best_dl: core::u64::MAX,

				mining_time: MILLISECS_PER_BLOCK,
				block: current_block, // 记录当前区块
			});
// 		<DlInfo<T>>::mutate(|dl| dl.push(
// 			MiningInfo{
// 				miner: None,
// 				best_dl: core::u64::MAX,
//
// 				mining_time: MILLISECS_PER_BLOCK,
// 				block: current_block, // 记录当前区块
// 			}));
		debug::info!("<<REWARD>> treasury on block {}", current_block);
// 		<Test<T>>::mutate(|h| h.push(T::BlockNumber::from(current_block as u32)));
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

    // 获取上次矿工挖矿的区块
    fn get_last_miner_mining_block() -> u64 {


		let mut dl = <DlInfo<T>>::get();

		let dl_cp = dl.clone();

		// 获取现在的区块
		let now = <staking::Module<T>>::now().saturated_into::<u64>();
		let len = dl_cp.len();

        for j in 0..len {

			let i = dl.get(len - 1 - j).unwrap();

			let mut index = 1;

			// 不能超过10个周期
			if index >= 10 {
				break;
			}

        	if i.miner.is_some() {

        		// 不在本区块
        		if (i.block != now) {
        			debug::info!("矿工挖出来的最后的区块是:{:?}", i);
        			return i.block;

        		}
        	}

        	index += 1;

        }

        0

    }




    fn get_last_adjust_block() -> u64 {
        let tis = Self::target_info();
        if let Some(ti) = tis.iter().last() {
            ti.block
        } else {
            0
        }
    }


    fn get_now_ts() -> u64 {

        let now = <staking::Module<T>>::now().saturated_into::<u64>();

		now * MILLISECS_PER_BLOCK

    }


    fn get_base_target_avg() -> u64 {
        let ti = Self::target_info();
        let mut iter = ti.iter().rev();
        let mut total = 0_u64;
        let mut count = 0_u64;
        while let Some(target) = iter.next() {
            if count == 24 {
                break;
            }

            total = total.saturating_add(target.base_target);

            count += 1;
        }

        if count == 0 { T::GENESIS_BASE_TARGET::get() } else { total/count }
    }


	/// 平均的出块时间
    fn get_mining_time_avg() -> u64 {
        let dl = Self::dl_info();
        let mut iter = dl.iter().rev();
        let mut total = 0_u64;
        let mut count = 0_u64;
        let mut real_count = 0_u64;

        while let Some(dl) = iter.next() {
        	if count == 24 {
                break;
				}
        	if dl.miner.is_some() {

				total += dl.mining_time;
				real_count += 1;
        	}

        	count += 1;

        }

        if real_count <=  5 { MILLISECS_PER_BLOCK } else { total/real_count }
    }


    fn verify_dl(account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> (bool, u64, u64) {
        let scoop_data = calculate_scoop(height, &sig) as u64;
        debug::info!("scoop_data: {:?}",scoop_data);
        debug::info!("sig: {:?}",sig);

        let mut cache = vec![0_u8; 262144];

        noncegen_rust(&mut cache[..], account_id, nonce, 1);

        let mirror_scoop_data = Self::gen_mirror_scoop_data(scoop_data, cache);

        let (target, _) = find_best_deadline_rust(mirror_scoop_data.as_ref(), 1, &sig);
        debug::info!("target: {:?}",target);
        let base_target = Self::get_current_base_target();
        let deadline_ = target/base_target;
        debug::info!("deadline: {:?}",deadline_);
        (deadline == target/base_target, target, base_target)
    }


    fn gen_mirror_scoop_data(scoop_data: u64, cache: Vec<u8>) -> Vec<u8>{
        let addr = 64 * scoop_data as usize;
        let mirror_scoop = 4095 - scoop_data as usize;
        let mirror_addr = 64 * mirror_scoop as usize;
        let mut mirror_scoop_data = vec![0_u8; 64];
        mirror_scoop_data[0..32].clone_from_slice(&cache[addr..addr + 32]);
        mirror_scoop_data[32..64].clone_from_slice(&cache[mirror_addr + 32..mirror_addr + 64]);
        mirror_scoop_data
    }


    fn get_mining_duration() -> result::Result<u64, DispatchError> {
		if T::MiningDuration::get() == 0u64{
			return Err(Error::<T>::DivZero)?;

		}
		else{
			Ok(T::MiningDuration::get())
		}
    }


    /// 获取国库id
    fn get_treasury_id() -> T::AccountId {
    	<treasury::Module<T>>::account_id()
    }


    /// 获取本次奖励
    fn get_reward_amount() -> result::Result<BalanceOf<T>, DispatchError> {
    	let now = <staking::Module<T>>::now();

    	let year = now.checked_div(&T::BlockNumber::from(YEAR)).ok_or(Error::<T>::DivZero)?;
    	let duration = year / T::BlockNumber::from(2u32);

    	let duration = <<T as system::Trait>::BlockNumber as TryInto<u32>>::try_into(duration).map_err(|_| Error::<T>::ConvertErr)?;

		let n_opt = 2u32.checked_pow(duration + 1u32);

		let reward: BalanceOf<T>;

		if n_opt.is_some() {

			let n = <BalanceOf<T>>::from(n_opt.unwrap());

			reward = T::TotalMiningReward::get() / n / Self::block_convert_to_balance(T::BlockNumber::from(YEAR))?;

			Ok(reward)
		}

		else{
			Ok(<BalanceOf<T>>::from(0u32))
		}
    }


    /// block_num类型数据转变成balance
    fn block_convert_to_balance(n: T::BlockNumber) -> result::Result<BalanceOf<T>, DispatchError> {
		let n_u = <<T as system::Trait>::BlockNumber as TryInto<u32>>::try_into(n).map_err(|_| Error::<T>::ConvertErr)?;
		let n_b = <	BalanceOf<T> as TryFrom::<u32>>::try_from(n_u).map_err(|_| Error::<T>::ConvertErr)?;
		Ok(n_b)
    }


    /// 奖励国库
    fn reward_treasury(reward: BalanceOf<T>) {
    	let account_id = Self::get_treasury_id();
    	T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(&account_id, reward));
    }


    /// 奖励矿工
    fn reward_miner(miner: T::AccountId, mut reward: BalanceOf<T>) -> DispatchResult {
		// 获取自己的本机容量
		let machine_info = <staking::Module<T>>::disk_of(&miner).ok_or(Error::<T>::NotRegister)?;
		let disk = machine_info.clone().plot_size;
		let update_time = machine_info.clone().update_time;

		let now = <staking::Module<T>>::now();

		// todo 假设一个块挖一次（也有可能几个块挖一次）
		let net_mining_num = (now - update_time).saturated_into::<u64>();

		let mut miner_mining_num = match <History<T>>::get(&miner) {
			Some(h) => {h.total_num + 1u64},
			None => 0u64,
		};

		// 判断自己的挖矿概率是否达标
		if disk.saturating_mul(net_mining_num) >= Self::get_total_capacity().saturating_mul(miner_mining_num) {
			debug::info!("矿工抵押达标！");
			reward = reward;
			Self::reward_staker(miner.clone(), reward);
		}

		// 如果不达标 拿百分之10的奖励
		else {
			debug::info!("矿工抵押不达标！");
			reward = Percent::from_percent(10) * reward;
			Self::reward_staker(miner.clone(), reward);
		}

		debug::info!("本次挖矿实际奖励是：{:?}", reward);

		miner_mining_num += 1;

		let history_opt = <History<T>>::get(&miner);

		if history_opt.is_some() {
			debug::info!("不是第一次挖矿！");
			let mut his = history_opt.unwrap();
			his.total_num = miner_mining_num;
			his.history.push((now, reward));
			/// 只存储最新的100条记录
			if his.history.len() >= 100 {
				let mut old_history = his.history.clone();
				let new_history = old_history.split_off(1);
				his.history = new_history;
			}
			<History<T>>::insert(miner.clone(), his);

		}

		else {
			debug::info!("第一次挖矿！");
			let history = vec![(now, reward)];
			<History<T>>::insert(miner.clone(), MiningHistory {
				total_num: miner_mining_num,
				history: history,
			});

		}

		Ok(())
    }

   	// 奖励每一个成员（抵押者）
   	fn reward_staker(miner: T::AccountId, reward: BalanceOf<T>) -> DispatchResult {

   		let now = <staking::Module<T>>::now();

		let staking_info = <staking::Module<T>>::staking_info_of(&miner).ok_or(Error::<T>::NotRegister)?;
		let stakers = staking_info.clone().others;
		if stakers.len() == 0 {
			T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(&miner, reward));
			Self::update_reword_history(miner.clone(), reward, now);

		}

		else {
			// 奖励矿工
			let miner_reward = staking_info.clone().miner_proportion * reward;
			T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(&miner, miner_reward));
			Self::update_reword_history(miner.clone(), miner_reward, now);
			let stakers_reward = reward - miner_reward;
			let total_staking = staking_info.clone().total_staking;
			for staker_info in stakers.iter() {
				let staker_reward = stakers_reward.saturating_mul(staker_info.clone().1).checked_div(&total_staking).ok_or(Error::<T>::DivZero)?;
				T::PocAddOrigin::on_unbalanced(T::StakingCurrency::deposit_creating(&staker_info.clone().0, staker_reward));
				Self::update_reword_history(staker_info.clone().0, staker_reward, now);
			}
		}

		Ok(())
   	}

    /// 获取全网容量
    fn get_total_capacity() -> KIB {
		0 as KIB
    }

    /// 更新用户的奖励记录
    fn update_reword_history(account_id: T::AccountId, amount: BalanceOf<T>, block_num: T::BlockNumber) {

    	let mut reward_history = <UserRewardHistory<T>>::get(account_id.clone());

    	reward_history.push((block_num, amount));

		/// 奖励记录限制在100条以内
    	if reward_history.len() >= 100 {
    		let mut old_history = reward_history.clone();
    		let new_history = old_history.split_off(1);
    		<UserRewardHistory<T>>::insert(account_id, new_history);
    	}

    	else {
    		<UserRewardHistory<T>>::insert(account_id, reward_history);
    	}



    }

    // 添加dl_info(最高数据量有所限制 目前设置500条)
    fn append_dl_info(dl_info: MiningInfo<T::AccountId>) {
    	// 获取dl_info
    	let mut old_dl_info_vec = <DlInfo<T>>::get();
    	let len = old_dl_info_vec.len();

    	old_dl_info_vec.push(dl_info);

    	if len >= 500 {

    		let new_dl_info = old_dl_info_vec.split_off(len - 500);
    		old_dl_info_vec = new_dl_info;
    	}

    	<DlInfo<T>>::put(old_dl_info_vec);


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
		/// 除以0错误
		DivZero,
		/// 没有注册过
		NotRegister,
		/// 提交的p盘id错误
		PidErr,
		/// 数据转换错误
		ConvertErr,
		/// 提交的高度不在当前周期
		HeightNotInDuration,
		/// 不是最优的deadline
		NotBestDeadline,

    }
}
