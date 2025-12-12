//! A Rust library for managing Wi-Fi connections via NetworkManager.
//!
//! This crate provides a high-level async API for common Wi-Fi operations:
//!
//! - Listing network devices and visible networks
//! - Connecting to open, WPA-PSK, and WPA-EAP networks
//! - Managing saved connection profiles
//! - Enabling/disabling Wi-Fi
//!
//! # Example
//!
//! ```no_run
//! use nmrs::{NetworkManager, WifiSecurity};
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//!
//! // List visible networks
//! let networks = nm.list_networks().await?;
//! for net in &networks {
//!     println!("{} ({}%)", net.ssid, net.strength.unwrap_or(0));
//! }
//!
//! // Connect to a network
//! nm.connect("MyNetwork", WifiSecurity::WpaPsk {
//!     psk: "password123".into()
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! All operations return `Result<T, ConnectionError>`. The error type provides
//! specific variants for common failures like authentication errors, timeouts,
//! and missing devices.
//!
//! # Logging
//!
//! This crate uses the [`log`](https://docs.rs/log) facade for logging. To see
//! log output, add a logging implementation like `env_logger`:
//!
//! ```no_run
//! fn main() {
//!     env_logger::init();
//!     // ...
//! }
//! ```
//!
//! Then run with `RUST_LOG=nmrs=debug` to see debug output.

// Internal implementation modules
mod connection;
mod connection_settings;
mod constants;
mod device;
mod network_info;
mod proxies;
mod scan;
mod state_wait;
mod utils;

// Public API modules
pub mod models;
pub mod network_manager;
pub mod wifi_builders;

// Re-exported public API
pub use models::{
    ConnectionError, ConnectionOptions, Device, DeviceState, DeviceType, EapMethod, EapOptions,
    Network, NetworkInfo, Phase2, StateReason, WifiSecurity, reason_to_error,
};
pub use network_manager::NetworkManager;

/// A specialized `Result` type for network operations.
pub type Result<T> = std::result::Result<T, ConnectionError>;
