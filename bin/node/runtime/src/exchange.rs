use sp_std::{prelude::*,convert::TryInto,fmt::Debug};
use sp_core::{crypto::AccountId32 as AccountId};
use sp_core::{crypto::KeyTypeId,offchain::Timestamp};

use frame_support::{print,Parameter,decl_module,decl_error, decl_storage, decl_event, dispatch, debug, traits::Get,IterableStorageMap,
                    StorageDoubleMap,ensure,weights::Weight};
use frame_system::{self as system,RawOrigin,Origin, ensure_signed,ensure_none, offchain};
use hex;

use pallet_timestamp as timestamp;
use pallet_authority_discovery as authority_discovery;

use sp_runtime::{traits::{MaybeSerializeDeserialize,MaybeDisplay}, DispatchResult,DispatchError};
use sp_io::{self, misc::print_utf8 as print_bytes};
use codec::{ Encode,Decode };
use num_traits::float::FloatCore;
use frame_system::offchain::{
    SendTransactionTypes,
    SubmitTransaction,
};


use sp_runtime::{
    AnySignature,MultiSignature,MultiSigner,
    offchain::http, transaction_validity::{
        TransactionValidity, TransactionLongevity, ValidTransaction, InvalidTransaction,TransactionSource,TransactionPriority},
    traits::{CheckedSub,CheckedAdd,Printable,Member,Zero,IdentifyAccount},
    RuntimeAppPublic};
use app_crypto::{sr25519};

use crate::ocw_common::*;

// 请求的查询接口
const EOS_NODE_URL: &[u8] = b"http://localhost:8421/v1/eosio/tx/";

type Signature = AnySignature;
pub mod eos_crypto {
    use super::{AccountIdPublicConver,Signature};
    pub mod app_sr25519 {
        use super::{AccountIdPublicConver};
        use sp_runtime::{MultiSignature,MultiSigner};
        use sp_runtime::traits::{IdentifyAccount};  // AccountIdConversion,
    use sp_core::{crypto::AccountId32 as AccountId};
        use sp_runtime::app_crypto::{app_crypto,key_types::ACCOUNT,sr25519};
        app_crypto!(sr25519, ACCOUNT);

        impl From<Signature> for super::Signature {
            fn from(a: Signature) -> Self {
                sr25519::Signature::from(a).into()
            }
        }

        impl AccountIdPublicConver for Public{
            type AccountId = AccountId;
            fn into_account32(self) -> AccountId{
                let s: sr25519::Public = self.into();
                MultiSigner::from(s).into_account()
            }
        }

        impl From<AccountId> for Public {
            fn from(acct: AccountId) -> Self { /// AccountId可以转换为 Public
                let mut data =  [0u8;32];
                let acct_data: &[u8;32] = acct.as_ref();
                for (index, val) in acct_data.iter().enumerate() {
                    data[index] = *val;
                }
                Self(sr25519::Public(data))
            }
        }


        impl From<[u8; 32]> for Public {
            fn from(acct: [u8; 32]) -> Self {
                let mut data =  [0u8;32];
                for (index, val) in acct.iter().enumerate() {
                    data[index] = *val;
                }
                Self(sr25519::Public(data))
            }
        }

    }

    app_crypto::with_pair! {
		/// An bridge-eos keypair using sr25519 as its crypto.
		pub type AuthorityPair = app_sr25519::Pair;
	}

    pub type AuthoritySignature = app_sr25519::Signature;

    pub type AuthorityId = app_sr25519::Public;
}

enum VerifyStatus {
    Continue,  //  不做任何处理
    Failed,    // 注册失败
    Pass,      // 注册成功
}

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + SendTransactionTypes<Call<Self>>{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    // type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default
    //                 + AsRef<[u8]> + From<[u8; 32]>;

    /// The local AuthorityId
   type AuthorityId: RuntimeAppPublic + Clone + Parameter +
                    Into<sr25519::Public> + From<sr25519::Public> + AccountIdPublicConver<AccountId=Self::AccountId> +
                    From<<Self as frame_system::Trait>::AccountId> + From<[u8; 32]>;

    // tx 队列的最大长度
    type TxsMaxCount: Get<u32>;

    type Duration: Get<Self::BlockNumber>;  // 对记录的清除周期

    type UnsignedPriority: Get<TransactionPriority>;

}

decl_error! {
    pub enum Error for Module<T: Trait> {
      /// 账号不一直
      MemoMissMatch,

      /// tx 已经被使用过
      TxExisted,

      /// tx 正在被使用中
      TxInUsing,

      /// txsoverlimit
	  OverMaximum,

	  /// 账号错误
	  MemoInvalid,

	  /// tx 已经被兑换过了
	  TxExChanged,
    }
}

decl_event!(
  pub enum Event<T> where
    AccountId = <T as system::Trait>::AccountId,
    BlockNumber = <T as system::Trait>::BlockNumber,
    {
        FetchedSuc(AccountId,BlockNumber, Vec<u8>, u64), // 当前address 状态记录事件

        FailedEvent(AccountId,BlockNumber,Vec<u8>), // 记录返回错误的情况

        AddExchangeQueueEvent(Vec<u8>), //
  }
);


// This module's storage items.
decl_storage! {
  trait Store for Module<T: Trait> as PostExchange {
        /// 等同于队列作用
        /// 设置状态位  u32表现形式1xxx.初始值为 1000,低3位分别表示 验证通过次数,验证失败次数,非正常情况返回次数, tx => 1xxx,AccountId,
	    TokenStatus get(fn tx_status): map hasher(blake2_128_concat) Vec<u8> => (u64,T::AccountId); // 收款的账号
		/// 记录 TokenStatus 的长度,防止队列过大
		pub TokenStatusLen: u32;

		/// The current set of notary keys that may send bridge transactions to Eos chain.
		NotaryKeys get(fn notary_keys) config(): Vec<T::AccountId>;   // 使用 json文件来配置

        /// 只记录成功兑换的 tx  tx=>bool
        SucTxExchange get(fn suc_tx_exchange): map hasher(blake2_128_concat) Vec<u8> => Option<bool>;

       /// 兑换记录  AccountId,tx => (AddressStatus,兑换的个数)
		EosExchangeInfo get(fn eos_exchange_info): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) Vec<u8> => (AddressStatus, u64);

       ///记录查询结果,key: T::BlockNumber(1小时的周期数)+T::AccountId, val:(成功次数,2000x 状态码次数,5000x状态码次数).不会删除
       FetchRecord get(fn fetch_record): double_map hasher(blake2_128_concat) T::BlockNumber,hasher(blake2_128_concat) T::AccountId => (u32,u32,u32);

       /// 记录失败的,定期全部清除. Vec<FetchFailedOf<T>> 最多保持50个的长度.原本是 linked_map
       pub FetchFailed get(fn fetch_failed): map hasher(blake2_128_concat) T::AccountId => Vec<FetchFailedOf<T>>;
  }
  	add_extra_genesis {
		build(|config: &GenesisConfig<T>| {
			NotaryKeys::<T>::put(config.notary_keys.clone());

		});
	}
}

// The module's dispatchable functions.
decl_module! {
  /// The module declaration.
  pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    type Error = Error<T>;
    // Initializing events
    // this is needed only if you are using events in your module
    fn deposit_event() = default;

    // Clean the state on initialization of the block
    fn on_initialize(block: T::BlockNumber) -> Weight{
        if (block % T::Duration::get()).is_zero() {
            // 删除所有的失败记录
        for key_value in <FetchFailed<T>>::iter().into_iter(){ // sym,vec<>, linked_map的作用
            let (key,val) = key_value;
             <FetchFailed<T>>::remove(&key);
            }
        }
        0
       }

     #[weight = 0]
     fn exchange(origin, tx: Vec<u8>) -> DispatchResult{
        //用户填写 eos 的转账tx, memo 表示 ipse的接收地址
        // 根据 转账的 post 个数兑换相应的 post2
        let who = ensure_signed(origin)?;
        // let account = Self::vec_convert_account(memo.clone()).ok_or(Error::<T>::MemoInvalid)?;
        // debug::info!("memo is {:?}",account);
        // 判断是否已经 兑换成功过了就直接返回
        let tx_hex = hex::encode(&tx);
        debug::info!("验证 tx = {:?}",tx_hex);
        match SucTxExchange::get(&tx){
            Some(tx) => return Err(Error::<T>::TxExChanged)?,
            _ => (),
        }

        // 初始化状态码
        let curent_status = <TokenStatus<T>>::get(tx.clone()).0;
        // 等于0表示没被占用,继续向下执行
        ensure!(<TokenStatus<T>>::get(tx.clone()).0 == 0, Error::<T>::TxInUsing);
        debug::info!("当前长度: {:?}",TokenStatusLen::get());
        ensure!(TokenStatusLen::get() <= T::TxsMaxCount::get(), Error::<T>::OverMaximum);
        <TokenStatus<T>>::insert(tx.clone(),(1000,who.clone()));
        debug::info!("TokenStatus 初始化状态为:{:?}",<TokenStatus<T>>::get(tx.clone()).0);
        TokenStatusLen::mutate(|n|*n += 1);
        Self::deposit_event(RawEvent::AddExchangeQueueEvent(tx.clone()));
        Ok(())
     }


    #[weight = 0]
    fn record_suc_verify(
      origin,
      block_num: T::BlockNumber,
      account: T::AccountId,  // 本地验证者账号
      key: T::AuthorityId,
      tx:Vec<u8>,
      status: u64,
      quantity: u64,
      _signature: <T::AuthorityId as RuntimeAppPublic>::Signature
    ) -> DispatchResult{
      ensure_none(origin)?;
      let now = <timestamp::Module<T>>::get();
      let block_num = <system::Module<T>>::block_number();
      let duration = block_num / T::Duration::get();
//        1000: 表示初始值
//        1001: 表示验证1次，但是请求失败了
//        130x: 终止，3个验证通过 pass
//        1x7x: 终止，7个节点验证不通过,就failed
//        1009: 终止，网络全部失败   todo: 怎么处理？ 目前按照 failed处理
//        1109: pass 处理

       let (token_status,accept_account) = <TokenStatus<T>>::get(tx.clone());
       debug::info!("token_status={:?},accept_account={:?}",token_status,accept_account);
       ensure!(<TokenStatus<T>>::contains_key(tx.clone()), "不需要再操作,tx已经从TokenStatus移除");
       match SucTxExchange::get(&tx){  // 也可以不需要此判断
        Some(_) => return Err(Error::<T>::TxExChanged)?,
        _ => (),
      }
      debug::info!("获取到了本地服务的返回信息,对状态位操作");
      // let status = post_tx_transfer_data.code;
      if status == 0{   // 0
        // 成功
       <FetchRecord<T>>::mutate(
        duration,account.clone(),
        |val|{
            val.0 = val.0.checked_add(1).unwrap();
        });
       <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(100).unwrap());//  通过次数加1,总次数加1
      }else if status == 1 { // query  ./token-query 没有收到返回的消息
         <FetchRecord<T>>::mutate(
            duration,account.clone(),
            |val|{
                val.2 = val.2.checked_add(1).unwrap();
//                val
            });
             <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(1).unwrap());// 仅仅对总次数加1
      }else{
        <FetchRecord<T>>::mutate( // 200x
            duration,account.clone(),
            |val|{
                val.1 = val.1.checked_add(1).unwrap();
//                val
            });
             <TokenStatus<T>>::mutate(&tx,|val|val.0 = val.0.checked_add(10).unwrap()); // 失败次数加1,总次数加1
      }
      Self::verify_handle(&tx, quantity);
      debug::info!("----上链成功: record_suc_verify:{:?}-----", duration);
      Ok(())
    }

    #[weight = 0]
    fn record_fail_verify(
        origin,
        block: T::BlockNumber,
        account: T::AccountId, // 本地验证者账号
        key: T::AuthorityId,
        tx: Vec<u8>,
        err: Vec<u8>,
        _signature: <T::AuthorityId as RuntimeAppPublic>::Signature
        )->DispatchResult{
            ensure_none(origin)?;
            <TokenStatus<T>>::try_mutate(&tx,|val|{
                if val.0 == 0 {
                    debug::error!("当前 TokenStatus 状态为 0.不需要再操作,tx已经从TokenStatus移除");
                    return Err("");
                }
                debug::info!("record_fail_verify 对状态位加 1");
                val.0 = val.0.checked_add(1).unwrap();  // 无应答情况
                return Ok(&tx)}
            )?;
              // 记录获取fetch失败的信息
            let failed_struct = FetchFailedOf::<T> {
                    block_num: block,
                    tx: tx.clone(),
                    err: err.clone()
            };
            let status:u64 = <TokenStatus<T>>::get(tx.clone()).0;
            debug::info!("------验证失败:status={:?},tx={:?}-------",status,hex::encode(&tx));
            Self::verify_handle(&tx, 0);
            <FetchFailed<T>>::mutate(&account, |fetch_failed| {
            if fetch_failed.len()>50{  // 最多保留50个的长度
                fetch_failed.pop();
            }
            fetch_failed.push(failed_struct)
            });

            if err == WAIT_HTTP_CONVER_REPONSE.as_bytes().to_vec(){ // 本地服务没开起来
            // todo: 需不需要额外处理
            }

            Self::deposit_event(RawEvent::FailedEvent(account.clone(),block,tx));
            debug::info!("------fetch失败记录上链成功:record_fail_verify--------");
            Ok(())
    }


    fn offchain_worker(block: T::BlockNumber) {
        if sp_io::offchain::is_validator() { // 是否是验证人的模式启动
             if let (Some(authority_id),Some(local_account)) = Self::local_authority_keys() {  // local_account 是本地验证者
                debug::info!("-----------exchange offchain work------------");
                match Self::offchain(block,authority_id,&local_account){
                    Err(e)=>{
                        debug::error!("ocw excute error:{:?}",e);
                    },
                    _ => debug::info!("ocw excute suc"),
                }
            }
        }
    } // end of `fn offchain_worker()`



    }
}

impl<T: Trait> Module<T> {
    fn offchain(block_num: T::BlockNumber, key: T::AuthorityId, local_account: &T::AccountId) -> DispatchResult{
        for (tx_key,value) in <TokenStatus<T>>::iter().into_iter(){
            let (status,accept_account) = value;   // 注册的账号名
            let tx= tx_key;  // 转账tx
            let tx_hex = hex::encode(&tx);
            debug::info!("迭代器获取 tx = {:?}",tx_hex);

            // post json 构造
            // let body = Self::get_json(&tx,&accept_account).ok_or("get_json error");
            // let body = match body{
            //     Ok(body) => body,
            //     Err(e) => {
            //         debug::error!("---------{:?}---------",e);
            //         Self::call_record_fail_verify(block_num,key.clone(),local_account,&tx,e)?;
            //         return Err(DispatchError::Other("get_json error"));
            //     }
            // };

            // get
            let body = tx_hex.as_bytes().to_vec();
            // get请求,并结果上链
            match Self::fetch_http_result_status(EOS_NODE_URL,body,accept_account){
                Ok(mut post_tx_transfer_data) => {
                    let tx_hex = hex::encode(&tx);
                    debug::info!("*** fetch ***: {:?}:{:?}",
                            core::str::from_utf8(EOS_NODE_URL).unwrap(),
                            tx_hex);
                    Self::call_record_address(block_num, key.clone(), local_account, &tx, post_tx_transfer_data)?;
                },
                Err(e) => {
                    debug::info!("~~~~~~ Error address fetching~~~~~~~~:  {:?}: {:?}",tx_hex,e);
                    Self::call_record_fail_verify(block_num,key.clone(),local_account,&tx,e)?;
                    // 实现错误信息上链
                }
            }
            break;
        }
        Ok(())

    }

    fn call_record_fail_verify<'a>(
        block_num: T::BlockNumber,
        key: T::AuthorityId,
        account: &T::AccountId, // 验证者账号
        tx: &'a [u8],
        e: &'a str,
    ) -> StrDispatchResult{
        // 错误信息上链
        let signature = key.sign(&(block_num,account.clone(),tx.to_vec()).encode()).ok_or("signing failed!")?;
        debug::info!("record_fail_verify signed,block_num = {:?},tx={:?}",block_num, hex::encode(&tx));

        let call = Call::record_fail_verify(block_num,account.clone(),key.clone(),tx.to_vec(), e.as_bytes().to_vec(),signature);
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|_| {
                debug::error!("===record_fail_verify: submit_unsigned_call error===");
                "===record_fail_verify: submit_unsigned_call error==="
            })?;
        debug::info!("+++++++record_fail_verify suc++++++++++++++");
        Ok(())
    }

    fn call_record_address(
        block_num: T::BlockNumber,
        key: T::AuthorityId,
        account: &T::AccountId,  // 本地验证者
        tx: &[u8],  //tx
        post_tx_transfer_data: PostTxTransferData
    ) -> StrDispatchResult{
        let signature = key.sign(&(block_num,account.clone(),tx.to_vec()).encode()).ok_or("signing failed!")?;
        debug::info!("record_suc_verify signed,block_num = {:?},tx={:?}",block_num, hex::encode(tx));
        let call = Call::record_suc_verify(
            block_num,
            account.clone(),
            key.clone(),
            tx.to_vec(),
            post_tx_transfer_data.code,
            post_tx_transfer_data.quantity,
            signature
        );

        // Unsigned tx
        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
            .map_err(|e| {
                debug::info!("{:?}",e);
                "============fetch_price: submit_signed(call) error=================="})?;

        debug::info!("***fetch price over ^_^***");
        Ok(())
    }


    // 组成 post json
    fn get_json(tx: &[u8], accept_account: &T::AccountId) -> Option<Vec<u8>> {
        let keys:[&[u8];2] = [b"tx",b"account"];
        let tx_vec= hex_to_u8(tx);
        debug::info!("转换后的tx={:?}",core::str::from_utf8(&tx_vec).ok()?);
        let acc = Self::account_convert_u8(accept_account.clone());
        debug::info!("转换后的 acc={:?}",core::str::from_utf8(&acc).ok()?);
        let vals:[&[u8];2] = [&tx_vec,&acc];

        let mut json_val = vec![POST_KEYWORD[0]];
        for (i,key) in keys.iter().enumerate(){
            // 形如 "tx": "xxxx",
            json_val.push(POST_KEYWORD[1]); //json_val.push("\"");
            json_val.push(key);
            json_val.push(POST_KEYWORD[2]);  //json_val.push("\":");
            json_val.push(POST_KEYWORD[1]);  // json_val.push("\"");
            json_val.push(vals[i]);
            json_val.push(POST_KEYWORD[1]);  //json_val.push("\"");
            json_val.push(POST_KEYWORD[3]);  //json_val.push(",");
        }
        json_val.pop();    // 移除最后一个 ","
        json_val.push(POST_KEYWORD[4]);    //json_val.push("}");

        let json_vec = json_val.concat().to_vec();
        debug::info!("请求的json:{:?}",core::str::from_utf8(&json_vec).ok()?);

        Some(json_vec)
    }


    fn local_authority_keys() -> (Option<T::AuthorityId>,Option<T::AccountId>){
        let authorities = NotaryKeys::<T>::get();
        let key_id = core::str::from_utf8(&T::AuthorityId::ID.0).unwrap();
        debug::info!("当前的节点 keytypeId: {:?}",key_id);
        for i in T::AuthorityId::all().iter(){   // 本地的账号
            let authority: T::AuthorityId = (*i).clone();
            let  authority_sr25519: sr25519::Public = authority.clone().into();
            let s: T::AccountId= authority.clone().into_account32();
            debug::info!("本地账号信息:{:?}",s);
            if authorities.contains(&s){
                debug::info!("找到了本地账号: {:?}",s);
                return (Some(authority),Some(s));
            }
        }
        return (None,None);
    }

    fn verify_handle(tx: &[u8], quantity: u64) -> StdResult<VerifyStatus>{
        // 是否举报, true 表示举报
//        判断
//        1.十位的数字 >=7 fail ,   记录失败个数
//        2.个位数字 >=8, failed
//        3.百位数字 >=3,pass
        let mut verify_status:VerifyStatus = VerifyStatus::Continue;//初始化一个值
        let (status,accept_account) = <TokenStatus<T>>::get(tx);
        let num = TokenStatusLen::get();
        if status  < 1000{
            debug::error!("=====绑定验证失败:当前的 tx={:?},状态为 {:?},=====",hex::encode(tx),status);
            <TokenStatus<T>>::remove(tx);
            if num > 0{
                TokenStatusLen::mutate(|n|*n -= 1);
            }
            return Err("status 小于 1000");
        }

        debug::info!("--------onchain set status={:?}--------",status);
        let units_digit = status%10;     // 个位数
        let tens_digit = status/10%10;   // 十位数
        let hundreds_digit  = status/100%10;  // 百位数

        if tens_digit >= 6 {    // 十位数
            verify_status = VerifyStatus::Failed;
        }else if hundreds_digit >=3 {   // 百位数大于3
            verify_status = VerifyStatus::Pass;
        } else if units_digit >= 8 {   // 个位数大于8
            // 且十位数
            if hundreds_digit >=2 {
                verify_status = VerifyStatus::Pass;
            }else{
                verify_status = VerifyStatus::Failed;
            }
        } else {
            verify_status = VerifyStatus::Continue;  // 默认值
        }

        // let active_status = <EosExchangeInfo<T>>::get(accept_account.clone(), tx.clone());
        match verify_status{
            VerifyStatus::Failed => {  // 失败
            debug::info!("--注册失败--");
                <TokenStatus<T>>::remove(tx); // 移除掉
                debug::info!("移除 tx={:?},队列剩余:{:?} 个",hex::encode(tx.clone()),num);
                if num > 0{
                    TokenStatusLen::mutate(|n|*n -= 1);
                }
                <EosExchangeInfo<T>>::insert(accept_account.clone(),tx.clone(), (AddressStatus::InActive, quantity));
                // Self::insert_active_status(accept_account.clone(), tx, AddressStatus::InActive);
            }
            VerifyStatus::Pass => {  // 成功
            debug::info!("--注册成功--");
                debug::info!("移除 tx={:?},队列剩余:{:?} 个",hex::encode(tx.clone()),num);
                <TokenStatus<T>>::remove(tx); // 移除掉
                if num >0{
                    TokenStatusLen::mutate(|n|*n -= 1);
                }
                <EosExchangeInfo<T>>::insert(accept_account.clone(), tx.clone(), (AddressStatus::Active, quantity));
                <SucTxExchange>::insert(tx.clone(),true);
                // Self::insert_active_status(accept_account.clone(), tx, AddressStatus::active);
            }
            _ => {}
        }
        return Ok(verify_status);
    }


    // fn insert_active_status(accept_account: T::AccountId, tx:&[u8], active_status: AddressStatus){
    //     let position = register_list.iter().position(|p| p.3 == symbol.clone());  // 注册的列表
    //     match position{
    //         Some(x) => {
    //             debug::info!("---------AddressOf 已经存在了 {:?}---------",hex::encode(tx));
    //             register_list[x] = (token_address,active_status,tx.to_vec(),symbol);
    //             <AddressOf<T>>::insert(register_account,register_list);
    //         },
    //         None => {
    //             debug::info!("------- AddressOf 不存在 {:?}-----------",hex::encode(tx));
    //             <AddressOf<T>>::mutate(register_account, |v|{
    //                 v.push((token_address,active_status,tx.to_vec(),symbol));
    //             });
    //         }
    //     }
    //
    //
    // }

    fn fetch_http_result_status(
        remote_url: &[u8],
        body: Vec<u8>,
        accept_account: T::AccountId
    ) -> StdResult<PostTxTransferData> {
        let json = Self::fetch_json(remote_url, body)?; // http请求
        let mut post_tx_transfer_data: PostTxTransferData = Self::fetch_parse(json)?; // json parse
        Self::get_verify_status(&mut post_tx_transfer_data,accept_account);
        debug::info!("----verified status = {:?}----", post_tx_transfer_data.code);
        Ok(post_tx_transfer_data)
    }


    fn fetch_json<'a>(remote_url: &'a [u8], body:Vec<u8>) -> StdResult<Vec<u8>>{  // http get
        let url = &[remote_url,body.as_slice()].concat();
        let remote_url_str = core::str::from_utf8(url)
            .map_err(|_| "Error in converting remote_url to string")?;
        debug::info!("get url: {:?}",remote_url_str);
        let now = <timestamp::Module<T>>::get();
        let deadline:u64 = now.try_into().
            map_err(|_|"An error occurred when moment was converted to usize")?  // usize类型
            .try_into().map_err(|_|"An error occurred when usize was converted to u64")?;
        let deadline = Timestamp::from_unix_millis(deadline+20000); // 等待最多10s
        // let body = sp_std::str::from_utf8(&body).map_err(|e|"symbol from utf8 to str failed")?;
        let mut new_reuest = http::Request::get(remote_url_str);
        new_reuest.deadline = Some(deadline);
        let pending = new_reuest.send()
            .map_err(|_| "Error in sending http get request")?;

        let http_result = pending.try_wait(deadline)
            .map_err(|_| PENDING_TIME_OUT)?; // "Error in waiting http response back"
        let response = http_result.map_err(|_| WAIT_HTTP_CONVER_REPONSE )?;

        if response.code != 200 {
            debug::warn!("Unexpected status code: {}", response.code);
            let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();
            debug::info!("error body:{:?}", core::str::from_utf8(&json_result).unwrap());
            return Err("Non-200 status code returned from http request");
        }

        let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

        // Print out the whole JSON blob
        // debug::info!("---response---{:?}",&core::str::from_utf8(&json_result).unwrap());
        Ok(json_result)
    }

    fn fetch_parse(resp_bytes: Vec<u8>) -> StdResult<PostTxTransferData> {
        let resp_str = core::str::from_utf8(&resp_bytes).map_err(|_| "Error in fetch_parse")?;
        // Print out our fetched JSON string
        debug::info!("json: {}", resp_str);

        // Deserializing JSON to struct, thanks to `serde` and `serde_derive`
        let post_tx_transfer_data: PostTxTransferData =
            serde_json::from_str(&resp_str).map_err(|e|{
                debug::info!("parse error: {:?}",e);
                "convert to ResponseStatus failed"})?;

        debug::info!("http get status:{:?}", post_tx_transfer_data.code);
        Ok(post_tx_transfer_data)
    }

    fn get_verify_status(post_transfer_data: &mut PostTxTransferData, acc: T::AccountId){
       /// 验证, 返回状态码
        if post_transfer_data.code != 1 {  // 1: 本地服务获取eos节点查询无效
            // 255和0是本地服务返回来的状态
            if post_transfer_data.code == 0 {post_transfer_data.code = 2100;}  // 设置一个初始值
            // 对状态进行赋值
            if post_transfer_data.irreversible && post_transfer_data.is_post_transfer{ // 首先保证不可逆 和post转账
                if post_transfer_data.contract_account == CONTRACT_ACCOUNT.to_vec(){
                    match Self::vec_convert_account(post_transfer_data.pk.clone()){
                        Some(new_acc) =>{
                            if acc == new_acc{
                                post_transfer_data.code = 0;   // 验证通过

                            }else{
                                post_transfer_data.code = 2003;
                            }
                        },
                        None => debug::info!("can not parse account"),
                    }

                }else{
                    post_transfer_data.code = 2002;
                }


            }else{
                debug::info!("可逆 或者 不是post");
                post_transfer_data.code = 2001;
            }

        }
        post_transfer_data.code;
    }


    // 以下两个函数都是以 AuthorityId 作为中介转换
    fn vec_convert_account(acc: Vec<u8>) -> Option<T::AccountId>{
        // 将 Vec<u8> 转换为 accountId
        // debug::info!("------ acc ={:?} -------", hex::encode(&acc.clone()));
        if acc.len() != 32{
            debug::error!("acc len={:?}",acc.len());
            return None;
        }
        let acc_u8: [u8;32]= acc.as_slice().try_into().expect("");
        let authority_id: T::AuthorityId = acc_u8.into();
        // debug::info!("authority_id is {:?}",authority_id);
        Some(authority_id.into_account32())

    }

    fn account_convert_u8(acc: T::AccountId) -> Vec<u8>{
        /// 将账号转换为字符串(公钥)
        debug::info!("acc={:?}",acc);
        let author: T::AuthorityId = acc.into();
        debug::info!("author={:?}",author);
        let author_vec = author.to_raw_vec();
        hex_to_u8(&author_vec)
    }
}


#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
    type Call = Call<T>;

    fn validate_unsigned(
        _source: TransactionSource,
        call: &Self::Call,
    ) -> TransactionValidity {
        let now = <timestamp::Module<T>>::get();
        debug::info!("--------------validate_unsigned time:{:?}--------------------",now);
        match call {   // Call::record_address(block_num,account_id,key,tx,.., signature)
            Call::record_suc_verify(block_num,account,key,tx,status,quantity,signature) => {
                debug::info!("############## record_suc_verify : now = {:?},block_num = {:?}##############",now,block_num);

                // check signature (this is expensive so we do it last).
                let signature_valid = &(block_num,account,tx).using_encoded(|encoded_sign| {
                    key.verify(&encoded_sign, &signature)
                });

                if !signature_valid {
                    debug::error!("................ record_suc_verify 签名验证失败 .....................");
                    return InvalidTransaction::BadProof.into();
                }
                debug::info!("................ record_suc_verify 签名验证成功 .....................");
                Ok(ValidTransaction {
                    priority: <T as Trait>::UnsignedPriority::get(),
                    requires: vec![],
                    provides: vec![(block_num,tx,status,account).encode()],
                    longevity: TransactionLongevity::max_value(),
                    propagate: true,
                })
            },

            Call::record_fail_verify(block,account,key,tx,err,signature) => {
                debug::info!("############# record_fail_verify :block={:?},time={:?}##############",block,now);
                // check signature (this is expensive so we do it last).
                let signature_valid = &(block,account,tx).using_encoded(|encoded_sign| {
                    key.verify(&encoded_sign, &signature)
                });
                if !signature_valid {
                    debug::error!("................ record_fail_verify 签名验证失败 .....................");
                    return InvalidTransaction::BadProof.into();
                }
                Ok(ValidTransaction {
                    priority: <T as Trait>::UnsignedPriority::get(),
                    requires: vec![],
                    provides: vec![(block,tx,err,account).encode()], // vec![(now).encode()],
                    longevity: TransactionLongevity::max_value()-1,
                    propagate: true,
                })
            },

            _ => InvalidTransaction::Call.into()
        }
    }
}







