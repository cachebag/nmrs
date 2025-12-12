//! NetworkManager Wireless Device proxy.

use std::collections::HashMap;
use zbus::{Result, proxy};
use zvariant::OwnedObjectPath;

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
