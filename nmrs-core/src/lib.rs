// Internal implementation modules (not exposed to external users)
mod connection;
mod connection_settings;
mod constants;
mod device;
mod network_info;
mod proxies;
mod scan;
mod utils;

// Public API modules
pub mod models;
pub mod network_manager;
pub mod wifi_builders;

// Re-exported types
pub type Result<T> = std::result::Result<T, ConnectionError>;

// Re-exported public API
pub use network_manager::NetworkManager;

use crate::models::ConnectionError;
