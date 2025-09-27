use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    pub ssid: String,
    pub strength: u8,
    pub secure: bool,
}

#[derive(Debug, Clone)]
pub struct Device {
    pub path: String,
    pub interface: String,
    pub device_type: DeviceType,
    pub state: DeviceState,
    pub managed: Option<bool>,
    pub driver: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    Ethernet,
    Wifi,
    WifiP2P,
    Loopback,
    Other(u32),
}

#[derive(Debug, Clone)]
pub enum DeviceState {
    Unmanaged,
    Unavailable,
    Disconnected,
    Prepare,
    Config,
    Activated,
    Other(u32),
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

impl From<u32> for DeviceType {
    fn from(value: u32) -> Self {
        match value {
            1 => DeviceType::Ethernet,
            2 => DeviceType::Wifi,
            30 => DeviceType::WifiP2P,
            32 => DeviceType::Loopback,
            v => DeviceType::Other(v),
        }
    }
}

impl From<u32> for DeviceState {
    fn from(value: u32) -> Self {
        match value {
            10 => DeviceState::Unmanaged,
            20 => DeviceState::Unavailable,
            30 => DeviceState::Disconnected,
            40 => DeviceState::Prepare,
            50 => DeviceState::Config,
            100 => DeviceState::Activated,
            v => DeviceState::Other(v),
        }
    }
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Ethernet => write!(f, "Ethernet"),
            DeviceType::Wifi => write!(f, "Wi-Fi"),
            DeviceType::WifiP2P => write!(f, "Wi-Fi P2P"),
            DeviceType::Loopback => write!(f, "Loopback"),
            DeviceType::Other(v) => write!(f, "Other({})", v),
        }
    }
}

impl Display for DeviceState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceState::Unmanaged => write!(f, "Unmanaged"),
            DeviceState::Unavailable => write!(f, "Unavailable"),
            DeviceState::Disconnected => write!(f, "Disconnected"),
            DeviceState::Prepare => write!(f, "Preparing"),
            DeviceState::Config => write!(f, "Configuring"),
            DeviceState::Activated => write!(f, "Activated"),
            DeviceState::Other(v) => write!(f, "Other({})", v),
        }
    }
}
