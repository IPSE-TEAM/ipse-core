#![cfg_attr(not(feature = "std"), no_std)]
#![feature(option_result_contains)]

use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage, decl_error, ensure,
    dispatch::DispatchResult,
    storage::IterableStorageMap,
    traits::{Currency, ReservableCurrency, Get},
    weights::{
        DispatchClass, constants::WEIGHT_PER_MICROS
    }
};
use sp_std::{str, convert::TryInto, prelude::*, collections::btree_set::BTreeSet};
use frame_system::{self as system, ensure_root, ensure_signed};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use primitives::{
    constants::storagereport::*,
    MerkleRoot, StoragePubKey, SworkerSignature,
    ReportSlot, BlockNumber, IASSig,
    ISVBody, SworkerCert, SworkerCode
};

/// Provides crypto and other std functions by implementing `runtime_interface`
pub mod api;

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Identity {
    pub pub_key: StoragePubKey,
    pub code: SworkerCode,
}

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct StorageReport {
    pub block_number: u64,
    pub used: u64,
    pub reserved: u64,
    pub cached_reserved: u64,
    pub files: Vec<(MerkleRoot, u64)>,
}

/// An event handler for reporting storage
pub trait Storagereport<AccountId> {
    fn report_storage(controller: &AccountId, own_workload: u128, total_workload: u128);
}

impl<AId> Storagereport<AId> for () {
    fn report_storage(_: &AId, _: u128, _: u128) {}
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// The payment balance.
    type Currency: ReservableCurrency<Self::AccountId>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// The handler for reporting storage.
    type Storagereport: Storagereport<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Swork {
        /// The sWorker enclave code, this should be managed by sudo/democracy
        pub Code get(fn code) config(): SworkerCode;

        /// The AB upgrade expired block, this should be managed by sudo/democracy
        pub ABExpire get(fn ab_expire): Option<T::BlockNumber>;

        /// The sWorker identities, mapping from controller to an optional identity tuple
        /// (elder_id, current_id) = (before-upgrade identity, upgraded identity)
        pub Identities get(fn identities) config():
            map hasher(blake2_128_concat) T::AccountId => (Option<Identity>, Option<Identity>);

        /// Node's storage report, mapping from controller to an optional storage report
        pub storageReports get(fn work_reports) config():
            map hasher(blake2_128_concat) T::AccountId  => Option<StorageReport>;

        /// The current report slot block number, this value should be a multiple of era block
        pub CurrentReportSlot get(fn current_report_slot) config(): ReportSlot;

        /// Recording whether the validator reported works of each era
        /// We leave it keep all era's report info
        /// cause B-tree won't build index on key2(ReportSlot)
        /// value (bool, bool) represent two id (elder_reported, current_reported)
        pub ReportedInSlot get(fn reported_in_slot) build(|config: &GenesisConfig<T>| {
            config.work_reports.iter().map(|(account_id, _)|
                (account_id.clone(), 0, (false, true))
            ).collect::<Vec<_>>()
        }): double_map hasher(twox_64_concat) T::AccountId, hasher(twox_64_concat) ReportSlot
        => (bool, bool) = (false, false);

        /// The used workload, used for calculating stake limit in the end of era
        /// default is 0
        pub Used get(fn used): u128 = 0;

        /// The reserved workload, used for calculating stake limit in the end of era
        /// default is 0
        pub Reserved get(fn reserved): u128 = 0;
    }
}


decl_error! {
    /// Error for the storagereport module.
    pub enum Error for Module<T: Trait> {
        /// Illegal applier
        IllegalApplier,
        /// Duplicate identity
        DuplicateId,
        /// Identity check failed
        IllegalTrustedChain,
        /// Illegal reporter
        IllegalReporter,
        /// Invalid public key
        InvalidPubKey,
        /// Invalid timing
        InvalidReportTime,
        /// Illegal work report signature
        IllegalStorageReportSig
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        /// Register as new trusted node,can noly called from storagenode.
        /// All `inputs` can only be generated from storagenode's enclave.
        pub fn register(
            origin,
            ias_sig: IASSig,
            ias_cert: SworkerCert,
            applier: T::AccountId,
            isv_body: ISVBody,
            sig: SworkerSignature
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // 1. Ensure who is applier
            ensure!(&who == &applier, Error::<T>::IllegalApplier);

            // 2. Ensure unparsed_identity trusted chain id legal
            let maybe_pk = Self::check_and_get_pk(&ias_sig, &ias_cert, &applier, &isv_body, &sig);
            ensure!(maybe_pk.is_some(), Error::<T>::IllegalTrustedChain);

            // 3. Ensure public key is unique
            let pk = maybe_pk.unwrap();
            ensure!(Self::id_is_unique(&pk), Error::<T>::DuplicateId);

            // 4. Construct the identity
            let identity = Identity {
                pub_key: pk,
                code: Self::code()
            };

            // 5. Applier is new add or needs to be updated
            if Self::maybe_upsert_id(&applier, &identity) {
                // Emit event
                Self::deposit_event(RawEvent::RegisterSuccess(who));
            }

            Ok(())
        }

        /// Report storage report from storagenode.
        /// All `inputs` can only be generated from storagenode's enclave
        pub fn report_works(
            origin,
            pub_key: StoragePubKey,
            block_number: u64,
            block_hash: Vec<u8>,
            reserved: u64,
            files: Vec<(MerkleRoot, u64)>,
            sig: StorageSignature
        ) -> DispatchResult {
            // todo
        }
    }
}