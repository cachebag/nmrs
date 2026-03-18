use std::fmt::{Display, Formatter};

use super::error::ConnectionError;

/// NetworkManager device state reason codes.
///
/// These values come from the NM D-Bus API and indicate why a device
/// transitioned to its current state. Use `StateReason::from(code)` to
/// convert from the raw u32 values returned by NetworkManager.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateReason {
    /// The reason is unknown.
    Unknown,
    /// No specific reason given.
    None,
    /// The user disconnected the device.
    UserDisconnected,
    /// The device was disconnected by the system.
    DeviceDisconnected,
    /// The carrier/link status changed (e.g., cable unplugged).
    CarrierChanged,
    /// The Wi-Fi supplicant disconnected unexpectedly.
    SupplicantDisconnected,
    /// The Wi-Fi supplicant's configuration failed.
    SupplicantConfigFailed,
    /// The Wi-Fi supplicant failed (authentication issue).
    SupplicantFailed,
    /// The Wi-Fi supplicant timed out during authentication.
    SupplicantTimeout,
    /// PPP connection start failed.
    PppStartFailed,
    /// DHCP client failed to start.
    DhcpStartFailed,
    /// DHCP client encountered an error.
    DhcpError,
    /// DHCP client failed to obtain an IP address.
    DhcpFailed,
    /// Modem connection failed.
    ModemConnectionFailed,
    /// Modem initialization failed.
    ModemInitFailed,
    /// InfiniBand device mode mismatch.
    InfinibandMode,
    /// A dependency connection failed.
    DependencyFailed,
    /// BR2684 bridge setup failed.
    Br2684Failed,
    /// Failed to set the device mode (e.g., AP mode).
    ModeSetFailed,
    /// GSM modem APN selection failed.
    GsmApnSelectFailed,
    /// GSM modem is not searching for networks.
    GsmNotSearching,
    /// GSM network registration was denied.
    GsmRegistrationDenied,
    /// GSM network registration timed out.
    GsmRegistrationTimeout,
    /// GSM network registration failed.
    GsmRegistrationFailed,
    /// GSM SIM PIN check failed.
    GsmPinCheckFailed,
    /// Required firmware is missing for the device.
    FirmwareMissing,
    /// The device was removed from the system.
    DeviceRemoved,
    /// The system is entering sleep mode.
    Sleeping,
    /// The connection profile was removed.
    ConnectionRemoved,
    /// The user requested the operation.
    UserRequested,
    /// Carrier status changed.
    Carrier,
    /// NetworkManager assumed an existing connection.
    ConnectionAssumed,
    /// The Wi-Fi supplicant became available.
    SupplicantAvailable,
    /// The modem device was not found.
    ModemNotFound,
    /// Bluetooth connection failed.
    BluetoothFailed,
    /// GSM SIM card is not inserted.
    GsmSimNotInserted,
    /// GSM SIM PIN is required.
    GsmSimPinRequired,
    /// GSM SIM PUK is required.
    GsmSimPukRequired,
    /// Wrong GSM SIM card inserted.
    GsmSimWrong,
    /// The requested SSID was not found.
    SsidNotFound,
    /// A secondary connection failed.
    SecondaryConnectionFailed,
    /// DCB/FCoE setup failed.
    DcbFcoeFailed,
    /// teamd control interface failed.
    TeamdControlFailed,
    /// Modem operation failed.
    ModemFailed,
    /// Modem became available.
    ModemAvailable,
    /// SIM PIN was incorrect.
    SimPinIncorrect,
    /// A new connection activation was queued.
    NewActivationEnqueued,
    /// Parent device became unreachable.
    ParentUnreachable,
    /// Parent device changed.
    ParentChanged,
    /// Unknown reason code not mapped to a specific variant.
    Other(u32),
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
#[must_use]
pub fn reason_to_error(code: u32) -> ConnectionError {
    let reason = StateReason::from(code);
    match reason {
        StateReason::SupplicantFailed
        | StateReason::SupplicantDisconnected
        | StateReason::SimPinIncorrect
        | StateReason::GsmPinCheckFailed => ConnectionError::AuthFailed,

        StateReason::SupplicantConfigFailed => ConnectionError::SupplicantConfigFailed,

        StateReason::SupplicantTimeout => ConnectionError::SupplicantTimeout,

        StateReason::DhcpStartFailed | StateReason::DhcpError | StateReason::DhcpFailed => {
            ConnectionError::DhcpFailed
        }

        StateReason::SsidNotFound => ConnectionError::NotFound,

        _ => ConnectionError::DeviceFailed(reason),
    }
}
