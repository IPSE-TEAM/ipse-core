#![cfg_attr(not(feature = "std"), no_std)]

extern crate frame_system as system;

use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
};
use system::{ensure_signed};
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec::Vec;
use sp_std::vec;
use log::info;

use conjugate_poc::{poc_hashing::{calculate_scoop, find_best_deadline_rust}, nonce::noncegen_rust};

pub trait Trait: system::Trait + pallet_timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as PoC {
        BaseTarget get(base_target): u64 = 488671834567;
        /// Block Number of last mining
        MiningBlockNum get(mining_block): u64 = 0;
        /// Block Number of last adjusting difficulty
        AdjustBlockNum get(adjust_block): u64 = 0;
        /// Current best deadline
        BestDeadline get(best_dl): u64;
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

        fn mining(origin, account_id: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {
            let miner = ensure_signed(origin)?;

            let current_block = <system::Module<T>>::block_number().saturated_into::<u64>();
            let last_mining_block = Self::mining_block();
            if current_block - last_mining_block >= 3 {
                if Self::verify_dl(account_id, current_block, sig, nonce, deadline) {
                    info!("reward!!!");
                }
            }
        }

        fn verify_deadline(origin, account_id: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> DispatchResult {
            let miner = ensure_signed(origin)?;
            let height = <system::Module<T>>::block_number().saturated_into::<u64>();
            let is_ok = Self::verify_dl(account_id, current_block, sig, nonce, deadline);
            Self::deposit_event(RawEvent::VerifyDeadline(miner, is_ok));
            Ok(())
        }

        fn on_finalize(n: T::BlockNumber) {
            let current_block = n.saturated_into::<u64>();
            let last_adjust_block = Self::adjust_block();

            if current_block - last_adjust_block >= 3 {
                    info!("reward!!!");
            }

            // adjust base_target and difficulty
            if current_block - last_adjust_block >= 30 {
                Self::adjust_difficulty();
                <AdjustBlockNum<T>>::mutate(|num| {
                    *num = current_block;
                });
            }
        }

     }
}

impl<T: Trait> Module<T> {

    fn verify_dl(account_id: u64, height: u64, sig: [u8; 32], nonce: u64, deadline: u64) -> bool {
        let scoop_data = calculate_scoop(height, &sig) as u64;
        info!("scoop_data: {:?}",scoop_data);
        info!("sig: {:?}",sig);

        let mut cache = vec![0_u8; 262144];
        noncegen_rust(&mut cache[..], account_id, nonce, 1);
        let mirror_scoop_data = Self::gen_mirror_scoop_data(scoop_data, cache);

        let (target, _) = find_best_deadline_rust(mirror_scoop_data.as_ref(), 1, &sig);
        info!("target: {:?}",target);
        let base_target = Self::base_target();
        let deadline_ = target/base_target;
        info!("deadline: {:?}",deadline_);
        deadline == target/base_target
    }

    fn adjust_difficulty() {

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