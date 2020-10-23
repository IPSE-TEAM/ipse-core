use codec::{ Encode,Decode };
use sp_std::{prelude::*,convert::TryInto};
use hex;
use pallet_timestamp as timestamp;
use sp_runtime::RuntimeAppPublic;
use frame_support::{Parameter,debug};
use app_crypto::{sr25519};
use frame_system::{self as system};
use sp_core::{crypto::KeyTypeId,offchain::Timestamp};
use pallet_authority_discovery as authority_discovery;
use sp_runtime::{offchain::http};
use alt_serde::{Deserialize, Deserializer};
use frame_support::{StorageMap,StorageValue,traits::{LockableCurrency,Currency}}; // 含有get

pub const CONTRACT_ACCOUNT: &[u8] = b"ipseaccounts";
pub const VERIFY_STATUS: &[u8] = b"verify_status";  // 验证的返回状态
pub const PENDING_TIME_OUT: &'static str = "Error in waiting http response back";
pub const WAIT_HTTP_CONVER_REPONSE: &'static str ="Error in waiting http_result convert response";

#[cfg_attr(feature = "std", derive())]
/// 这个用于表述地址状态
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum AddressStatus{
    Active,  // 已经激活
    InActive,  // 未激活
}

/// enum中derive不了Default
impl Default for AddressStatus{
    fn default() -> Self {
        Self::InActive
    }
}


#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode,Clone,Debug,PartialEq, Eq, Default)]
pub struct PostTxTransferData {
    pub verify_status: u64,
    pub irreversible: bool,
    pub is_post_transfer: bool,

    #[serde(deserialize_with = "de_string_to_bytes")]
    pub contract_account: Vec<u8>,   // 必须要验证合约账号是否一致
    #[serde(deserialize_with = "de_string_to_bytes")]
    pub from: Vec<u8>,
    pub to: Vec<u8>,
    #[serde(deserialize_with = "de_float_to_integer")]
    pub quantity: u64,
    pub memo: Vec<u8>,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(de)?;
    Ok(s.as_bytes().to_vec())
}


pub fn de_float_to_integer<'de, D>(de: D) -> Result<u64, D::Error>
    where D: Deserializer<'de> {
    let f: f64 = Deserialize::deserialize(de)?;
    Ok(f as u64)
}

// post json中常用的关键字符
pub(crate) const POST_KEYWORD:[&[u8]; 5] = [
    b"{",     // {
    b"\"",   // "
    b"\":",  // ":
    b",",    // ,
    b"}"      // }
];


#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode,Clone)]
pub struct FetchFailed<BlockNumber>{
    // 失败的请求
    pub block_num: BlockNumber,
    pub tx: Vec<u8>,
    pub err: Vec<u8>
}


pub type FetchFailedOf<T> = FetchFailed<<T as system::Trait>::BlockNumber>;

pub type BlockNumberOf<T> = <T as system::Trait>::BlockNumber;  // u32
pub type StdResult<T> = core::result::Result<T, &'static str>;

// 为了兼容返回为空的情况
pub type StrDispatchResult = core::result::Result<(), &'static str>;

pub fn vecchars_to_vecbytes <I: IntoIterator<Item = char> + Clone>(it: &I) -> Vec<u8> {
    it.clone().into_iter().map(|c| c as u8).collect::<_>()
}

pub fn int_covert_str(inner: u64) ->Vec<u8>{
    let mut x:u32 = 0;    //位数
    let mut s :Vec<&str> = vec![]; //保存字符串
    loop {
        let r = inner / ((10 as u64).pow(x));
        if r == 0 {
            s.reverse();
            return  s.join("").as_bytes().to_vec();
        }
        let r = r % 10;
        s.push(num_to_char(r));
        x += 1;
    }
}

pub fn num_to_char<'a>(n:u64)->&'a str{
    if n > 10{return ""}
    match n{
        0=>"0",
        1=>"1",
        2=>"2",
        3=>"3",
        4=>"4",
        5=>"5",
        6=>"6",
        7=>"7",
        8=>"8",
        9=>"9",
        _ => {""},
    }
}

pub fn hex_to_u8<'a>(param: &'a [u8]) -> Vec<u8>{
    // 将 param  首先转化为 16进制字符串,然后加上0x  . 将tx等16进制保持字符串传递
    // 例如: param的十六进制形式为0x1122,变为"0x"+"1122"的字符串,然后编码为&[u8]
    let hex_0x = "0x".as_bytes();
    let tx_hex =  hex::encode(param);   // tx_hex 是 16进制的字符串
    let tx_vec = &[hex_0x,tx_hex.as_bytes()].concat();

    return tx_vec.to_vec();
}


pub trait AccountIdPublicConver{
    type AccountId;
    fn into_account32(self)->Self::AccountId; // 转化为accountId
}