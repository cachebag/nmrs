//! NetworkManager Settings D-Bus proxy (`org.freedesktop.NetworkManager.Settings`).

use zbus::proxy;
use zvariant::OwnedObjectPath;

/// Proxy for `/org/freedesktop/NetworkManager/Settings`.
#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
pub trait NMSettings {
    /// Returns object paths of all saved connection profiles.
    fn list_connections(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    /// Resolves a connection object path by UUID string.
    fn get_connection_by_uuid(&self, uuid: &str) -> zbus::Result<OwnedObjectPath>;

    /// Reload connection profiles from disk.
    fn reload_connections(&self) -> zbus::Result<bool>;
}
