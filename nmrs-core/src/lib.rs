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
    Network, NetworkInfo, Phase2, WifiSecurity,
};
pub use network_manager::NetworkManager;

// Re-exported types
pub type Result<T> = std::result::Result<T, ConnectionError>;
