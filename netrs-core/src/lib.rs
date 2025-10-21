pub mod config;
pub mod dbus;
pub mod models;
pub mod wifi_builders;
pub type Result<T> = std::result::Result<T, ConnectionError>;

pub use dbus::NetworkManager;

use crate::models::ConnectionError;
