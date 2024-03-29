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

pub trait Trait {
	type BlockNumber: codec::Codec + codec::EncodeLike + Default;
	type Origin;
}

frame_support::decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin, system=self {}
}

frame_support::decl_storage! {
	trait Store for Module<T: Trait> as Test {
		pub AppendableDM config(t): double_map hasher(identity) u32, hasher(identity) T::BlockNumber => Vec<u32>;
	}
}

struct Test;

impl Trait for Test {
	type BlockNumber = u32;
	type Origin = ();
}

#[test]
fn init_genesis_config() {
	GenesisConfig::<Test> { t: Default::default() };
}
