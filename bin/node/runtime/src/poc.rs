#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage, decl_error,
    dispatch::{DispatchResult, DispatchError}, debug,
    weights::Weight, traits::{Get, Currency, Imbalance, OnUnbalanced, ReservableCurrency},
};
use system::{ensure_signed};
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec::Vec;
use sp_std::vec;
use sp_std::result;
use pallet_treasury as treasury;

use conjugate_poc::{poc_hashing::{calculate_scoop, find_best_deadline_rust}, nonce::noncegen_rust};

use crate::constants::time::MILLISECS_PER_BLOCK;


type BalanceOf<T> =
	<<T as Trait>::PocCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type PositiveImbalanceOf<T> =
	<<T as Trait>::PocCurrency as Currency<<T as frame_system::Trait>::AccountId>>::PositiveImbalance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::PocCurrency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait + timestamp::Trait + treasury::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    /// 多少个区块高度挖一次矿
    type MiningDuration: Get<u64>;

    type PocAddOrigin: OnUnbalanced<PositiveImbalanceOf<Self>>;

    type PocCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    /// GENESIS_BASE_TARGET tib为单位
    type GENESIS_BASE_TARGET: Get<u64>;
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
pub struct Difficulty {
    pub base_target: u64,
    pub net_difficulty: u64,
    // the block height of adjust difficulty
    pub block: u64,
}

decl_storage! {
    trait Store for Module<T: Trait> as PoC {
        // timestamp of last mining
        pub LastMiningTs get(fn lts): u64;
        // info of base_target and difficulty
        pub TargetInfo get(fn target_info): Vec<Difficulty>;
        // deadline info of mining success
        pub DlInfo get(fn dl_info): Vec<MiningInfo<T::AccountId>>;

    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId
    {
        Minning(AccountId, bool),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {

     	type Error = Error<T>;
        fn deposit_event() = default;
        /// 多少个区块poc挖一次矿
        const MiningDuration: u64 = T::MiningDuration::get();
        const GENESIS_BASE_TARGET: u64 = T::GENESIS_BASE_TARGET::get();


		/// 验证
        #[weight = 1000]
        fn verify_deadline(origin, account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            let is_ok = Self::verify_dl(account_id, height, sig, nonce, deadline);
            Self::deposit_event(RawEvent::Minning(miner, is_ok));
            Ok(())
        }


		/// 挖矿
		#[weight = 10_000]
        fn mining(origin, account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            let current_block = <system::Module<T>>::block_number().saturated_into::<u64>();

            debug::info!("starting Verify Deadline !!!");

            // illegal block height
            // 高度大于当前即非法
            if height > current_block {
                debug::info!("illegal height = {} !", height);
                Self::deposit_event(RawEvent::Minning(miner, false));
                return Ok(())
            }

			// 获取区块高度和最佳的deadline
            let dl = Self::dl_info();
            let (block, best_dl) = if let Some(dl_info) = dl.iter().last() {
                (dl_info.clone().block, dl_info.best_dl)
            } else {
                (0, core::u64::MAX)
            };

            // the verifying expired
            if height/Self::get_mining_duration()? - block/Self::get_mining_duration()? > 1 {  // 挖矿时候提交的高度不能太偏离最后一个dl_info的 高度
                debug::info!("verifying expired height = {} !", height);
                Self::deposit_event(RawEvent::Minning(miner, false));
                return Ok(())
            }

            // Someone(miner) has mined a better deadline at this mining cycle before.
            // 如果之前已经有比较好的deadline 那么就终止执行
            if best_dl <= deadline && current_block/Self::get_mining_duration()? == block/Self::get_mining_duration()? {
                debug::info!("Some miner has mined a better deadline at this mining cycle.  height = {} !", height);
                Self::deposit_event(RawEvent::Minning(miner, false));
                return Ok(())
            }
            let verify_ok = Self::verify_dl(account_id, height, sig, nonce, deadline);

            if verify_ok {
                // delete the old deadline in this mining cycle
                // 这里保证了dl_info的最后一个总是最优解
                if current_block/Self::get_mining_duration()? == block/Self::get_mining_duration()? {
                    DlInfo::<T>::mutate(|dl| dl.pop());
                }

                // append a better deadline
                let now = Self::get_now_ts(current_block);
                // 上次出块与本次出块的时间间隔
                let mining_time = now - Self::lts();
                DlInfo::<T>::mutate(|dl| dl.push(
                    MiningInfo{
                        miner: Some(miner.clone()),
                        best_dl: deadline,
                        block: current_block,
                        mining_time
                    }));
                LastMiningTs::mutate( |ts| *ts = now);
            };

            debug::info!("verify result: {}", verify_ok);
            Self::deposit_event(RawEvent::Minning(miner, verify_ok));

            Ok(())
        }

        fn on_initialize(n: T::BlockNumber) -> Weight{
            let n = n.saturated_into::<u64>();
            if n == 1 {
               LastMiningTs::put(0);
               TargetInfo::mutate(|target| target.push(
                    Difficulty{
                        base_target: T::GENESIS_BASE_TARGET::get(),
                        net_difficulty: 1,
                        block: 0,
                    }));
            }
            0
        }

        fn on_finalize(n: T::BlockNumber) {
            let current_block = n.saturated_into::<u64>();
            let last_mining_block = Self::get_last_mining_block();

            debug::info!("current-block = {}, last-mining-block = {}", current_block, last_mining_block);

			// 调整挖矿难度
            if current_block%10 == 0 {
                Self::adjust_difficulty(current_block);
            }


			// 每三个区块出一个块
            if current_block%Self::get_mining_duration().unwrap() == 0 {
                if current_block/Self::get_mining_duration().unwrap() - last_mining_block/Self::get_mining_duration().unwrap() <= 1 {
                	let dl = Self::dl_info();

                	if dl.is_empty() || dl.last().unwrap().miner.is_none() {
                		Self::treasury_minning(current_block);
                	}

                	else {
                		debug::info!("<<REWARD>> miner on block {}, last_mining_block {}", current_block, last_mining_block);
                	}

                } else {
                	Self::treasury_minning(current_block);
                }
            }
        }

     }
}

impl<T: Trait> Module<T> {

    fn adjust_difficulty(block: u64) {
        debug::info!("[ADJUST] difficulty on block {}", block);
        let base_target_avg = Self::get_base_target_avg();
        let mining_time_avg = Self::get_mining_time_avg();
        debug::info!("BASE_TARGET_AVG = {},  MINING_TIME_AVG = {}", base_target_avg, mining_time_avg);
        // base_target跟出块的平均时间成正比
        if mining_time_avg >= 16000 {
            let new = base_target_avg.saturating_mul(2);
            debug::info!("[DIFFICULTY] make easier = {}", new);
            TargetInfo::mutate(|target| target.push(
                Difficulty{
                    block,
                    base_target: new,
                    net_difficulty: T::GENESIS_BASE_TARGET::get() / new,
                }));
        }

        else if mining_time_avg <= 8000 {
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
		let now = Self::get_now_ts(current_block);
		<DlInfo<T>>::mutate(|dl| dl.push(
			MiningInfo{
				miner: None,
				best_dl: core::u64::MAX,

				mining_time: 12000,
				block: current_block, // 记录当前区块
			}));
		LastMiningTs::mutate( |ts| *ts = now);
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

    fn get_last_adjust_block() -> u64 {
        let tis = Self::target_info();
        if let Some(ti) = tis.iter().last() {
            ti.block
        } else {
            0
        }
    }

    fn get_now_ts(block_num: u64) -> u64 {
        //let now = <timestamp::Module<T>>::get();
        //<T::Moment as TryInto<u64>>::try_into(now).ok().unwrap()
        block_num * MILLISECS_PER_BLOCK
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
        while let Some(dl) = iter.next() {
            if count == 24 {
                break;
            }
            total += dl.mining_time;
            count += 1;
        }
        if count == 0 { 12000 } else { total/count }
    }

    fn verify_dl(account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> bool {
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
        deadline == target/base_target
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
}

decl_error! {
    /// Error for the ipse module.
    pub enum Error for Module<T: Trait> {
		/// 除以0错误
		DivZero,
    }
}
