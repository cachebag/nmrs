use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    pub ssid: String,
    pub strength: u8,
    pub secure: bool,
}

// Abstracting way the bus allows us to dump devices into the UI
// without worrying about OwnedObjectPath, zbus::Connection, D-Bus errors, etc.
#[derive(Debug, Clone)]
pub struct Device {
    pub path: String,
    pub interface: String,
    pub device_type: u32,
    pub state: u32,
    pub managed: Option<bool>,
    pub driver: Option<String>,
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("D-Bus error: {0}")]
    Dbus(#[from] zbus::Error),
    #[error("Network not found")]
    NotFound,
    #[error("Authentication failed")]
    AuthFailed,
}
