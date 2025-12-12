//! D-Bus proxy traits for NetworkManager interfaces.
//!
//! These traits define the NetworkManager D-Bus API surface used by this crate.
//! The `zbus::proxy` macro generates proxy implementations that handle
//! D-Bus communication automatically.
//!
//! # NetworkManager D-Bus Structure
//!
//! - `/org/freedesktop/NetworkManager` - Main NM object
//! - `/org/freedesktop/NetworkManager/Devices/*` - Device objects
//! - `/org/freedesktop/NetworkManager/AccessPoint/*` - Access point objects
//! - `/org/freedesktop/NetworkManager/Settings` - Connection settings

use std::collections::HashMap;
use zbus::{Result, proxy};
use zvariant::OwnedObjectPath;

/// Proxy for NetworkManager device interface.
///
/// Provides access to device properties like interface name, type, state,
/// and the reason for state transitions.
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NMDevice {
    /// The network interface name (e.g., "wlan0").
    #[zbus(property)]
    fn interface(&self) -> Result<String>;

    /// Device type as a numeric code (2 = Wi-Fi).
    #[zbus(property)]
    fn device_type(&self) -> Result<u32>;

    /// Current device state (100 = activated, 120 = failed).
    #[zbus(property)]
    fn state(&self) -> Result<u32>;

    /// Whether NetworkManager manages this device.
    #[zbus(property)]
    fn managed(&self) -> Result<bool>;

    /// The kernel driver in use.
    #[zbus(property)]
    fn driver(&self) -> Result<String>;

    /// Current state and reason code for the last state change.
    #[zbus(property)]
    fn state_reason(&self) -> Result<(u32, u32)>;
}

/// Proxy for the main NetworkManager interface.
///
/// Provides methods for listing devices, managing connections,
/// and controlling Wi-Fi state.
#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub trait NM {
    /// Returns paths to all network devices.
    fn get_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    /// Whether Wi-Fi is globally enabled.
    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;

    /// Enable or disable Wi-Fi globally.
    #[zbus(property)]
    fn set_wireless_enabled(&self, value: bool) -> zbus::Result<()>;

    /// Paths to all active connections.
    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    /// Creates a new connection and activates it simultaneously.
    ///
    /// Returns paths to both the new connection settings and active connection.
    fn add_and_activate_connection(
        &self,
        connection: HashMap<&str, HashMap<&str, zvariant::Value<'_>>>,
        device: OwnedObjectPath,
        specific_object: OwnedObjectPath,
    ) -> zbus::Result<(OwnedObjectPath, OwnedObjectPath)>;

    /// Activates an existing saved connection.
    fn activate_connection(
        &self,
        connection: OwnedObjectPath,
        device: OwnedObjectPath,
        specific_object: OwnedObjectPath,
    ) -> zbus::Result<OwnedObjectPath>;

    /// Deactivates an active connection.
    fn deactivate_connection(&self, active_connection: OwnedObjectPath) -> zbus::Result<()>;
}

/// Proxy for wireless device interface.
///
/// Extends the base device interface with Wi-Fi specific functionality
/// like scanning and access point enumeration.
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NMWireless {
    /// Returns paths to all visible access points.
    fn get_all_access_points(&self) -> Result<Vec<OwnedObjectPath>>;

    /// Requests a Wi-Fi scan. Options are usually empty.
    fn request_scan(&self, options: HashMap<String, zvariant::Value<'_>>) -> Result<()>;

    /// Signal emitted when a new access point is discovered.
    #[zbus(signal)]
    fn access_point_added(&self, path: OwnedObjectPath);

    /// Signal emitted when an access point is no longer visible.
    #[zbus(signal)]
    fn access_point_removed(&self, path: OwnedObjectPath);

    /// Path to the currently connected access point ("/" if none).
    #[zbus(property)]
    fn active_access_point(&self) -> Result<OwnedObjectPath>;

    /// Current connection bitrate in Kbit/s.
    #[zbus(property)]
    fn bitrate(&self) -> Result<u32>;
}

/// Proxy for access point interface.
///
/// Provides information about a visible Wi-Fi network including
/// SSID, signal strength, security capabilities, and frequency.
#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NMAccessPoint {
    /// SSID as raw bytes (may not be valid UTF-8).
    #[zbus(property)]
    fn ssid(&self) -> Result<Vec<u8>>;

    /// Signal strength as percentage (0-100).
    #[zbus(property)]
    fn strength(&self) -> Result<u8>;

    /// BSSID (MAC address) of the access point.
    #[zbus(property)]
    fn hw_address(&self) -> Result<String>;

    /// General capability flags (bit 0 = privacy/WEP).
    #[zbus(property)]
    fn flags(&self) -> Result<u32>;

    /// WPA security flags (PSK, EAP, etc.).
    #[zbus(property)]
    fn wpa_flags(&self) -> Result<u32>;

    /// RSN/WPA2 security flags.
    #[zbus(property)]
    fn rsn_flags(&self) -> Result<u32>;

    /// Operating frequency in MHz.
    #[zbus(property)]
    fn frequency(&self) -> Result<u32>;

    /// Maximum supported bitrate in Kbit/s.
    #[zbus(property)]
    fn max_bitrate(&self) -> Result<u32>;

    /// Wi-Fi mode (1 = adhoc, 2 = infrastructure, 3 = AP).
    #[zbus(property)]
    fn mode(&self) -> Result<u32>;
}
