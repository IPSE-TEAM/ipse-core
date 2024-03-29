// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shareable Substrate traits.

use crate::{
	crypto::{CryptoTypePublicPair, KeyTypeId},
	ecdsa, ed25519, sr25519,
	vrf::{VRFSignature, VRFTranscriptData},
};
use std::{
	borrow::Cow,
	fmt::{Debug, Display},
	panic::UnwindSafe,
	sync::Arc,
};

pub use sp_externalities::{Externalities, ExternalitiesExt};

/// BareCryptoStore error
#[derive(Debug, derive_more::Display)]
pub enum Error {
	/// Public key type is not supported
	#[display(fmt = "Key not supported: {:?}", _0)]
	KeyNotSupported(KeyTypeId),
	/// Pair not found for public key and KeyTypeId
	#[display(fmt = "Pair was not found: {}", _0)]
	PairNotFound(String),
	/// Validation error
	#[display(fmt = "Validation error: {}", _0)]
	ValidationError(String),
	/// Keystore unavailable
	#[display(fmt = "Keystore unavailable")]
	Unavailable,
	/// Programming errors
	#[display(fmt = "An unknown keystore error occurred: {}", _0)]
	Other(String),
}

/// Something that generates, stores and provides access to keys.
pub trait BareCryptoStore: Send + Sync {
	/// Returns all sr25519 public keys for the given key type.
	fn sr25519_public_keys(&self, id: KeyTypeId) -> Vec<sr25519::Public>;
	/// Generate a new sr25519 key pair for the given key type and an optional seed.
	///
	/// If the given seed is `Some(_)`, the key pair will only be stored in memory.
	///
	/// Returns the public key of the generated key pair.
	fn sr25519_generate_new(
		&mut self,
		id: KeyTypeId,
		seed: Option<&str>,
	) -> Result<sr25519::Public, Error>;
	/// Returns all ed25519 public keys for the given key type.
	fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public>;
	/// Generate a new ed25519 key pair for the given key type and an optional seed.
	///
	/// If the given seed is `Some(_)`, the key pair will only be stored in memory.
	///
	/// Returns the public key of the generated key pair.
	fn ed25519_generate_new(
		&mut self,
		id: KeyTypeId,
		seed: Option<&str>,
	) -> Result<ed25519::Public, Error>;
	/// Returns all ecdsa public keys for the given key type.
	fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public>;
	/// Generate a new ecdsa key pair for the given key type and an optional seed.
	///
	/// If the given seed is `Some(_)`, the key pair will only be stored in memory.
	///
	/// Returns the public key of the generated key pair.
	fn ecdsa_generate_new(
		&mut self,
		id: KeyTypeId,
		seed: Option<&str>,
	) -> Result<ecdsa::Public, Error>;

	/// Insert a new key. This doesn't require any known of the crypto; but a public key must be
	/// manually provided.
	///
	/// Places it into the file system store.
	///
	/// `Err` if there's some sort of weird filesystem error, but should generally be `Ok`.
	fn insert_unknown(
		&mut self,
		_key_type: KeyTypeId,
		_suri: &str,
		_public: &[u8],
	) -> Result<(), ()>;

	/// Get the password for this store.
	fn password(&self) -> Option<&str>;
	/// Find intersection between provided keys and supported keys
	///
	/// Provided a list of (CryptoTypeId,[u8]) pairs, this would return
	/// a filtered set of public keys which are supported by the keystore.
	fn supported_keys(
		&self,
		id: KeyTypeId,
		keys: Vec<CryptoTypePublicPair>,
	) -> Result<Vec<CryptoTypePublicPair>, Error>;
	/// List all supported keys
	///
	/// Returns a set of public keys the signer supports.
	fn keys(&self, id: KeyTypeId) -> Result<Vec<CryptoTypePublicPair>, Error>;

	/// Checks if the private keys for the given public key and key type combinations exist.
	///
	/// Returns `true` iff all private keys could be found.
	fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool;

	/// Sign with key
	///
	/// Signs a message with the private key that matches
	/// the public key passed.
	///
	/// Returns the SCALE encoded signature if key is found & supported,
	/// an error otherwise.
	fn sign_with(
		&self,
		id: KeyTypeId,
		key: &CryptoTypePublicPair,
		msg: &[u8],
	) -> Result<Vec<u8>, Error>;

	/// Sign with any key
	///
	/// Given a list of public keys, find the first supported key and
	/// sign the provided message with that key.
	///
	/// Returns a tuple of the used key and the SCALE encoded signature.
	fn sign_with_any(
		&self,
		id: KeyTypeId,
		keys: Vec<CryptoTypePublicPair>,
		msg: &[u8],
	) -> Result<(CryptoTypePublicPair, Vec<u8>), Error> {
		if keys.len() == 1 {
			return self.sign_with(id, &keys[0], msg).map(|s| (keys[0].clone(), s))
		} else {
			for k in self.supported_keys(id, keys)? {
				if let Ok(sign) = self.sign_with(id, &k, msg) {
					return Ok((k, sign))
				}
			}
		}
		Err(Error::KeyNotSupported(id))
	}

	/// Sign with all keys
	///
	/// Provided a list of public keys, sign a message with
	/// each key given that the key is supported.
	///
	/// Returns a list of `Result`s each representing the SCALE encoded
	/// signature of each key or a Error for non-supported keys.
	fn sign_with_all(
		&self,
		id: KeyTypeId,
		keys: Vec<CryptoTypePublicPair>,
		msg: &[u8],
	) -> Result<Vec<Result<Vec<u8>, Error>>, ()> {
		Ok(keys.iter().map(|k| self.sign_with(id, k, msg)).collect())
	}

	/// Generate VRF signature for given transcript data.
	///
	/// Receives KeyTypeId and Public key to be able to map
	/// them to a private key that exists in the keystore which
	/// is, in turn, used for signing the provided transcript.
	///
	/// Returns a result containing the signature data.
	/// Namely, VRFOutput and VRFProof which are returned
	/// inside the `VRFSignature` container struct.
	///
	/// This function will return an error in the cases where
	/// the public key and key type provided do not match a private
	/// key in the keystore. Or, in the context of remote signing
	/// an error could be a network one.
	fn sr25519_vrf_sign(
		&self,
		key_type: KeyTypeId,
		public: &sr25519::Public,
		transcript_data: VRFTranscriptData,
	) -> Result<VRFSignature, Error>;
}

/// A pointer to the key store.
pub type BareCryptoStorePtr = Arc<parking_lot::RwLock<dyn BareCryptoStore>>;

sp_externalities::decl_extension! {
	/// The keystore extension to register/retrieve from the externalities.
	pub struct KeystoreExt(BareCryptoStorePtr);
}

/// Code execution engine.
pub trait CodeExecutor: Sized + Send + Sync + CallInWasm + Clone + 'static {
	/// Externalities error type.
	type Error: Display + Debug + Send + 'static;

	/// Call a given method in the runtime. Returns a tuple of the result (either the output data
	/// or an execution error) together with a `bool`, which is true if native execution was used.
	fn call<R: codec::Codec + PartialEq, NC: FnOnce() -> Result<R, String> + UnwindSafe>(
		&self,
		ext: &mut dyn Externalities,
		runtime_code: &RuntimeCode,
		method: &str,
		data: &[u8],
		use_native: bool,
		native_call: Option<NC>,
	) -> (Result<crate::NativeOrEncoded<R>, Self::Error>, bool);
}

/// Something that can fetch the runtime `:code`.
pub trait FetchRuntimeCode {
	/// Fetch the runtime `:code`.
	///
	/// If the `:code` could not be found/not available, `None` should be returned.
	fn fetch_runtime_code<'a>(&'a self) -> Option<Cow<'a, [u8]>>;
}

/// Wrapper to use a `u8` slice or `Vec` as [`FetchRuntimeCode`].
pub struct WrappedRuntimeCode<'a>(pub std::borrow::Cow<'a, [u8]>);

impl<'a> FetchRuntimeCode for WrappedRuntimeCode<'a> {
	fn fetch_runtime_code<'b>(&'b self) -> Option<Cow<'b, [u8]>> {
		Some(self.0.as_ref().into())
	}
}

/// Type that implements [`FetchRuntimeCode`] and always returns `None`.
pub struct NoneFetchRuntimeCode;

impl FetchRuntimeCode for NoneFetchRuntimeCode {
	fn fetch_runtime_code<'a>(&'a self) -> Option<Cow<'a, [u8]>> {
		None
	}
}

/// The Wasm code of a Substrate runtime.
#[derive(Clone)]
pub struct RuntimeCode<'a> {
	/// The code fetcher that can be used to lazily fetch the code.
	pub code_fetcher: &'a dyn FetchRuntimeCode,
	/// The optional heap pages this `code` should be executed with.
	///
	/// If `None` are given, the default value of the executor will be used.
	pub heap_pages: Option<u64>,
	/// The SCALE encoded hash of `code`.
	///
	/// The hashing algorithm isn't that important, as long as all runtime
	/// code instances use the same.
	pub hash: Vec<u8>,
}

impl<'a> PartialEq for RuntimeCode<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.hash == other.hash
	}
}

impl<'a> RuntimeCode<'a> {
	/// Create an empty instance.
	///
	/// This is only useful for tests that don't want to execute any code.
	pub fn empty() -> Self {
		Self { code_fetcher: &NoneFetchRuntimeCode, hash: Vec::new(), heap_pages: None }
	}
}

impl<'a> FetchRuntimeCode for RuntimeCode<'a> {
	fn fetch_runtime_code<'b>(&'b self) -> Option<Cow<'b, [u8]>> {
		self.code_fetcher.fetch_runtime_code()
	}
}

/// Could not find the `:code` in the externalities while initializing the [`RuntimeCode`].
#[derive(Debug)]
pub struct CodeNotFound;

impl std::fmt::Display for CodeNotFound {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "the storage entry `:code` doesn't have any code")
	}
}

/// `Allow` or `Disallow` missing host functions when instantiating a WASM blob.
#[derive(Clone, Copy, Debug)]
pub enum MissingHostFunctions {
	/// Any missing host function will be replaced by a stub that returns an error when
	/// being called.
	Allow,
	/// Any missing host function will result in an error while instantiating the WASM blob,
	Disallow,
}

impl MissingHostFunctions {
	/// Are missing host functions allowed?
	pub fn allowed(self) -> bool {
		matches!(self, Self::Allow)
	}
}

/// Something that can call a method in a WASM blob.
pub trait CallInWasm: Send + Sync {
	/// Call the given `method` in the given `wasm_blob` using `call_data` (SCALE encoded arguments)
	/// to decode the arguments for the method.
	///
	/// Returns the SCALE encoded return value of the method.
	///
	/// # Note
	///
	/// If `code_hash` is `Some(_)` the `wasm_code` module and instance will be cached internally,
	/// otherwise it is thrown away after the call.
	fn call_in_wasm(
		&self,
		wasm_code: &[u8],
		code_hash: Option<Vec<u8>>,
		method: &str,
		call_data: &[u8],
		ext: &mut dyn Externalities,
		missing_host_functions: MissingHostFunctions,
	) -> Result<Vec<u8>, String>;
}

sp_externalities::decl_extension! {
	/// The call-in-wasm extension to register/retrieve from the externalities.
	pub struct CallInWasmExt(Box<dyn CallInWasm>);
}

impl CallInWasmExt {
	/// Creates a new instance of `Self`.
	pub fn new<T: CallInWasm + 'static>(inner: T) -> Self {
		Self(Box::new(inner))
	}
}

sp_externalities::decl_extension! {
	/// Task executor extension.
	pub struct TaskExecutorExt(Box<dyn SpawnNamed>);
}

impl TaskExecutorExt {
	/// New instance of task executor extension.
	pub fn new(spawn_handle: impl SpawnNamed + Send + 'static) -> Self {
		Self(Box::new(spawn_handle))
	}
}

/// Something that can spawn futures (blocking and non-blocking) with an assigned name.
#[dyn_clonable::clonable]
pub trait SpawnNamed: Clone + Send + Sync {
	/// Spawn the given blocking future.
	///
	/// The given `name` is used to identify the future in tracing.
	fn spawn_blocking(&self, name: &'static str, future: futures::future::BoxFuture<'static, ()>);
	/// Spawn the given non-blocking future.
	///
	/// The given `name` is used to identify the future in tracing.
	fn spawn(&self, name: &'static str, future: futures::future::BoxFuture<'static, ()>);
}

impl SpawnNamed for Box<dyn SpawnNamed> {
	fn spawn_blocking(&self, name: &'static str, future: futures::future::BoxFuture<'static, ()>) {
		(**self).spawn_blocking(name, future)
	}

	fn spawn(&self, name: &'static str, future: futures::future::BoxFuture<'static, ()>) {
		(**self).spawn(name, future)
	}
}
