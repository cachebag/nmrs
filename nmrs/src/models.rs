use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// NetworkManager device state reason codes.
///
/// These values come from the NM D-Bus API and indicate why a device
/// transitioned to its current state. Use `StateReason::from(code)` to
/// convert from the raw u32 values returned by NetworkManager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateReason {
    Unknown,
    None,
    UserDisconnected,
    DeviceDisconnected,
    CarrierChanged,
    SupplicantDisconnected,
    SupplicantConfigFailed,
    SupplicantFailed,
    SupplicantTimeout,
    PppStartFailed,
    DhcpStartFailed,
    DhcpError,
    DhcpFailed,
    ModemConnectionFailed,
    ModemInitFailed,
    InfinibandMode,
    DependencyFailed,
    Br2684Failed,
    ModeSetFailed,
    GsmApnSelectFailed,
    GsmNotSearching,
    GsmRegistrationDenied,
    GsmRegistrationTimeout,
    GsmRegistrationFailed,
    GsmPinCheckFailed,
    FirmwareMissing,
    DeviceRemoved,
    Sleeping,
    ConnectionRemoved,
    UserRequested,
    Carrier,
    ConnectionAssumed,
    SupplicantAvailable,
    ModemNotFound,
    BluetoothFailed,
    GsmSimNotInserted,
    GsmSimPinRequired,
    GsmSimPukRequired,
    GsmSimWrong,
    SsidNotFound,
    SecondaryConnectionFailed,
    DcbFcoeFailed,
    TeamdControlFailed,
    ModemFailed,
    ModemAvailable,
    SimPinIncorrect,
    NewActivationEnqueued,
    ParentUnreachable,
    ParentChanged,
    /// Unknown reason code not mapped to a specific variant.
    Other(u32),
}

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

/// Errors that can occur during network operations.
#[derive(Debug, Error)]
pub enum ConnectionError {
    /// A D-Bus communication error occurred.
    #[error("D-Bus error: {0}")]
    Dbus(#[from] zbus::Error),

    /// The requested network was not found during scan.
    #[error("network not found")]
    NotFound,

    /// Authentication with the access point failed (wrong password, rejected credentials).
    #[error("authentication failed")]
    AuthFailed,

    /// The supplicant (wpa_supplicant) encountered a configuration error.
    #[error("supplicant configuration failed")]
    SupplicantConfigFailed,

    /// The supplicant timed out during authentication.
    #[error("supplicant timeout")]
    SupplicantTimeout,

    /// DHCP failed to obtain an IP address.
    #[error("DHCP failed")]
    DhcpFailed,

    /// The connection timed out waiting for activation.
    #[error("connection timeout")]
    Timeout,

    /// The connection is stuck in an unexpected state.
    #[error("connection stuck in state: {0}")]
    Stuck(String),

    /// No Wi-Fi device was found on the system.
    #[error("no Wi-Fi device found")]
    NoWifiDevice,

    /// Wi-Fi device did not become ready in time.
    #[error("Wi-Fi device not ready")]
    WifiNotReady,

    /// No saved connection exists for the requested network.
    #[error("no saved connection for network")]
    NoSavedConnection,

    /// A general connection failure with a reason code from NetworkManager.
    #[error("connection failed: {0}")]
    Failed(StateReason),

    /// Invalid UTF-8 encountered in SSID.
    #[error("invalid UTF-8 in SSID: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),
}

impl From<u32> for StateReason {
    fn from(code: u32) -> Self {
        match code {
            0 => Self::Unknown,
            1 => Self::None,
            2 => Self::UserDisconnected,
            3 => Self::DeviceDisconnected,
            4 => Self::CarrierChanged,
            7 => Self::SupplicantDisconnected,
            8 => Self::SupplicantConfigFailed,
            9 => Self::SupplicantFailed,
            10 => Self::SupplicantTimeout,
            11 => Self::PppStartFailed,
            15 => Self::DhcpStartFailed,
            16 => Self::DhcpError,
            17 => Self::DhcpFailed,
            24 => Self::ModemConnectionFailed,
            25 => Self::ModemInitFailed,
            42 => Self::InfinibandMode,
            43 => Self::DependencyFailed,
            44 => Self::Br2684Failed,
            45 => Self::ModeSetFailed,
            46 => Self::GsmApnSelectFailed,
            47 => Self::GsmNotSearching,
            48 => Self::GsmRegistrationDenied,
            49 => Self::GsmRegistrationTimeout,
            50 => Self::GsmRegistrationFailed,
            51 => Self::GsmPinCheckFailed,
            52 => Self::FirmwareMissing,
            53 => Self::DeviceRemoved,
            54 => Self::Sleeping,
            55 => Self::ConnectionRemoved,
            56 => Self::UserRequested,
            57 => Self::Carrier,
            58 => Self::ConnectionAssumed,
            59 => Self::SupplicantAvailable,
            60 => Self::ModemNotFound,
            61 => Self::BluetoothFailed,
            62 => Self::GsmSimNotInserted,
            63 => Self::GsmSimPinRequired,
            64 => Self::GsmSimPukRequired,
            65 => Self::GsmSimWrong,
            70 => Self::SsidNotFound,
            71 => Self::SecondaryConnectionFailed,
            72 => Self::DcbFcoeFailed,
            73 => Self::TeamdControlFailed,
            74 => Self::ModemFailed,
            75 => Self::ModemAvailable,
            76 => Self::SimPinIncorrect,
            77 => Self::NewActivationEnqueued,
            78 => Self::ParentUnreachable,
            79 => Self::ParentChanged,
            v => Self::Other(v),
        }
    }
}

impl Display for StateReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::None => write!(f, "none"),
            Self::UserDisconnected => write!(f, "user disconnected"),
            Self::DeviceDisconnected => write!(f, "device disconnected"),
            Self::CarrierChanged => write!(f, "carrier changed"),
            Self::SupplicantDisconnected => write!(f, "supplicant disconnected"),
            Self::SupplicantConfigFailed => write!(f, "supplicant config failed"),
            Self::SupplicantFailed => write!(f, "supplicant failed"),
            Self::SupplicantTimeout => write!(f, "supplicant timeout"),
            Self::PppStartFailed => write!(f, "PPP start failed"),
            Self::DhcpStartFailed => write!(f, "DHCP start failed"),
            Self::DhcpError => write!(f, "DHCP error"),
            Self::DhcpFailed => write!(f, "DHCP failed"),
            Self::ModemConnectionFailed => write!(f, "modem connection failed"),
            Self::ModemInitFailed => write!(f, "modem init failed"),
            Self::InfinibandMode => write!(f, "infiniband mode"),
            Self::DependencyFailed => write!(f, "dependency failed"),
            Self::Br2684Failed => write!(f, "BR2684 failed"),
            Self::ModeSetFailed => write!(f, "mode set failed"),
            Self::GsmApnSelectFailed => write!(f, "GSM APN select failed"),
            Self::GsmNotSearching => write!(f, "GSM not searching"),
            Self::GsmRegistrationDenied => write!(f, "GSM registration denied"),
            Self::GsmRegistrationTimeout => write!(f, "GSM registration timeout"),
            Self::GsmRegistrationFailed => write!(f, "GSM registration failed"),
            Self::GsmPinCheckFailed => write!(f, "GSM PIN check failed"),
            Self::FirmwareMissing => write!(f, "firmware missing"),
            Self::DeviceRemoved => write!(f, "device removed"),
            Self::Sleeping => write!(f, "sleeping"),
            Self::ConnectionRemoved => write!(f, "connection removed"),
            Self::UserRequested => write!(f, "user requested"),
            Self::Carrier => write!(f, "carrier"),
            Self::ConnectionAssumed => write!(f, "connection assumed"),
            Self::SupplicantAvailable => write!(f, "supplicant available"),
            Self::ModemNotFound => write!(f, "modem not found"),
            Self::BluetoothFailed => write!(f, "bluetooth failed"),
            Self::GsmSimNotInserted => write!(f, "GSM SIM not inserted"),
            Self::GsmSimPinRequired => write!(f, "GSM SIM PIN required"),
            Self::GsmSimPukRequired => write!(f, "GSM SIM PUK required"),
            Self::GsmSimWrong => write!(f, "GSM SIM wrong"),
            Self::SsidNotFound => write!(f, "SSID not found"),
            Self::SecondaryConnectionFailed => write!(f, "secondary connection failed"),
            Self::DcbFcoeFailed => write!(f, "DCB/FCoE setup failed"),
            Self::TeamdControlFailed => write!(f, "teamd control failed"),
            Self::ModemFailed => write!(f, "modem failed"),
            Self::ModemAvailable => write!(f, "modem available"),
            Self::SimPinIncorrect => write!(f, "SIM PIN incorrect"),
            Self::NewActivationEnqueued => write!(f, "new activation enqueued"),
            Self::ParentUnreachable => write!(f, "parent device unreachable"),
            Self::ParentChanged => write!(f, "parent device changed"),
            Self::Other(v) => write!(f, "unknown reason ({v})"),
        }
    }
}

/// Converts a NetworkManager state reason code to a specific `ConnectionError`.
///
/// Maps authentication-related failures to `AuthFailed`, DHCP issues to `DhcpFailed`,
/// and other failures to the appropriate variant.
pub fn reason_to_error(code: u32) -> ConnectionError {
    let reason = StateReason::from(code);
    match reason {
        // Authentication failures
        StateReason::SupplicantFailed
        | StateReason::SupplicantDisconnected
        | StateReason::SimPinIncorrect
        | StateReason::GsmPinCheckFailed => ConnectionError::AuthFailed,

        // Supplicant configuration issues
        StateReason::SupplicantConfigFailed => ConnectionError::SupplicantConfigFailed,

        // Supplicant timeout
        StateReason::SupplicantTimeout => ConnectionError::SupplicantTimeout,

        // DHCP failures
        StateReason::DhcpStartFailed | StateReason::DhcpError | StateReason::DhcpFailed => {
            ConnectionError::DhcpFailed
        }

        // Network not found
        StateReason::SsidNotFound => ConnectionError::NotFound,

        // All other failures
        _ => ConnectionError::Failed(reason),
    }
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

    #[test]
    fn state_reason_from_u32_known_codes() {
        assert_eq!(StateReason::from(0), StateReason::Unknown);
        assert_eq!(StateReason::from(1), StateReason::None);
        assert_eq!(StateReason::from(7), StateReason::SupplicantDisconnected);
        assert_eq!(StateReason::from(8), StateReason::SupplicantConfigFailed);
        assert_eq!(StateReason::from(9), StateReason::SupplicantFailed);
        assert_eq!(StateReason::from(10), StateReason::SupplicantTimeout);
        assert_eq!(StateReason::from(16), StateReason::DhcpError);
        assert_eq!(StateReason::from(17), StateReason::DhcpFailed);
        assert_eq!(StateReason::from(70), StateReason::SsidNotFound);
        assert_eq!(StateReason::from(76), StateReason::SimPinIncorrect);
    }

    #[test]
    fn state_reason_from_u32_unknown_code() {
        assert_eq!(StateReason::from(999), StateReason::Other(999));
        assert_eq!(StateReason::from(255), StateReason::Other(255));
    }

    #[test]
    fn state_reason_display() {
        assert_eq!(format!("{}", StateReason::Unknown), "unknown");
        assert_eq!(
            format!("{}", StateReason::SupplicantFailed),
            "supplicant failed"
        );
        assert_eq!(format!("{}", StateReason::DhcpFailed), "DHCP failed");
        assert_eq!(format!("{}", StateReason::SsidNotFound), "SSID not found");
        assert_eq!(
            format!("{}", StateReason::Other(123)),
            "unknown reason (123)"
        );
    }

    #[test]
    fn reason_to_error_auth_failures() {
        // Supplicant failures indicate auth issues
        assert!(matches!(reason_to_error(9), ConnectionError::AuthFailed));
        assert!(matches!(reason_to_error(7), ConnectionError::AuthFailed));
        assert!(matches!(reason_to_error(76), ConnectionError::AuthFailed));
        assert!(matches!(reason_to_error(51), ConnectionError::AuthFailed));
    }

    #[test]
    fn reason_to_error_supplicant_config() {
        assert!(matches!(
            reason_to_error(8),
            ConnectionError::SupplicantConfigFailed
        ));
    }

    #[test]
    fn reason_to_error_supplicant_timeout() {
        assert!(matches!(
            reason_to_error(10),
            ConnectionError::SupplicantTimeout
        ));
    }

    #[test]
    fn reason_to_error_dhcp_failures() {
        assert!(matches!(reason_to_error(15), ConnectionError::DhcpFailed));
        assert!(matches!(reason_to_error(16), ConnectionError::DhcpFailed));
        assert!(matches!(reason_to_error(17), ConnectionError::DhcpFailed));
    }

    #[test]
    fn reason_to_error_network_not_found() {
        assert!(matches!(reason_to_error(70), ConnectionError::NotFound));
    }

    #[test]
    fn reason_to_error_generic_failure() {
        // User disconnected maps to generic Failed
        match reason_to_error(2) {
            ConnectionError::Failed(reason) => {
                assert_eq!(reason, StateReason::UserDisconnected);
            }
            _ => panic!("expected ConnectionError::Failed"),
        }
    }

    #[test]
    fn connection_error_display() {
        assert_eq!(
            format!("{}", ConnectionError::NotFound),
            "network not found"
        );
        assert_eq!(
            format!("{}", ConnectionError::AuthFailed),
            "authentication failed"
        );
        assert_eq!(format!("{}", ConnectionError::DhcpFailed), "DHCP failed");
        assert_eq!(
            format!("{}", ConnectionError::Timeout),
            "connection timeout"
        );
        assert_eq!(
            format!("{}", ConnectionError::NoWifiDevice),
            "no Wi-Fi device found"
        );
        assert_eq!(
            format!("{}", ConnectionError::Stuck("config".into())),
            "connection stuck in state: config"
        );
        assert_eq!(
            format!("{}", ConnectionError::Failed(StateReason::CarrierChanged)),
            "connection failed: carrier changed"
        );
    }
}
