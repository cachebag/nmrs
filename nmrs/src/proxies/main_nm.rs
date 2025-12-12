//! Main NetworkManager proxy.

use std::collections::HashMap;
use zbus::proxy;
use zvariant::OwnedObjectPath;

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
