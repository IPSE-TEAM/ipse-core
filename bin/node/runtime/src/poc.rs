#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, debug,
    weights::{SimpleDispatchInfo},
};
use system::{ensure_signed};
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec::Vec;
use sp_std::vec;
use sp_std::convert::TryInto;

use conjugate_poc::{poc_hashing::{calculate_scoop, find_best_deadline_rust}, nonce::noncegen_rust};

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

const GENESIS_BASE_TARGET: u64 = 488671834567;

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
        LastMiningTs get(lts): u64;
        // info of base_target and difficulty
        pub TargetInfo get(target_info): Vec<Difficulty>;
        // deadline info of mining success
        pub DlInfo get(dl_info): Vec<MiningInfo<T::AccountId>>;
    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId
    {
        VerifyDeadline(AccountId, bool),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FixedNormal(1000)]
        fn verify_deadline(origin, account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            let is_ok = Self::verify_dl(account_id, height, sig, nonce, deadline);
            Self::deposit_event(RawEvent::VerifyDeadline(miner, is_ok));
            Ok(())
        }

		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        fn mining(origin, account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            let current_block = <system::Module<T>>::block_number().saturated_into::<u64>();

            debug::info!("starting Verify Deadline !!!");

            // illegal block height
            if height > current_block {
                debug::info!("illegal height = {} !", height);
                Self::deposit_event(RawEvent::VerifyDeadline(miner, false));
                return Ok(())
            }

            let dl = Self::dl_info();
            let (block, best_dl) = if let Some(dl_info) = dl.iter().last() {
                (dl_info.clone().block, dl_info.best_dl)
            } else {
                (0, core::u64::MAX)
            };

            // the verifying expired
            if height/3 - block/3 > 1 {
                debug::info!("verifying expired height = {} !", height);
                Self::deposit_event(RawEvent::VerifyDeadline(miner, false));
                return Ok(())
            }
            // Someone(miner) has mined a better deadline at this mining cycle before.
            if best_dl <= deadline && current_block/3 == block/3 {
                debug::info!("Some miner has mined a better deadline at this cycle.  height = {} !", height);
                Self::deposit_event(RawEvent::VerifyDeadline(miner, false));
                return Ok(())
            }
            let verify_ok = Self::verify_dl(account_id, height, sig, nonce, deadline);
            if verify_ok {
                // delete the old deadline in this mining cycle
                if current_block/3 == block/3 {
                    DlInfo::<T>::mutate(|dl| dl.pop());
                }

                // insert a better deadline
                let now = Self::get_now_ts();
                let mining_time = now - Self::lts();
                DlInfo::<T>::mutate(|dl| dl.push(
                    MiningInfo{
                        miner: Some(miner.clone()),
                        best_dl: deadline,
                        block: current_block,
                        mining_time
                    }));
                LastMiningTs::mutate(|ts| *ts = now );
            };

            debug::info!("verify result: {}", verify_ok);
            Self::deposit_event(RawEvent::VerifyDeadline(miner, verify_ok));

            Ok(())
        }

        #[weight = SimpleDispatchInfo::FixedNormal(1000)]
        fn on_initialize(n: T::BlockNumber) {
            let n = n.saturated_into::<u64>();
            if n == 1 {
               let now = Self::get_now_ts();
               LastMiningTs::put(now);
               TargetInfo::mutate(|target| target.push(
                    Difficulty{
                        base_target: GENESIS_BASE_TARGET,
                        net_difficulty: 1,
                        block: 0,
                    }));
            }
        }

        #[weight = SimpleDispatchInfo::FixedNormal(2000)]
        fn on_finalize(n: T::BlockNumber) {
            let current_block = n.saturated_into::<u64>();
            let last_mining_block = Self::get_last_mining_block();

            debug::info!("current-block = {}, last-mining-block = {}", current_block, last_mining_block);

            if current_block%10 == 0 {
                Self::adjust_difficulty(current_block);
            }

            if current_block%3 == 0 {
                if current_block/3 - last_mining_block/3 <= 1 {
                    debug::info!("<<REWARD>> miner on block {}", current_block);
                } else {
                    <DlInfo<T>>::mutate(|dl| dl.push(
                        MiningInfo{
                            miner: None,
                            best_dl: 0,
                            mining_time: 18000,
                            block: current_block,
                        }));
                    debug::info!("<<REWARD>> treasury on block {}", current_block);
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
        if mining_time_avg >= 18000 {
            let new = base_target_avg * 2;
            debug::info!("[DIFFICULTY] make easier = {}", new);
            TargetInfo::mutate(|target| target.push(
                Difficulty{
                    block,
                    base_target: new,
                    net_difficulty: GENESIS_BASE_TARGET / new,
                }));
        }
        if mining_time_avg <= 4000 {
            let new = base_target_avg / 2;
            debug::info!("[DIFFICULTY] make more difficult = {}", new);
            TargetInfo::mutate(|target| target.push(
                Difficulty{
                    block,
                    base_target: new,
                    net_difficulty: GENESIS_BASE_TARGET / new,
                }));
        }
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

    fn get_now_ts() -> u64 {
        let now = <timestamp::Module<T>>::get();
        <T::Moment as TryInto<u64>>::try_into(now).ok().unwrap()
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
            total += target.base_target;
            count += 1;
        }
        if count == 0 { GENESIS_BASE_TARGET } else { total/count }
    }

    fn get_mining_time_avg() -> u64 {
        let dl = Self::dl_info();
        let mut iter = dl.iter().rev();
        let mut total = 0_u64;
        let mut count = 0_u64;
        while let Some(dl) = iter.next() {
            if count == 10 {
                break;
            }
            total += dl.mining_time;
            count += 1;
        }
        if count == 0 { 18000 } else { total/count }
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
}