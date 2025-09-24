use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    pub ssid: String,
    pub strength: u8,
    pub secure: bool,
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
