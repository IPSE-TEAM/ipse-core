
extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;
extern crate pallet_balances as balances;

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, DispatchError,}, debug,

    weights::Weight,
	StorageMap, StorageValue,
	decl_error, ensure,
};
use sp_std::result;

use system::{ensure_signed};
use sp_runtime::{traits::SaturatedConversion, Percent};
use sp_std::vec::Vec;
use sp_std::vec;
use node_primitives::KIB;

pub trait Trait: system::Trait + timestamp::Trait + balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


/// 矿工的机器信息
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineInfo<BlockNumber> {
	/// 磁盘空间
	disk: KIB,
	/// 更新时间
	update_time: BlockNumber,
	/// 机器是否在运行（这个是用户抵押的依据)
	is_stop: bool,
}


/// 抵押信息
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct StakingInfo<AccountId, Balance> {
	/// 矿工
	miner: AccountId,
	/// 矿工分润占比
	miner_portation: Percent,
	/// 总的抵押金额
	total_staking: Balance,
	/// 其他人的抵押 （staker， 抵押金额， 保留金额)
	others: Vec<(AccountId, Balance, Balance)>,
}


/// 操作
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Oprate {
	/// 添加
	Add,
	/// 减少
	Sub,
}

impl Default for Oprate {
	fn default() -> Self {
		Self::Add
	}
}


decl_storage! {
    trait Store for Module<T: Trait> as IpseStakingModule {

		/// 矿工磁盘空间信息
		pub DiskOf get(fn maner_info): map hasher(twox_64_concat) T::AccountId => Option<MachineInfo<T::BlockNumber>>;

		/// 是否在非抵押操作期间（冷冻期，只有矿工能改变信息)
		pub IsChillTime get(fn is_chill_time): bool = true;

		/// 每个矿工对应的抵押信息
		pub StakingInfoOf get(fn stking_info_of): map hasher(twox_64_concat) T::AccountId => Option<StakingInfo<T::AccountId, T::Balance>>;

		/// 用户抵押的矿工
		pub MinersOf get(fn mminers_of): map hasher(twox_64_concat) T::AccountId => Option<Vec<T::AccountId>>;

		/// 自增的p盘id
		pub Pid get(fn p_id): u64;

		/// 矿工对应的p盘id
		pub PidOf get(fn account_id_of): map hasher(twox_64_concat) T::AccountId => Option<u64>;
    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId,
    Balance = <T as balances::Trait>::Balance,
    {
        UpdateDiskInfo(AccountId, KIB),
        Register(AccountId, u64),
        StopMining(AccountId),
        RemoveStaker(AccountId, AccountId),
        Staking(AccountId, AccountId, Balance),
        UpdatePortation(AccountId, Percent),
		UpdateStaking(AccountId, Balance),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;


		/// 矿工注册
		#[weight = 10_000]
		fn register(origin, disk: KIB, miner_portation: Percent) {
			let miner = ensure_signed(origin)?;

			ensure!(disk != 0 as KIB, Error::<T>::DiskEmpty);

			ensure!(Self::is_register(miner.clone())?, Error::<T>::NotRegister);

			let now = Self::now();
			<DiskOf<T>>::insert(miner.clone(), MachineInfo {
        		disk: disk,
        		update_time: now,
        		is_stop: false,

        	});

        	<StakingInfoOf<T>>::insert(&miner,
        		StakingInfo {

        			miner: miner.clone(),
        			miner_portation: miner_portation,
        			total_staking: T::Balance::from(0u32),
        			others: vec![],
        		}
        	);

			// 映射p盘id
        	let mut p_id = <Pid>::get();
        	<PidOf<T>>::insert(miner.clone(), p_id);
        	p_id = p_id.saturating_add(1);

        	<Pid>::put(p_id);

        	Self::deposit_event(RawEvent::Register(miner, disk));

		}


		/// 更新磁盘信息
        #[weight = 10_000]
        fn update_disk_info(origin, disk: KIB) {

        	let miner = ensure_signed(origin)?;

        	ensure!(disk != 0 as KIB, Error::<T>::DiskEmpty);

			/// 必须在非冷冻期
			ensure!(Self::is_chill_time(), Error::<T>::ChillTime);

        	let now = Self::now();

        	ensure!(Self::is_register(miner.clone())?, Error::<T>::NotRegister);

        	<DiskOf<T>>::insert(miner.clone(), MachineInfo {
        		disk: disk,
        		update_time: now,
        		is_stop: false,

        	});

        	Self::deposit_event(RawEvent::UpdateDiskInfo(miner, disk));

        }


		/// 矿工停止挖矿
		#[weight = 10_000]
        fn stop_mining(origin) {

        	let miner = ensure_signed(origin)?;

        	Self::is_can_mining(miner.clone())?;

			<DiskOf<T>>::mutate(miner.clone(), |h| {
				if let Some(x) = h {
					x.is_stop = true
				}
			});

			// todo 对抵押的用户进行解抵押

			Self::deposit_event(RawEvent::StopMining(miner));
        }


        /// 矿工删除抵押者
        #[weight = 10_000]
        fn remove_staker(origin, staker: T::AccountId) {
			let miner = ensure_signed(origin)?;

			Self::is_can_mining(miner.clone())?;

			let staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();

			// todo 查询是否有这个人 没有抛出错误 NotYourStaker

			// todo 归还抵押（抵押 + 保留), 改变总抵押金额

			Self::deposit_event(RawEvent::RemoveStaker(miner, staker));
        }


		/// 用户第一次抵押
        #[weight = 10_000]
        fn staking(origin, miner: T::AccountId, amount: T::Balance) {

        	let who = ensure_signed(origin)?;

			Self::is_can_mining(miner.clone())?;

			// 不在冷冻期
			ensure!(!<IsChillTime>::get(), Error::<T>::ChillTime);

			// todo 自己还没有对这个矿工进行抵押

			// todo 自己有足够余额进行抵押

			// todo 修改存储

			Self::deposit_event(RawEvent::Staking(who, miner, amount));

        }


		/// 抵押者更新抵押金额
        #[weight = 10_000]
        fn update_staking(origin, miner: T::AccountId, oprate: Oprate, amount: T::Balance) {

        	let who = ensure_signed(origin)?;

			// todo 使用saturating进行操作

			Self::deposit_event(RawEvent::UpdateStaking(who, amount));
        }


		/// 矿工更改分润比
        #[weight = 10_000]
        fn update_portation(origin, portation: Percent) {

        	let miner = ensure_signed(origin)?;

			// 在冻结期内才能执行
        	ensure!(<IsChillTime>::get(), Error::<T>::NotChillTime);

        	Self::is_can_mining(miner.clone())?;

        	let mut staking_info = <StakingInfoOf<T>>::get(miner.clone()).unwrap();

        	staking_info.miner_portation = portation.clone();

        	<StakingInfoOf<T>>::insert(miner.clone(), staking_info);

			Self::deposit_event(RawEvent::UpdatePortation(miner, portation));
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


	/// 判断是否已经注册
	fn is_register(miner: T::AccountId) -> result::Result<bool, DispatchError> {

		if <DiskOf<T>>::contains_key(&miner) && <StakingInfoOf<T>>::contains_key(&miner) {
			Ok(true)

		}

		return Err(Error::<T>::NotRegister)?;
	}


	/// 判断矿工是否可以挖矿
	fn is_can_mining(miner: T::AccountId) -> result::Result<(), DispatchError> {
		ensure!(Self::is_register(miner.clone())?, Error::<T>::NotRegister);

		// 已经停止挖矿不能再操作
		ensure!(!<DiskOf<T>>::get(&miner).unwrap().is_stop, Error::<T>::AlreadyStopMining);

		Ok(())
	}

}

decl_error! {
    /// Error for the ipse module.
    pub enum Error for Module<T: Trait> {
    	/// 已经注册过
		AlreadyRegister,
		/// 没有注册过
		NotRegister,
		/// p盘空间为0(不允许)
		DiskEmpty,
		/// 在冷冻期（只能矿工修改信息，用户不能进行抵押或是解抵押操作)
		ChillTime,
		/// 不在冷冻期
		NotChillTime,
		/// 已经停止挖矿
		AlreadyStopMining,
		/// 不是当前矿工的抵押者
		NotYourStaker,

    }
}
