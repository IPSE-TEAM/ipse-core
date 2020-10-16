
extern crate frame_system as system;
extern crate pallet_timestamp as timestamp;

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::DispatchResult, debug,
    weights::Weight,
};

use system::{ensure_signed};
use sp_runtime::traits::SaturatedConversion;
use sp_std::vec::Vec;
use sp_std::vec;

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}


decl_storage! {
    trait Store for Module<T: Trait> as IpseStakingModule {

    }
}

decl_event! {
pub enum Event<T>
    where
    AccountId = <T as system::Trait>::AccountId
    {
        Staked(AccountId),
    }
}

decl_module! {
     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;


     }
}

impl<T: Trait> Module<T> {


}
