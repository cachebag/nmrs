pub mod config;
pub mod dbus;
pub mod models;
pub type Result<T> = std::result::Result<T, ConnectionError>;

pub use dbus::NetworkManager;

use crate::models::ConnectionError;
