use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub device: String,
    pub ssid: String,
    pub bssid: Option<String>,
    pub strength: Option<u8>,
    pub frequency: Option<u32>,
    pub secured: bool,
    pub is_psk: bool,
    pub is_eap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub ssid: String,
    pub bssid: String,
    pub strength: u8,
    pub freq: Option<u32>,
    pub channel: Option<u16>,
    pub mode: String,
    pub rate_mbps: Option<u32>,
    pub bars: String,
    pub security: String,
    pub status: String,
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

pub enum EapMethod {
    Peap, // PEAPv0/EAP-MSCHAPv2
    Ttls, // EAP-TTLS
}

pub enum Phase2 {
    Mschapv2,
    Pap,
}

pub struct EapOptions {
    pub identity: String,
    pub password: String,
    pub anonymous_identity: Option<String>,
    pub domain_suffix_match: Option<String>,
    pub ca_cert_path: Option<String>,
    pub system_ca_certs: bool,
    pub method: EapMethod,
    pub phase2: Phase2,
}

pub struct ConnectionOptions {
    pub autoconnect: bool,
    pub autoconnect_priority: Option<i32>,
    pub autoconnect_retries: Option<i32>,
}

pub enum WifiSecurity {
    Open,
    WpaPsk { psk: String },
    WpaEap { opts: EapOptions },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Ethernet,
    Wifi,
    WifiP2P,
    Loopback,
    Other(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    Unmanaged,
    Unavailable,
    Disconnected,
    Prepare,
    Config,
    Activated,
    Deactivating,
    Failed,
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
    #[error("Invalid UTF-8 in SSID: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
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
            110 => DeviceState::Deactivating,
            120 => DeviceState::Failed,
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
            DeviceType::Other(v) => write!(f, "Other({v})"),
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
            DeviceState::Deactivating => write!(f, "Deactivating"),
            DeviceState::Failed => write!(f, "Failed"),
            DeviceState::Other(v) => write!(f, "Other({v})"),
        }
    }
}

impl WifiSecurity {
    pub fn secured(&self) -> bool {
        !matches!(self, WifiSecurity::Open)
    }

    pub fn is_psk(&self) -> bool {
        matches!(self, WifiSecurity::WpaPsk { .. })
    }

    pub fn is_eap(&self) -> bool {
        matches!(self, WifiSecurity::WpaEap { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_type_from_u32_all_variants() {
        assert_eq!(DeviceType::from(1), DeviceType::Ethernet);
        assert_eq!(DeviceType::from(2), DeviceType::Wifi);
        assert_eq!(DeviceType::from(30), DeviceType::WifiP2P);
        assert_eq!(DeviceType::from(32), DeviceType::Loopback);
        assert_eq!(DeviceType::from(999), DeviceType::Other(999));
        assert_eq!(DeviceType::from(0), DeviceType::Other(0));
    }

    #[test]
    fn device_type_display() {
        assert_eq!(format!("{}", DeviceType::Ethernet), "Ethernet");
        assert_eq!(format!("{}", DeviceType::Wifi), "Wi-Fi");
        assert_eq!(format!("{}", DeviceType::WifiP2P), "Wi-Fi P2P");
        assert_eq!(format!("{}", DeviceType::Loopback), "Loopback");
        assert_eq!(format!("{}", DeviceType::Other(42)), "Other(42)");
    }

    #[test]
    fn device_state_from_u32_all_variants() {
        assert_eq!(DeviceState::from(10), DeviceState::Unmanaged);
        assert_eq!(DeviceState::from(20), DeviceState::Unavailable);
        assert_eq!(DeviceState::from(30), DeviceState::Disconnected);
        assert_eq!(DeviceState::from(40), DeviceState::Prepare);
        assert_eq!(DeviceState::from(50), DeviceState::Config);
        assert_eq!(DeviceState::from(100), DeviceState::Activated);
        assert_eq!(DeviceState::from(110), DeviceState::Deactivating);
        assert_eq!(DeviceState::from(120), DeviceState::Failed);
        assert_eq!(DeviceState::from(7), DeviceState::Other(7));
        assert_eq!(DeviceState::from(0), DeviceState::Other(0));
    }

    #[test]
    fn device_state_display() {
        assert_eq!(format!("{}", DeviceState::Unmanaged), "Unmanaged");
        assert_eq!(format!("{}", DeviceState::Unavailable), "Unavailable");
        assert_eq!(format!("{}", DeviceState::Disconnected), "Disconnected");
        assert_eq!(format!("{}", DeviceState::Prepare), "Preparing");
        assert_eq!(format!("{}", DeviceState::Config), "Configuring");
        assert_eq!(format!("{}", DeviceState::Activated), "Activated");
        assert_eq!(format!("{}", DeviceState::Deactivating), "Deactivating");
        assert_eq!(format!("{}", DeviceState::Failed), "Failed");
        assert_eq!(format!("{}", DeviceState::Other(99)), "Other(99)");
    }

    #[test]
    fn wifi_security_open() {
        let open = WifiSecurity::Open;
        assert!(!open.secured());
        assert!(!open.is_psk());
        assert!(!open.is_eap());
    }

    #[test]
    fn wifi_security_psk() {
        let psk = WifiSecurity::WpaPsk {
            psk: "password123".into(),
        };
        assert!(psk.secured());
        assert!(psk.is_psk());
        assert!(!psk.is_eap());
    }

    #[test]
    fn wifi_security_eap() {
        let eap = WifiSecurity::WpaEap {
            opts: EapOptions {
                identity: "user@example.com".into(),
                password: "secret".into(),
                anonymous_identity: None,
                domain_suffix_match: None,
                ca_cert_path: None,
                system_ca_certs: false,
                method: EapMethod::Peap,
                phase2: Phase2::Mschapv2,
            },
        };
        assert!(eap.secured());
        assert!(!eap.is_psk());
        assert!(eap.is_eap());
    }
}
