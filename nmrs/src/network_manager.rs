use zbus::Connection;

use crate::Result;
use crate::connection::{connect, forget};
use crate::connection_settings::{get_saved_connection_path, has_saved_connection};
use crate::device::{list_devices, set_wifi_enabled, wait_for_wifi_ready, wifi_enabled};
use crate::models::{Device, Network, NetworkInfo, WifiSecurity};
use crate::network_info::{current_connection_info, current_ssid, show_details};
use crate::network_monitor;
use crate::scan::{list_networks, scan_networks};

/// High-level interface to NetworkManager over D-Bus.
///
/// Provides methods for listing devices, scanning networks, connecting,
/// and managing saved connections.
#[derive(Clone)]
pub struct NetworkManager {
    conn: Connection,
}

impl NetworkManager {
    /// Creates a new `NetworkManager` connected to the system D-Bus.
    pub async fn new() -> Result<Self> {
        let conn = Connection::system().await?;
        Ok(Self { conn })
    }

    /// Lists all network devices managed by NetworkManager.
    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        list_devices(&self.conn).await
    }

    /// Lists all visible Wi-Fi networks.
    pub async fn list_networks(&self) -> Result<Vec<Network>> {
        list_networks(&self.conn).await
    }

    /// Connects to a Wi-Fi network with the given credentials.
    ///
    /// # Errors
    ///
    /// Returns `ConnectionError::NotFound` if the network is not visible,
    /// `ConnectionError::AuthFailed` if authentication fails, or other
    /// variants for specific failure reasons.
    pub async fn connect(&self, ssid: &str, creds: WifiSecurity) -> Result<()> {
        connect(&self.conn, ssid, creds).await
    }

    /// Returns whether Wi-Fi is currently enabled.
    pub async fn wifi_enabled(&self) -> Result<bool> {
        wifi_enabled(&self.conn).await
    }

    /// Enables or disables Wi-Fi.
    pub async fn set_wifi_enabled(&self, value: bool) -> Result<()> {
        set_wifi_enabled(&self.conn, value).await
    }

    /// Waits for a Wi-Fi device to become ready (disconnected or activated).
    pub async fn wait_for_wifi_ready(&self) -> Result<()> {
        wait_for_wifi_ready(&self.conn).await
    }

    /// Triggers a Wi-Fi scan on all wireless devices.
    pub async fn scan_networks(&self) -> Result<()> {
        scan_networks(&self.conn).await
    }

    /// Returns the SSID of the currently connected network, if any.
    pub async fn current_ssid(&self) -> Option<String> {
        current_ssid(&self.conn).await
    }

    /// Returns the SSID and frequency of the current connection, if any.
    pub async fn current_connection_info(&self) -> Option<(String, Option<u32>)> {
        current_connection_info(&self.conn).await
    }

    /// Returns detailed information about a specific network.
    pub async fn show_details(&self, net: &Network) -> Result<NetworkInfo> {
        show_details(&self.conn, net).await
    }

    /// Returns whether a saved connection exists for the given SSID.
    pub async fn has_saved_connection(&self, ssid: &str) -> Result<bool> {
        has_saved_connection(&self.conn, ssid).await
    }

    /// Returns the D-Bus object path of a saved connection for the given SSID.
    pub async fn get_saved_connection_path(
        &self,
        ssid: &str,
    ) -> Result<Option<zvariant::OwnedObjectPath>> {
        get_saved_connection_path(&self.conn, ssid).await
    }

    /// Forgets (deletes) a saved connection for the given SSID.
    ///
    /// If currently connected to this network, disconnects first.
    pub async fn forget(&self, ssid: &str) -> Result<()> {
        forget(&self.conn, ssid).await
    }

    /// Monitors Wi-Fi network changes in real-time.
    ///
    /// Subscribes to D-Bus signals for access point additions and removals
    /// on all Wi-Fi devices. Invokes the callback whenever the network list
    /// changes, enabling live UI updates without polling.
    ///
    /// This function runs indefinitely until an error occurs. Run it in a
    /// background task.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use nmrs::NetworkManager;
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    ///
    /// // Spawn monitoring task
    /// glib::MainContext::default().spawn_local({
    ///     let nm = nm.clone();
    ///     async move {
    ///         nm.monitor_network_changes(|| {
    ///             println!("Networks changed!");
    ///         }).await
    ///     }
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub async fn monitor_network_changes<F>(&self, callback: F) -> Result<()>
    where
        F: Fn() + 'static,
    {
        network_monitor::monitor_network_changes(&self.conn, callback).await
    }
}
