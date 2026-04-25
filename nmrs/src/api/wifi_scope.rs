//! Per-Wi-Fi-device scoped operations.
//!
//! [`WifiScope`] is a lightweight, ergonomic wrapper around
//! [`NetworkManager`](crate::NetworkManager) that pins every operation to a
//! single Wi-Fi interface. Build it with
//! [`NetworkManager::wifi`](crate::NetworkManager::wifi):
//!
//! ```no_run
//! use nmrs::{NetworkManager, WifiSecurity};
//!
//! # async fn example() -> nmrs::Result<()> {
//! let nm = NetworkManager::new().await?;
//! let wlan1 = nm.wifi("wlan1");
//!
//! wlan1.scan().await?;
//! let networks = wlan1.list_networks().await?;
//! wlan1.connect("Guest", WifiSecurity::Open).await?;
//! # Ok(())
//! # }
//! ```

use crate::Result;
use crate::api::models::access_point::AccessPoint;
use crate::api::models::{Network, WifiSecurity};
use crate::core::connection::{connect, connect_to_bssid, disconnect, forget_by_name_and_type};
use crate::core::scan::{list_access_points, list_networks, scan_networks};
use crate::core::wifi_device::set_wifi_enabled_for_interface;
use crate::types::constants::device_type;

/// Operations scoped to a single Wi-Fi interface.
///
/// Created via [`NetworkManager::wifi`](crate::NetworkManager::wifi).
/// Cheap to construct (`Clone` is fine).
#[derive(Debug, Clone)]
pub struct WifiScope {
    pub(crate) conn: zbus::Connection,
    pub(crate) interface: String,
    pub(crate) timeout_config: crate::api::models::TimeoutConfig,
}

impl WifiScope {
    /// The interface name this scope is pinned to (e.g. `"wlan0"`).
    #[must_use]
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Trigger a Wi-Fi scan on this interface only.
    pub async fn scan(&self) -> Result<()> {
        scan_networks(&self.conn, Some(&self.interface)).await
    }

    /// List visible networks on this interface (grouped by SSID).
    pub async fn list_networks(&self) -> Result<Vec<Network>> {
        list_networks(&self.conn, Some(&self.interface)).await
    }

    /// List individual access points on this interface (one per BSSID).
    pub async fn list_access_points(&self) -> Result<Vec<AccessPoint>> {
        list_access_points(&self.conn, Some(&self.interface)).await
    }

    /// Connect this interface to the given SSID.
    pub async fn connect(&self, ssid: &str, creds: WifiSecurity) -> Result<()> {
        connect(
            &self.conn,
            ssid,
            creds,
            Some(&self.interface),
            Some(self.timeout_config),
        )
        .await
    }

    /// Connect this interface to a specific BSSID for the given SSID.
    pub async fn connect_to_bssid(
        &self,
        ssid: &str,
        bssid: Option<&str>,
        creds: WifiSecurity,
    ) -> Result<()> {
        connect_to_bssid(
            &self.conn,
            ssid,
            bssid,
            creds,
            Some(&self.interface),
            Some(self.timeout_config),
        )
        .await
    }

    /// Disconnect this interface from its active network, if any.
    pub async fn disconnect(&self) -> Result<()> {
        disconnect(&self.conn, Some(&self.interface), Some(self.timeout_config)).await
    }

    /// Enable or disable autoconnect on this interface only.
    ///
    /// Independent of NetworkManager's global Wi-Fi killswitch
    /// ([`set_wireless_enabled`](crate::NetworkManager::set_wireless_enabled)).
    pub async fn set_enabled(&self, enabled: bool) -> Result<()> {
        set_wifi_enabled_for_interface(&self.conn, &self.interface, enabled).await
    }

    /// Forget a saved Wi-Fi connection by SSID.
    ///
    /// Note: NetworkManager keys profiles by SSID, not by interface, so this
    /// forgets the profile globally — but is exposed here for ergonomic use
    /// alongside the other per-scope operations.
    pub async fn forget(&self, ssid: &str) -> Result<()> {
        forget_by_name_and_type(
            &self.conn,
            ssid,
            Some(device_type::WIFI),
            Some(self.timeout_config),
        )
        .await
    }
}
