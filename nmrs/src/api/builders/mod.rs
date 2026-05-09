//! Connection builders for different network types.
//!
//! This module provides two complementary APIs for constructing NetworkManager
//! settings dictionaries:
//!
//! - **Fluent builder types** that support method chaining and perform
//!   validation when `.build()` is called.
//! - **Free `build_*` functions** that take already-prepared structs and
//!   produce the same settings map directly.
//!
//! # Fluent builders
//!
//! - [`ConnectionBuilder`] — generic, low-level builder used as the foundation
//!   for all other builders. Supports IPv4/IPv6 method, manual addresses,
//!   routes, DNS, autoconnect, MTU, and more. See [`IpConfig`] / [`Route`].
//! - [`WifiConnectionBuilder`] — Wi-Fi (open / WPA-PSK / WPA-EAP), with band,
//!   channel, hidden SSID, BSSID pinning, and AP-mode shortcuts. See
//!   [`WifiBand`] / [`WifiMode`].
//! - [`WireGuardBuilder`] — kernel-level WireGuard tunnels.
//! - [`OpenVpnBuilder`] — NM-plugin OpenVPN connections, with
//!   [`from_ovpn_file`](OpenVpnBuilder::from_ovpn_file) for `.ovpn` import.
//!
//! # Free functions
//!
//! - [`build_wifi_connection`] / [`build_ethernet_connection`] (in [`wifi`])
//! - [`build_wireguard_connection`] / [`build_openvpn_connection`] (in [`vpn`])
//! - [`build_bluetooth_connection`] (in [`bluetooth`])
//! - [`build_vlan_connection`] (in [`vlan`])
//!
//! # When to use these
//!
//! Most users should use the high-level
//! [`NetworkManager`](crate::NetworkManager) API instead of calling these
//! builders directly. They are exposed for advanced use cases where you
//! need fine-grained control over the raw settings dictionary before
//! handing it to NetworkManager's `AddConnection` or
//! `AddAndActivateConnection` D-Bus methods.
//!
//! # Examples
//!
//! ## Wi-Fi (free function)
//!
//! ```rust
//! use nmrs::builders::{build_ethernet_connection, build_wifi_connection};
//! use nmrs::{ConnectionOptions, WifiSecurity};
//!
//! let opts = ConnectionOptions::new(true).with_priority(10);
//!
//! let wifi = build_wifi_connection(
//!     "MyNetwork",
//!     &WifiSecurity::WpaPsk { psk: "password".into() },
//!     &opts,
//! );
//! let eth = build_ethernet_connection("eth0", &opts);
//! ```
//!
//! ## WireGuard (fluent builder)
//!
//! ```rust
//! use nmrs::builders::WireGuardBuilder;
//! use nmrs::WireGuardPeer;
//!
//! let peer = WireGuardPeer::new(
//!     "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
//!     "vpn.example.com:51820",
//!     vec!["0.0.0.0/0".into()],
//! ).with_persistent_keepalive(25);
//!
//! let settings = WireGuardBuilder::new("MyVPN")
//!     .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
//!     .address("10.0.0.2/24")
//!     .add_peer(peer)
//!     .dns(vec!["1.1.1.1".into()])
//!     .build()
//!     .expect("WireGuardBuilder is fully configured");
//! ```
//!
//! The returned settings can then be passed to NetworkManager's
//! `AddConnection` or `AddAndActivateConnection` D-Bus methods.

pub mod bluetooth;
pub mod connection_builder;
pub mod openvpn_builder;
pub mod vlan;
pub mod vpn;
pub mod wifi;
pub mod wifi_builder;
pub mod wireguard_builder;

// Re-export core builder types
pub use connection_builder::{ConnectionBuilder, IpConfig, Route};
pub use openvpn_builder::OpenVpnBuilder;
pub use wifi_builder::{WifiBand, WifiConnectionBuilder, WifiMode};
pub use wireguard_builder::WireGuardBuilder;

// Re-export builder functions for convenience
pub use bluetooth::build_bluetooth_connection;
pub use vlan::build_vlan_connection;
pub use vpn::{build_openvpn_connection, build_wireguard_connection};
pub use wifi::{build_ethernet_connection, build_wifi_connection};
