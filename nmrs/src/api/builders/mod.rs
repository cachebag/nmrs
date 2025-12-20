//! Connection builders for different network types.
//!
//! This module provides functions to construct NetworkManager connection settings
//! dictionaries for various connection types. These settings are used with
//! NetworkManager's D-Bus API to create and activate connections.
//!
//! # Available Builders
//!
//! - [`wifi`] - WiFi connection builders (WPA-PSK, WPA-EAP, Open)
//! - [`vpn`] - VPN connection builders (WireGuard)
//! - Ethernet builders (via [`build_ethernet_connection`])
//!
//! # When to Use These
//!
//! Most users should use the high-level [`NetworkManager`](crate::NetworkManager) API
//! instead of calling these builders directly. These are exposed for advanced use cases
//! where you need fine-grained control over connection settings.
//!
//! # Examples
//!
//! ```ignore
//! use nmrs::builders::{build_wifi_connection, build_wireguard_connection, build_ethernet_connection};
//! use nmrs::{WifiSecurity, ConnectionOptions, VpnCredentials, VpnType, WireGuardPeer};
//!
//! let opts = ConnectionOptions {
//!     autoconnect: true,
//!     autoconnect_priority: Some(10),
//!     autoconnect_retries: Some(3),
//! };
//!
//! // Build WiFi connection settings
//! let wifi_settings = build_wifi_connection(
//!     "MyNetwork",
//!     &WifiSecurity::WpaPsk { psk: "password".into() },
//!     &opts
//! );
//!
//! // Build Ethernet connection settings
//! let eth_settings = build_ethernet_connection("eth0", &opts);
//! // Build WireGuard VPN connection settings
//! let creds = VpnCredentials {
//!     vpn_type: VpnType::WireGuard,
//!     name: "MyVPN".into(),
//!     gateway: "vpn.example.com:51820".into(),
//!     private_key: "PRIVATE-KEY".into(),
//!     address: "10.0.0.2/24".into(),
//!     peers: vec![WireGuardPeer {
//!         public_key: "PUBLIC-KEY".into(),
//!         gateway: "vpn.example.com:51820".into(),
//!         allowed_ips: vec!["0.0.0.0/0".into()],
//!         preshared_key: None,
//!         persistent_keepalive: Some(25),
//!     }],
//!     dns: None,
//!     mtu: None,
//!     uuid: None,
//! };
//!
//! let vpn_settings = build_wireguard_connection(&creds, &opts).unwrap();
//! ```
//!
//! These settings can then be passed to NetworkManager's
//! `AddConnection` or `AddAndActivateConnection` D-Bus methods.

pub mod bluetooth;
pub mod vpn;
pub mod wifi;

// Re-export builder functions for convenience
pub use bluetooth::build_bluetooth_connection;
pub use vpn::build_wireguard_connection;
pub use wifi::{build_ethernet_connection, build_wifi_connection};
