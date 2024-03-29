// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
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

use crate::params::node_key_params::NodeKeyParams;
use sc_network::{
	config::{NetworkConfiguration, NodeKeyConfig, NonReservedPeerMode, TransportConfig},
	multiaddr::Protocol,
};
use sc_service::{
	config::{Multiaddr, MultiaddrWithPeerId},
	ChainSpec,
};
use std::path::PathBuf;
use structopt::StructOpt;

/// Parameters used to create the network configuration.
#[derive(Debug, StructOpt)]
pub struct NetworkParams {
	/// Specify a list of bootnodes.
	#[structopt(long = "bootnodes", value_name = "ADDR")]
	pub bootnodes: Vec<MultiaddrWithPeerId>,

	/// Specify a list of reserved node addresses.
	#[structopt(long = "reserved-nodes", value_name = "ADDR")]
	pub reserved_nodes: Vec<MultiaddrWithPeerId>,

	/// Whether to only allow connections to/from reserved nodes.
	///
	/// If you are a validator your node might still connect to other validator
	/// nodes regardless of whether they are defined as reserved nodes.
	#[structopt(long = "reserved-only")]
	pub reserved_only: bool,

	/// The public address that other nodes will use to connect to it.
	/// This can be used if there's a proxy in front of this node.
	#[structopt(long, value_name = "PUBLIC_ADDR")]
	pub public_addr: Vec<Multiaddr>,

	/// Listen on this multiaddress.
	#[structopt(long = "listen-addr", value_name = "LISTEN_ADDR")]
	pub listen_addr: Vec<Multiaddr>,

	/// Specify p2p protocol TCP port.
	#[structopt(long = "port", value_name = "PORT", conflicts_with_all = &[ "listen-addr" ])]
	pub port: Option<u16>,

	/// Forbid connecting to private IPv4 addresses (as specified in
	/// [RFC1918](https://tools.ietf.org/html/rfc1918)), unless the address was passed with
	/// `--reserved-nodes` or `--bootnodes`.
	#[structopt(long = "no-private-ipv4")]
	pub no_private_ipv4: bool,

	/// Specify the number of outgoing connections we're trying to maintain.
	#[structopt(long = "out-peers", value_name = "COUNT", default_value = "25")]
	pub out_peers: u32,

	/// Specify the maximum number of incoming connections we're accepting.
	#[structopt(long = "in-peers", value_name = "COUNT", default_value = "25")]
	pub in_peers: u32,

	/// Disable mDNS discovery.
	///
	/// By default, the network will use mDNS to discover other nodes on the
	/// local network. This disables it. Automatically implied when using --dev.
	#[structopt(long = "no-mdns")]
	pub no_mdns: bool,

	/// Maximum number of peers from which to ask for the same blocks in parallel.
	///
	/// This allows downloading announced blocks from multiple peers. Decrease to save
	/// traffic and risk increased latency.
	#[structopt(long = "max-parallel-downloads", value_name = "COUNT", default_value = "5")]
	pub max_parallel_downloads: u32,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub node_key_params: NodeKeyParams,

	/// Disable the yamux flow control. This option will be removed in the future once there is
	/// enough confidence that this feature is properly working.
	#[structopt(long)]
	pub no_yamux_flow_control: bool,

	/// Enable peer discovery on local networks.
	///
	/// By default this option is true for `--dev` and false otherwise.
	#[structopt(long)]
	pub discover_local: bool,
}

impl NetworkParams {
	/// Fill the given `NetworkConfiguration` by looking at the cli parameters.
	pub fn network_config(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
		is_dev: bool,
		net_config_path: Option<PathBuf>,
		client_id: &str,
		node_name: &str,
		node_key: NodeKeyConfig,
		default_listen_port: u16,
	) -> NetworkConfiguration {
		let port = self.port.unwrap_or(default_listen_port);

		let listen_addresses = if self.listen_addr.is_empty() {
			vec![
				Multiaddr::empty()
					.with(Protocol::Ip6([0, 0, 0, 0, 0, 0, 0, 0].into()))
					.with(Protocol::Tcp(port)),
				Multiaddr::empty()
					.with(Protocol::Ip4([0, 0, 0, 0].into()))
					.with(Protocol::Tcp(port)),
			]
		} else {
			self.listen_addr.clone()
		};

		let public_addresses = self.public_addr.clone();

		let mut boot_nodes = chain_spec.boot_nodes().to_vec();
		boot_nodes.extend(self.bootnodes.clone());

		NetworkConfiguration {
			boot_nodes,
			net_config_path,
			reserved_nodes: self.reserved_nodes.clone(),
			non_reserved_mode: if self.reserved_only {
				NonReservedPeerMode::Deny
			} else {
				NonReservedPeerMode::Accept
			},
			listen_addresses,
			public_addresses,
			notifications_protocols: Vec::new(),
			request_response_protocols: Vec::new(),
			node_key,
			node_name: node_name.to_string(),
			client_version: client_id.to_string(),
			in_peers: self.in_peers,
			out_peers: self.out_peers,
			transport: TransportConfig::Normal {
				enable_mdns: !is_dev && !self.no_mdns,
				allow_private_ipv4: !self.no_private_ipv4,
				wasm_external_transport: None,
				use_yamux_flow_control: !self.no_yamux_flow_control,
			},
			max_parallel_downloads: self.max_parallel_downloads,
			allow_non_globals_in_dht: self.discover_local || is_dev,
		}
	}
}
