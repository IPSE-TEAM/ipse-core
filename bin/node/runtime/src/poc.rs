#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;

use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    weights::{SimpleDispatchInfo, DispatchInfo, DispatchClass, ClassifyDispatch, WeighData, Weight, PaysFee},
};
use system::{ensure_signed};
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec::Vec;
use sp_std::vec;
use log::info;

use conjugate_poc::{poc_hashing::{calculate_scoop, find_best_deadline_rust}, nonce::noncegen_rust};

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as PoC {
        // timestamp of last mining
        LastMiningTs get(lts): u64;
        // key: Block Number of last adjusting difficulty
        // value: (base_target, net_difficulty)
        TargetInfo get(target_info): map hasher(blake2_128_concat) u64 = 0 => (u64, u64) = (488671834567, 1);
        // deadline info
        // key: Block Number of last mining
        // value: (Current best deadline,  mining_time)
        DlInfo get(dl_info): map hasher(blake2_128_concat) u64 => (u64, u64);
    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId
    {
        MiningSuccess(AccountId),
        VerifyDeadline(AccountId, bool),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FixedNormal(1000)]
        fn verify_deadline(origin, account_id: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            let height = <system::Module<T>>::block_number().saturated_into::<u64>();
            let is_ok = Self::verify_dl(account_id, current_block, sig, nonce, deadline);
            Self::deposit_event(RawEvent::VerifyDeadline(miner, is_ok));
            Ok(())
        }

		#[weight = SimpleDispatchInfo::FixedNormal(10_000)]
        fn mining(origin, account_id: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {
            let miner = ensure_signed(origin)?;

            let current_block = <system::Module<T>>::block_number().saturated_into::<u64>();
            let now = Self::get_now_ts();
            if Self::verify_dl(account_id, current_block, sig, nonce, deadline) {
                let now = Self::get_now_ts();
                let mining_time = now - Self::lts();
                DlInfo::<T>::mutate(current_block, |dl| { *dl = (deadline, mining_time) });
                LastMiningTs::<T>::mutate(|ts| *ts = now );
                info!("reward!!!");
                Self::deposit_event(RawEvent::MiningSuccess(miner));
            }

        }

        #[weight = SimpleDispatchInfo::FixedNormal(1000)]
        fn on_initialize(n: T::BlockNumber) {
            let n = n.saturated_into::<u64>();
            if n == 0 {
               let now = Self::get_now_ts();
               LastMiningTs::<T>::put(now);
            }
        }

        #[weight = SimpleDispatchInfo::FixedNormal(2000)]
        fn on_finalize(n: T::BlockNumber) {
            let current_block = n.saturated_into::<u64>();
            let last_block = Self::get_last_mining_block();

            if current_block - last_block == 3 {
                Self::adjust_difficulty();
            }
            if current_block - last_block > 3 {
                info!("no new deadline, reward treasury")
            }
        }

     }
}

impl<T: Trait> Module<T> {

    fn adjust_difficulty() {
        info!("adjust base_target and net_difficulty");
    }

    fn get_last_dl_info() -> Option<&(u64, (u64, u64))>{
        DlInfo::<T>::iter().last()
    }

    fn get_last_target_info() -> Option<&(u64, (u64, u64))>{
        TargetInfo::<T>::iter().last()
    }

    fn get_current_base_target() -> u64 {
        let (_, (base_target, _)) = TargetInfo::<T>::iter().last().unwrap();
        base_target
    }

    fn get_last_mining_block() -> u64 {
        let (block, _) = DlInfo::<T>::iter().last().unwrap();
        block
    }

    fn get_last_adjust_block() -> u64 {
        let (block, _) = TargetInfo::<T>::iter().last().unwrap();
        block
    }

    fn get_now_ts() -> u64 {
        let now = <timestamp::Module<T>>::get();
        <T::Moment as TryInto<u64>>::try_into(now).unwrap()
    }

    fn verify_dl(account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> bool {
        let scoop_data = calculate_scoop(height, &sig) as u64;
        info!("scoop_data: {:?}",scoop_data);
        info!("sig: {:?}",sig);

        let mut cache = vec![0_u8; 262144];
        noncegen_rust(&mut cache[..], account_id, nonce, 1);
        let mirror_scoop_data = Self::gen_mirror_scoop_data(scoop_data, cache);

        let (target, _) = find_best_deadline_rust(mirror_scoop_data.as_ref(), 1, &sig);
        info!("target: {:?}",target);
        let base_target = Self::get_current_base_target();
        let deadline_ = target/base_target;
        info!("deadline: {:?}",deadline_);
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