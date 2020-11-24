
extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;
extern crate pallet_balances as balances;
extern crate pallet_babe as babe;
use crate::constants::time::MILLISECS_PER_BLOCK;

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, DispatchError,}, debug,
	traits::{Get, Currency, ReservableCurrency, OnUnbalanced},
    weights::Weight,
	StorageMap, StorageValue,
	decl_error, ensure,
};
use pallet_staking as staking;

use sp_std::result;

use system::{ensure_signed};
use sp_runtime::{traits::{SaturatedConversion, Saturating}, Percent};
use sp_std::vec::Vec;
use sp_std::vec;
use node_primitives::KIB;
use num_traits::{CheckedAdd, CheckedSub};
use crate::ipse_traits::PocHandler;

type BalanceOf<T> =
	<<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait + timestamp::Trait + balances::Trait + babe::Trait + staking::Trait {

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type ChillDuration: Get<Self::BlockNumber>;

	type StakingCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	type StakingDeposit: Get<BalanceOf<Self>>;

	type StakingSlash: OnUnbalanced<NegativeImbalanceOf<Self>>;

	type StakerMaxNumber: Get<usize>;
	type PocHandler: PocHandler<Self::AccountId>;

}


/// 矿工的机器信息
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct MachineInfo<BlockNumber> {
	/// 磁盘空间
	pub disk: KIB,
	/// P盘id
	pub pid: u128,
	/// 更新时间
	pub update_time: BlockNumber,
	/// 机器是否在运行（这个是用户抵押的依据)
	is_stop: bool,
}


/// 抵押信息
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq)]
pub struct StakingInfo<AccountId, Balance> {
	/// 矿工
	pub miner: AccountId,
	/// 矿工分润占比
	pub miner_proportion: Percent,
	/// 总的抵押金额
	pub total_staking: Balance,
	/// 其他人的抵押 （staker， 抵押金额， 保留金额)
	pub others: Vec<(AccountId, Balance, Balance)>,
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
		pub DiskOf get(fn disk_of): map hasher(twox_64_concat) T::AccountId => Option<MachineInfo<T::BlockNumber>>;

		/// 是否在非抵押操作期间（冷却期，只有矿工能改变信息)
		pub IsChillTime get(fn is_chill_time): bool = true;

		/// 每个矿工对应的抵押信息
		pub StakingInfoOf get(fn staking_info_of): map hasher(twox_64_concat) T::AccountId => Option<StakingInfo<T::AccountId, BalanceOf<T>>>;

		/// 用户现在抵押的矿工
		pub MinersOf get(fn miners_of): map hasher(twox_64_concat) T::AccountId => Vec<T::AccountId>;

		/// P盘id对应的矿工
		pub AccountIdOfPid get(fn accouont_id_of_pid): map hasher(twox_64_concat) u128 => Option<T::AccountId>;

		/// 推荐的矿工列表
		pub RecommendList get(fn recommend_list): Vec<(T::AccountId, BalanceOf<T>)>;

    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId,
	Balance = <<T as Trait>::StakingCurrency as Currency<<T as frame_system::Trait>::AccountId>>::Balance,
    {

        UpdateDiskInfo(AccountId, KIB),
        Register(AccountId, u64),
        StopMining(AccountId),
        RemoveStaker(AccountId, AccountId),
        Staking(AccountId, AccountId, Balance),
        UpdateProportion(AccountId, Percent),
		UpdateStaking(AccountId, Balance),
		ExitStaking(AccountId, AccountId),
		UpdatePid(AccountId, u128),
		RequestUpToList(AccountId, Balance),
		RequestDownFromList(AccountId),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
     	/// 冷却期时长（从每个era开始计算，前面的区块是冷却期)
     	const ChillDuration: T::BlockNumber = T::ChillDuration::get();
     	/// staking时候需要保留的余额
     	const StakingDeposit: BalanceOf<T> = T::StakingDeposit::get();
     	/// 一名矿工最多有多少名抵押者
     	const StakerMaxNumber: u32 = T::StakerMaxNumber::get() as u32;

     	type Error = Error<T>;

        fn deposit_event() = default;


		/// 矿工注册
		#[weight = 10_000]
		fn register(origin, kib: KIB, pid: u128, miner_proportion: Percent) {

			let miner = ensure_signed(origin)?;

			ensure!(kib != 0 as KIB, Error::<T>::DiskEmpty);

			// 把kib转变成b
			let disk = kib.checked_mul(1000 as KIB).ok_or(Error::<T>::Overflow)?;

			ensure!(!Self::is_register(miner.clone()), Error::<T>::AlreadyRegister);

			ensure!(!<AccountIdOfPid<T>>::contains_key(pid), Error::<T>::PidInUsing);

			let now = Self::now();
			<DiskOf<T>>::insert(miner.clone(), MachineInfo {
        		disk: disk,
        		pid: pid,
        		update_time: now,
        		is_stop: false,

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

        	Self::deposit_event(RawEvent::Register(miner, disk));

		}


		/// 矿工申请进入推荐列表
		#[weight = 10_000]
		fn request_up_to_list(origin, amount: BalanceOf<T>) {

			// 矿工才能操作
			let miner = ensure_signed(origin)?;

			// 自己是可以挖矿的矿工
			ensure!(Self::is_can_mining(miner.clone())?, Error::<T>::NotRegister);

			Self::sort_account_by_amount(miner.clone(), amount)?;

			Self::deposit_event(RawEvent::RequestUpToList(miner, amount));

		}


		/// 矿工退出推荐列表
		#[weight = 10_000]
		fn request_down_from_list(origin) {
			let miner = ensure_signed(origin)?;
			// 获取推荐列表
			let mut list = <RecommendList<T>>::get();
			if let Some(pos) = list.iter().position(|h| h.0 == miner) {
				let amount = list.swap_remove(pos).1;

				T::StakingCurrency::unreserve(&miner, amount);

				<RecommendList<T>>::put(list);
			}
			else {
				return Err(Error::<T>::NotInList)?;
			}

			Self::deposit_event(RawEvent::RequestDownFromList(miner));

		}



		/// 矿工修改p盘id
		#[weight = 10_000]
		fn update_pid(origin, pid: u128) {
			let miner = ensure_signed(origin)?;

			ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);

			ensure!(!<AccountIdOfPid<T>>::contains_key(pid), Error::<T>::PidInUsing);

			let old_pid = <DiskOf<T>>::get(miner.clone()).unwrap().pid;

			<AccountIdOfPid<T>>::remove(old_pid);

			<DiskOf<T>>::mutate(miner.clone(), |h| if let Some(i) = h {
				i.pid = pid;
				i.is_stop = false;
			}
			);

			<AccountIdOfPid<T>>::insert(pid, miner.clone());

			Self::deposit_event(RawEvent::UpdatePid(miner, pid));

		}


		/// 更新磁盘信息
        #[weight = 10_000]
        fn update_disk_info(origin, kib: KIB) {

        	let miner = ensure_signed(origin)?;

			// 把kib转变成b
			let disk = kib.checked_mul(1000 as KIB).ok_or(Error::<T>::Overflow)?;

			ensure!(disk != 0 as KIB, Error::<T>::DiskEmpty);

			/// 必须在非冷冻期
// 			ensure!(Self::is_chill_time(), Error::<T>::ChillTime);

			T::PocHandler::remove_history(miner.clone());

        	let now = Self::now();

        	ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);

        	<DiskOf<T>>::mutate(miner.clone(), |h| if let Some(i) = h {
        		i.disk = disk;
        		i.update_time = now;
        	}
        	);

        	Self::deposit_event(RawEvent::UpdateDiskInfo(miner, disk));

        }


		/// 矿工停止挖矿
		#[weight = 10_000]
        fn stop_mining(origin) {

        	let miner = ensure_signed(origin)?;

        	Self::is_can_mining(miner.clone())?;

			<DiskOf<T>>::mutate(miner.clone(), |h| {
				if let Some(x) = h {
					x.is_stop = true;
				}
			});

			let mut staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();
			let others = staking_info.others;
			for staker_info in others.iter() {

				T::StakingCurrency::unreserve(&staker_info.0, staker_info.1.clone());
				T::StakingCurrency::unreserve(&staker_info.0, staker_info.2.clone());

				Self::staker_remove_miner(staker_info.0.clone(), miner.clone());
			}

			staking_info.total_staking = <BalanceOf<T>>::from(0u32);

			staking_info.others = vec![];

			<StakingInfoOf<T>>::insert(&miner, staking_info);

			// 从推荐列表中删除
			<RecommendList<T>>::mutate(|h| h.retain(|i| if i.0 != miner.clone() {
				T::StakingCurrency::unreserve(&i.0, i.1);
				true
			}
			else {
				false
			}
			));

			Self::deposit_event(RawEvent::StopMining(miner));
        }


        /// 矿工删除抵押者
        #[weight = 10_000]
        fn remove_staker(origin, staker: T::AccountId) {

			let miner = ensure_signed(origin)?;

			Self::update_staking_info(miner.clone(), staker.clone(), Oprate::Sub, None, true)?;

			Self::deposit_event(RawEvent::RemoveStaker(miner, staker));
        }


		/// 用户第一次抵押
        #[weight = 10_000]
        fn staking(origin, miner: T::AccountId, amount: BalanceOf<T>) {

        	let who = ensure_signed(origin)?;

			Self::is_can_mining(miner.clone())?;

			// 不在冷冻期
// 			ensure!(!<IsChillTime>::get(), Error::<T>::ChillTime);

			// 还没有抵押
			if Self::staker_pos(miner.clone(), who.clone()).is_some() {

				return Err(Error::<T>::AlreadyStaking)?;
			}

			let bond = amount.checked_add(&T::StakingDeposit::get()).ok_or(Error::<T>::Overflow)?;

			let staker_info = (who.clone(), amount.clone(), T::StakingDeposit::get());

			let mut staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();

			ensure!(staking_info.others.len() <= T::StakerMaxNumber::get(), Error::<T>::StakerNumberToMax);

			let total_amount = staking_info.clone().total_staking;

			let now_amount = total_amount.checked_add(&amount).ok_or(Error::<T>::Overflow)?;

			T::StakingCurrency::reserve(&who, bond)?;

			staking_info.total_staking = now_amount;

			staking_info.others.push(staker_info);

			<StakingInfoOf<T>>::insert(miner.clone(), staking_info);

			<MinersOf<T>>::mutate(who.clone(), |h| h.push(miner.clone()));

			Self::deposit_event(RawEvent::Staking(who, miner, amount));

        }


		/// 抵押者更新抵押金额
        #[weight = 10_000]
        fn update_staking(origin, miner: T::AccountId, oprate: Oprate, amount: BalanceOf<T>) {

        	let staker = ensure_signed(origin)?;

			Self::update_staking_info(miner, staker.clone(), oprate, Some(amount), false)?;

			Self::deposit_event(RawEvent::UpdateStaking(staker, amount));

        }


        /// 用户退出抵押
        #[weight = 10_000]
        fn exit_Staking(origin, miner: T::AccountId) {
        	let staker = ensure_signed(origin)?;
        	Self::update_staking_info(miner.clone(), staker.clone(), Oprate::Sub, None, false)?;
        	Self::deposit_event(RawEvent::ExitStaking(staker, miner));
        }


		/// 矿工更改分润比
        #[weight = 10_000]
        fn update_proportion(origin, proportion: Percent) {

        	let miner = ensure_signed(origin)?;

// 			// 在冻结期内才能执行
//         	ensure!(<IsChillTime>::get(), Error::<T>::NotChillTime);

        	Self::is_can_mining(miner.clone())?;

        	let mut staking_info = <StakingInfoOf<T>>::get(miner.clone()).unwrap();

        	staking_info.miner_proportion = proportion.clone();

        	<StakingInfoOf<T>>::insert(miner.clone(), staking_info);

			Self::deposit_event(RawEvent::UpdateProportion(miner, proportion));
        }


		fn on_initialize(n: T::BlockNumber) -> Weight {
			let _ = Self::update_chill();
			0

       }

     }
}

impl<T: Trait> Module<T> {


	fn current_epoch_start() -> result::Result<u64, DispatchError> {

		let time = <babe::Module<T>>::current_epoch_start();
		let block_number = time.checked_div(MILLISECS_PER_BLOCK).ok_or((Error::<T>::Overflow))?;
		Ok(block_number)

	}


	/// 获取当前区块
	pub fn now() -> T::BlockNumber {

		<system::Module<T>>::block_number()
	}


	/// 判断自己是否是某个矿工的抵押者(是的话在什么位置)
	fn staker_pos(miner: T::AccountId, staker: T::AccountId) -> Option<usize> {
		let staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();
		let others = staking_info.others;
		let pos = others.iter().position(|h| h.0 == staker);
		pos
	}


	/// 判断是否进入冷却期
	fn update_chill() -> DispatchResult {

		let now = Self::now();

		let era_start_time = <staking::Module<T>>::era_start_block_number();

		let chill_duration = T::ChillDuration::get();

		let time = chill_duration.checked_add(&era_start_time).ok_or(Error::<T>::Overflow)?;

		debug::info!("poc_staking era start_time: {:?}, chill end_time: {:?}", era_start_time, time);

		if now <= time {
			<IsChillTime>::put(true)
		}

		else {
			<IsChillTime>::put(false)
		}

		Ok(())

	}

	/// 判断是否已经注册
	fn is_register(miner: T::AccountId) -> bool {

		if <DiskOf<T>>::contains_key(&miner) && <StakingInfoOf<T>>::contains_key(&miner) {
			true
		}

		else {
			false
		}


	}


	/// 判断矿工是否可以挖矿
	pub fn is_can_mining(miner: T::AccountId) -> result::Result<bool, DispatchError> {
		ensure!(Self::is_register(miner.clone()), Error::<T>::NotRegister);

		// 已经停止挖矿不能再操作
		ensure!(!<DiskOf<T>>::get(&miner).unwrap().is_stop, Error::<T>::AlreadyStopMining);

		Ok(true)
	}


	/// staker删除自己抵押的矿工记录
	fn staker_remove_miner(staker: T::AccountId, miner: T::AccountId) {

		<MinersOf<T>>::mutate(staker.clone(), |miners|  {
			miners.retain(|h| h != &miner);

		});

	}

	/// 排列矿工后需要做的
	fn sort_after(miner: T::AccountId, amount: BalanceOf<T>, index: usize, mut old_list: Vec<(T::AccountId, BalanceOf<T>)>) -> result::Result<(), DispatchError> {
		// 先对矿工进行抵押

		T::StakingCurrency::reserve(&miner, amount)?;

		old_list.insert(index, (miner, amount));

		if old_list.len() > 20 {
			let abandon = old_list.split_off(20);
			// 对被淘汰的人进行释放
			for i in abandon {
				T::StakingCurrency::unreserve(&i.0, i.1);
			}
		}

		<RecommendList<T>>::put(old_list);

		Ok(())

	}

	/// 根据抵押的金额来排列account_id
	fn sort_account_by_amount(miner: T::AccountId, mut amount: BalanceOf<T>) -> result::Result<(), DispatchError> {

		// 获取之前的列表
		let mut old_list = <RecommendList<T>>::get();

		let mut miner_old_info: Option<(T::AccountId, BalanceOf<T>)> = None;

		// 如果之前有 那就累加金额
		if let Some(pos) = old_list.iter().position(|h| h.0 == miner.clone()) {

			miner_old_info = Some(old_list.remove(pos));

		}
		if miner_old_info.is_some() {
			// 判断能否继续琐仓amount 如果是 就暂时释放；如果不行 就退出
			let old_amount = miner_old_info.clone().unwrap().1;

			ensure!(T::StakingCurrency::can_reserve(&miner, amount), Error::<T>::AmountNotEnough);

			T::StakingCurrency::unreserve(&miner, old_amount);

			amount = amount + old_amount;

		}

		// 如果列表为空， 直接更新数据
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

	/// 更新已经抵押过的用户的抵押金额
	fn update_staking_info(miner: T::AccountId, staker: T::AccountId, oprate: Oprate, amount_opt: Option<BalanceOf<T>>, is_slash: bool) -> DispatchResult {
		// 如果操作是减仓 那么amount_opt是none意味着抵押者退出
		// 如果操作是加仓 那么amount_opt 不能是none值
		Self::is_can_mining(miner.clone())?;

		let mut amount: BalanceOf<T>;


		if let Some(pos) = Self::staker_pos(miner.clone(), staker.clone()) {

			let mut staking_info = <StakingInfoOf<T>>::get(&miner).unwrap();

			let mut staker_info = staking_info.others.swap_remove(pos);

			/// 这个是减仓的时候
			if amount_opt.is_none() {
				amount = staker_info.1.clone()
			}
			else {
				amount = amount_opt.unwrap()
			}

			match  oprate {

				Oprate::Add => {
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
    	/// p盘id已经被使用
    	PidInUsing,
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
		/// 已经抵押
		AlreadyStaking,
		/// 数据溢出
		Overflow,
		/// 抵押人数超过限制
		StakerNumberToMax,
		/// 账户金额不够
		AmountNotEnough,
		/// 不在推荐列表中
		NotInList,



	}
}
