
extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;
extern crate pallet_balances as balances;

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, debug,
    weights::Weight,
	StorageMap, StorageValue,
	decl_error, ensure,
};

use system::{ensure_signed};
use sp_runtime::{traits::SaturatedConversion, Percent};
use sp_std::vec::Vec;
use sp_std::vec;
use node_primitives::KIB;

pub trait Trait: system::Trait + timestamp::Trait + balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}



#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineInfo<BlockNumber> {

	memery: KIB,
	update_time: BlockNumber,
}


#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct StakingInfo<AccountId, Balance> {

	miner: AccountId,
	miner_portation: Percent,
	total_staking: Balance,
	others: Vec<(AccountId, Balance)>,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Oprate {
	Add,
	Sub,
}

impl Default for Oprate {
	fn default() -> Self {
		Self::Add
	}
}


decl_storage! {
    trait Store for Module<T: Trait> as IpseStakingModule {

		pub MinerInfo get(fn maner_info): map hasher(twox_64_concat) T::AccountId => Option<MachineInfo<T::BlockNumber>>;

		pub IsChillTime get(fn is_chill_time): bool = true;

		pub SatkingInfoOf get(fn stking_info_of): map hasher(twox_64_concat) T::AccountId => Option<StakingInfo<T::AccountId, T::Balance>>;

    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId
    {
        UpdateMachineInfo(AccountId, KIB),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;


		#[weight = 10_000]
		fn register(origin, memery: KIB, miner_portation: Percent) {
			let miner = ensure_signed(origin)?;

			if <MinerInfo<T>>::contains_key(&miner) || <SatkingInfoOf<T>>::contains_key(&miner) {
				return Err(Error::<T>::AlreadyRegister)?;

			}

			let now = Self::now();
			<MinerInfo<T>>::insert(miner.clone(), MachineInfo {
        		memery: memery,
        		update_time: now,

        	});

        	<SatkingInfoOf<T>>::insert(miner.clone(),
        		StakingInfo {

        			miner: miner,
        			miner_portation: miner_portation,
        			total_staking: T::Balance::from(0u32),
        			others: vec![],
        		}
        	);

		}


        #[weight = 10_000]
        fn update_machine_info(origin, memery: KIB) {

        	let miner = ensure_signed(origin)?;

        	ensure!(memery != 0 as KIB, Error::<T>::MemeryEmpty);

        	let now = Self::now();

        	ensure!(<MinerInfo<T>>::contains_key(&miner), Error::<T>::NotRegister);

        	<MinerInfo<T>>::insert(miner.clone(), MachineInfo {
        		memery: memery,
        		update_time: now,

        	});

        	Self::deposit_event(RawEvent::UpdateMachineInfo(miner, memery));

        }


        #[weight = 10_000]
        fn staking(origin, miner: T::AccountId, amount: T::Balance) {

        	let who = ensure_signed(origin)?;

        }

        #[weight = 10_000]
        fn chill(origin, miner: T::AccountId) {

			let who = ensure_signed(origin)?;

        }

        #[weight = 10_000]
        fn update_staking(origin, miner: T::AccountId, oprate: Oprate, amount: T::Balance) {

        	let who = ensure_signed(origin)?;

        }


        #[weight = 10_000]
        fn update_portation(origin, portation: Percent) {

        	let miner = ensure_signed(origin)?;

        }


        #[weight = 10_000]
        fn remove_staker(origin, staker: T::AccountId) {


        }


//		fn on_initialize(n: T::BlockNumber) {
//
//        }

     }
}

impl<T: Trait> Module<T> {

	fn get_era() -> u64 {
		0u64

	}

	fn now() -> T::BlockNumber {

		<system::Module<T>>::block_number()
	}

}

decl_error! {
    /// Error for the ipse module.
    pub enum Error for Module<T: Trait> {
		AlreadyRegister,
		NotRegister,
		MemeryEmpty,
    }
}
