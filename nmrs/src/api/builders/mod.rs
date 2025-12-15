//! Connection builders for different network types.
//!
//! This module provides functions to construct NetworkManager connection settings
//! dictionaries for various connection types. These settings are used with
//! NetworkManager's D-Bus API to create and activate connections.
//!
//! # Available Builders
//!
//! - [`wifi`] - WiFi connection builders (WPA-PSK, WPA-EAP, Open)
//! - Ethernet builders (via [`build_ethernet_connection`])
//! - VPN builders (coming in future releases)
//!
//! # When to Use These
//!
//! Most users should use the high-level [`NetworkManager`](crate::NetworkManager) API
//! instead of calling these builders directly. These are exposed for advanced use cases
//! where you need fine-grained control over connection settings.
//!
//! # Examples
//!
//! ```rust
//! use nmrs::builders::{build_wifi_connection, build_ethernet_connection};
//! use nmrs::{WifiSecurity, ConnectionOptions};
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
//! ```

pub mod vpn;
pub mod wifi;

// Re-export builder functions for convenience
pub use vpn::build_wireguard_connection;
pub use wifi::{build_ethernet_connection, build_wifi_connection};
