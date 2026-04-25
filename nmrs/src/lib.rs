//! A Rust library for managing network connections via NetworkManager.
//!
//! This crate provides a high-level async API for NetworkManager over D-Bus,
//! enabling easy management of WiFi, Ethernet, and VPN connections on Linux.
//!
//! # Quick Start
//!
//! ## WiFi Connection
//!
//! ```rust
//! use nmrs::{NetworkManager, WifiSecurity};
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//!
//! // List visible networks (None = all Wi-Fi devices)
//! let networks = nm.list_networks(None).await?;
//! for net in &networks {
//!     println!("{} - Signal: {}%", net.ssid, net.strength.unwrap_or(0));
//! }
//!
//! // Connect to a network on the first Wi-Fi device
//! nm.connect("MyNetwork", None, WifiSecurity::WpaPsk {
//!     psk: "password123".into()
//! }).await?;
//!
//! // Check current connection
//! if let Some(ssid) = nm.current_ssid().await {
//!     println!("Connected to: {}", ssid);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## VPN Connection (WireGuard)
//!
//! ```rust
//! use nmrs::{NetworkManager, WireGuardConfig, WireGuardPeer};
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//!
//! let peer = WireGuardPeer::new(
//!     "peer_public_key",
//!     "vpn.example.com:51820",
//!     vec!["0.0.0.0/0".into()],
//! ).with_persistent_keepalive(25);
//!
//! let config = WireGuardConfig::new(
//!     "MyVPN",
//!     "vpn.example.com:51820",
//!     "your_private_key",
//!     "10.0.0.2/24",
//!     vec![peer],
//! ).with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()]);
//!
//! nm.connect_vpn(config).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## VPN Connection (OpenVPN)
//!
//! ```rust
//! use nmrs::{NetworkManager, OpenVpnConfig, OpenVpnAuthType};
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//!
//! let config = OpenVpnConfig::new("CorpVPN", "vpn.example.com", 1194, false)
//!     .with_auth_type(OpenVpnAuthType::PasswordTls)
//!     .with_username("user")
//!     .with_password("secret")
//!     .with_ca_cert("/etc/openvpn/ca.crt")
//!     .with_client_cert("/etc/openvpn/client.crt")
//!     .with_client_key("/etc/openvpn/client.key");
//!
//! nm.connect_vpn(config).await?;
//!
//! // Or import an .ovpn file directly:
//! nm.import_ovpn("corp.ovpn", Some("user"), Some("secret")).await?;
//!
//! // List VPN connections
//! let vpns = nm.list_vpn_connections().await?;
//! for vpn in vpns {
//!     println!("{}: {:?} - {:?}", vpn.name, vpn.vpn_type, vpn.state);
//! }
//!
//! nm.disconnect_vpn("CorpVPN").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Core Concepts
//!
//! ## NetworkManager
//!
//! The main entry point is [`NetworkManager`], which provides methods for:
//! - Listing and managing network devices
//! - Scanning for available WiFi networks
//! - Connecting to networks (WiFi, Ethernet, VPN)
//! - Managing saved connection profiles
//! - Real-time monitoring of network changes
//!
//! ## Models
//!
//! The [`models`] module contains all types, enums, and errors:
//! - [`Device`] - Represents a network device (WiFi, Ethernet, etc.)
//! - [`Network`] - Represents a discovered WiFi network
//! - [`WifiSecurity`] - Security types (Open, WPA-PSK, WPA-EAP)
//! - [`VpnCredentials`] - Legacy VPN connection credentials
//! - [`VpnType`] - Supported VPN types (WireGuard, OpenVPN)
//! - [`VpnConnection`] - VPN connection information
//! - [`VpnDetails`] - Protocol-specific VPN details (WireGuard / OpenVPN)
//! - [`WireGuardConfig`] - WireGuard connection configuration
//! - [`WireGuardPeer`] - WireGuard peer configuration
//! - [`OpenVpnConfig`] - OpenVPN connection configuration
//! - [`OpenVpnAuthType`] - OpenVPN authentication types
//! - [`ConnectionError`] - Comprehensive error types
//!
//! ## Connection Builders
//!
//! The [`builders`] module provides functions to construct connection settings
//! for different network types. These are typically used internally but exposed
//! for advanced use cases.
//!
//! # Examples
//!
//! ## Connecting to Different Network Types
//!
//! ```rust
//! use nmrs::{NetworkManager, WifiSecurity, EapOptions, EapMethod, Phase2};
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//!
//! // Open network
//! nm.connect("OpenWiFi", None, WifiSecurity::Open).await?;
//!
//! // WPA-PSK (password-protected)
//! nm.connect("HomeWiFi", None, WifiSecurity::WpaPsk {
//!     psk: "my_password".into()
//! }).await?;
//!
//! // WPA-EAP (Enterprise)
//! let eap_opts = EapOptions::new("user@company.com", "password")
//!     .with_domain_suffix_match("company.com")
//!     .with_system_ca_certs(true)
//!     .with_method(EapMethod::Peap)
//!     .with_phase2(Phase2::Mschapv2);
//!
//! nm.connect("CorpWiFi", None, WifiSecurity::WpaEap {
//!     opts: eap_opts
//! }).await?;
//!
//! // Ethernet (auto-connects when cable is plugged in)
//! nm.connect_wired().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All operations return [`Result<T>`], which is an alias for `Result<T, ConnectionError>`.
//! The [`ConnectionError`] type provides specific variants for different failure modes:
//!
//! ```rust
//! use nmrs::{NetworkManager, WifiSecurity, ConnectionError};
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//!
//! match nm.connect("MyNetwork", None, WifiSecurity::WpaPsk {
//!     psk: "wrong_password".into()
//! }).await {
//!     Ok(_) => println!("Connected successfully"),
//!     Err(ConnectionError::AuthFailed) => {
//!         eprintln!("Wrong password!");
//!     }
//!     Err(ConnectionError::NotFound) => {
//!         eprintln!("Network not found or out of range");
//!     }
//!     Err(ConnectionError::Timeout) => {
//!         eprintln!("Connection timed out");
//!     }
//!     Err(ConnectionError::DhcpFailed) => {
//!         eprintln!("Failed to obtain IP address");
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Device Management
//!
//! ```rust
//! use nmrs::NetworkManager;
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//!
//! // List all devices
//! let devices = nm.list_devices().await?;
//! for device in devices {
//!     println!("{}: {} ({})",
//!         device.interface,
//!         device.device_type,
//!         device.state
//!     );
//! }
//!
//! // Enable/disable WiFi
//! nm.set_wireless_enabled(false).await?;
//! nm.set_wireless_enabled(true).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Real-Time Monitoring
//!
//! Monitor network and device changes in real-time using D-Bus signals:
//!
//! ```rust
//! use nmrs::NetworkManager;
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//!
//! // Monitor network changes (new networks, signal changes, etc.)
//! nm.monitor_network_changes(|| {
//!     println!("Networks changed! Refresh your UI.");
//! }).await?;
//!
//! // Monitor device state changes (cable plugged in, device activated, etc.)
//! nm.monitor_device_changes(|| {
//!     println!("Device state changed!");
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! This crate uses D-Bus signals for efficient state monitoring instead of polling.
//! When connecting to a network, it subscribes to NetworkManager's `StateChanged`
//! signals to detect connection success or failure immediately. This provides:
//!
//! - **Faster response times** - Immediate notification vs polling delay
//! - **Lower CPU usage** - No spinning loops
//! - **Better error messages** - Specific failure reasons from NetworkManager
//!
//! # Logging
//!
//! This crate uses the [`log`](https://docs.rs/log) facade. To see log output,
//! add a logging implementation like `env_logger`:
//!
//! ```no_run,ignore
//! env_logger::init();
//! ```
//!
//! # Feature Flags
//!
//! This crate currently has no optional features. All functionality is enabled by default.
//!
//! # Platform Support
//!
//! This crate is Linux-only and requires:
//! - NetworkManager running and accessible via D-Bus
//! - Appropriate permissions to manage network connections

// Internal modules (not exposed in public API)
mod api;
mod core;
mod dbus;
mod monitoring;
mod types;
mod util;

/// NetworkManager secret agent for credential prompting over D-Bus.
///
/// See the [module documentation](agent) for the three-stream model,
/// lifecycle, and a full example.
pub mod agent;

// ============================================================================
// Public API
// ============================================================================

/// Connection builders for WiFi, Ethernet, and VPN connections.
///
/// This module provides functions to construct NetworkManager connection settings
/// dictionaries. These are primarily used internally but exposed for advanced use cases.
///
/// # Examples
///
/// ```rust
/// use nmrs::builders::build_wifi_connection;
/// use nmrs::{WifiSecurity, ConnectionOptions};
///
/// let opts = ConnectionOptions::new(true);
///
/// let settings = build_wifi_connection(
///     "MyNetwork",
///     &WifiSecurity::Open,
///     &opts
/// );
/// ```
pub mod builders {
    pub use crate::api::builders::*;
}

/// Types, enums, and errors for NetworkManager operations.
///
/// This module contains all the public types used throughout the crate:
///
/// # Core Types
/// - [`NetworkManager`] - Main API entry point
/// - [`Device`] - Network device representation
/// - [`Network`] - WiFi network representation
/// - [`NetworkInfo`] - Detailed network information
///
/// # Configuration
/// - [`WifiSecurity`] - WiFi security types (Open, WPA-PSK, WPA-EAP)
/// - [`EapOptions`] - Enterprise authentication options
/// - [`ConnectionOptions`] - Connection settings (autoconnect, priority, etc.)
/// - [`TimeoutConfig`] - Timeout configuration for network operations
///
/// # Enums
/// - [`DeviceType`] - Device types (Ethernet, WiFi, etc.)
/// - [`DeviceState`] - Device states (Disconnected, Activated, etc.)
/// - [`EapMethod`] - EAP authentication methods
/// - [`Phase2`] - Phase 2 authentication for EAP
///
/// # Errors
/// - [`ConnectionError`] - Comprehensive error type for all operations
/// - [`StateReason`] - Device state change reasons
/// - [`ConnectionStateReason`] - Connection state change reasons
///
/// # Helper Functions
/// - [`reason_to_error`] - Convert device state reason to error
/// - [`connection_state_reason_to_error`] - Convert connection state reason to error
pub mod models {
    pub use crate::api::models::*;
}

// Re-export commonly used types at crate root for convenience
#[allow(deprecated)]
pub use api::models::{
    AccessPoint, ActiveConnectionState, AirplaneModeState, ApMode, BluetoothDevice,
    BluetoothIdentity, BluetoothNetworkRole, ConnectType, ConnectionError, ConnectionOptions,
    ConnectionStateReason, Device, DeviceState, DeviceType, EapMethod, EapOptions, Network,
    NetworkInfo, OpenVpnAuthType, OpenVpnCompression, OpenVpnConfig, OpenVpnProxy, Phase2,
    RadioState, SecurityFeatures, StateReason, TimeoutConfig, VpnConfig, VpnConfiguration,
    VpnConnection, VpnConnectionInfo, VpnCredentials, VpnDetails, VpnRoute, VpnType, WifiDevice,
    WifiSecurity, WireGuardConfig, WireGuardPeer, connection_state_reason_to_error,
    reason_to_error,
};
pub use api::network_manager::NetworkManager;
pub use api::wifi_scope::WifiScope;

/// A specialized `Result` type for network operations.
///
/// This is an alias for `Result<T, ConnectionError>` and is used throughout
/// the crate for all fallible operations.
///
/// # Examples
///
/// ```rust
/// use nmrs::Result;
///
/// async fn connect_to_wifi() -> Result<()> {
///     // Your code here
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, ConnectionError>;
