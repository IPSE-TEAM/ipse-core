// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::chain_spec::ChainSpec;
use browser_utils::{browser_configuration, init_console_log, set_console_error_panic_hook, Client};
use log::info;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

/// Starts the client.
#[wasm_bindgen]
pub async fn start_client(chain_spec: Option<String>, log_level: String) -> Result<Client, JsValue> {
	start_inner(chain_spec, log_level)
		.await
		.map_err(|err| JsValue::from_str(&err.to_string()))
}

async fn start_inner(chain_spec: Option<String>, log_level: String) -> Result<Client, Box<dyn std::error::Error>> {
	set_console_error_panic_hook();
	init_console_log(log::Level::from_str(&log_level)?)?;
	let chain_spec = match chain_spec {
		Some(chain_spec) => {
			ChainSpec::from_json_bytes(chain_spec.as_bytes().to_vec()).map_err(|e| format!("{:?}", e))?
		}
		None => crate::chain_spec::development_config(),
	};

	let config = browser_configuration(chain_spec).await?;

	info!("Substrate browser node");
	info!("✌️  version {}", config.impl_version);
	info!("❤️  by Parity Technologies, 2017-2020");
	info!("📋 Chain specification: {}", config.chain_spec.name());
	info!("🏷 Node name: {}", config.network.node_name);
	info!("👤 Role: {:?}", config.role);

	// Create the service. This is the most heavy initialization step.
	let (task_manager, rpc_handlers) = crate::service::new_light_base(config)
		.map(|(components, rpc_handlers, _, _, _)| (components, rpc_handlers))
		.map_err(|e| format!("{:?}", e))?;

	Ok(browser_utils::start_client(task_manager, rpc_handlers))
}
