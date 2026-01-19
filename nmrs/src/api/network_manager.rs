use tokio::sync::watch;
use zbus::Connection;

use crate::api::models::{Device, Network, NetworkInfo, WifiSecurity};
use crate::core::connection::{
    connect, connect_wired, disconnect, forget, get_device_by_interface, is_connected,
};
use crate::core::connection_settings::{
    get_saved_connection_path, has_saved_connection, list_saved_connections,
};
use crate::core::device::{list_devices, set_wifi_enabled, wait_for_wifi_ready, wifi_enabled};
use crate::core::scan::{current_network, list_networks, scan_networks};
use crate::core::vpn::{connect_vpn, disconnect_vpn, get_vpn_info, list_vpn_connections};
use crate::models::{VpnConnection, VpnConnectionInfo, VpnCredentials};
use crate::monitoring::device as device_monitor;
use crate::monitoring::info::{current_connection_info, current_ssid, show_details};
use crate::monitoring::network as network_monitor;
use crate::Result;

/// High-level interface to NetworkManager over D-Bus.
///
/// This is the main entry point for managing network connections on Linux systems.
/// It provides a safe, async Rust API over NetworkManager's D-Bus interface.
///
/// # Creating an Instance
///
/// ```no_run
/// use nmrs::NetworkManager;
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
/// # Ok(())
/// # }
/// ```
///
/// # Capabilities
///
/// - **Device Management**: List devices, enable/disable WiFi
/// - **Network Scanning**: Discover available WiFi networks
/// - **Connection Management**: Connect to WiFi, Ethernet networks
/// - **Profile Management**: Save, retrieve, and delete connection profiles
/// - **Real-Time Monitoring**: Subscribe to network and device state changes
///
/// # Examples
///
/// ## Basic WiFi Connection
///
/// ```no_run
/// use nmrs::{NetworkManager, WifiSecurity};
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
///
/// // Scan and list networks
/// let networks = nm.list_networks().await?;
/// for net in &networks {
///     println!("{}: {}%", net.ssid, net.strength.unwrap_or(0));
/// }
///
/// // Connect to a network
/// nm.connect("MyNetwork", WifiSecurity::WpaPsk {
///     psk: "password".into()
/// }).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Device Management
///
/// ```no_run
/// use nmrs::NetworkManager;
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
///
/// // List all network devices
/// let devices = nm.list_devices().await?;
///
/// // Control WiFi
/// nm.set_wifi_enabled(false).await?;  // Disable WiFi
/// nm.set_wifi_enabled(true).await?;   // Enable WiFi
/// # Ok(())
/// # }
/// ```
///
/// ## Connection Profiles
///
/// ```no_run
/// use nmrs::NetworkManager;
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
///
/// // Check for saved connection
/// if nm.has_saved_connection("MyNetwork").await? {
///     println!("Connection profile exists");
///     
///     // Delete it
///     nm.forget("MyNetwork").await?;
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Thread Safety
///
/// `NetworkManager` is `Clone` and can be safely shared across async tasks.
/// Each clone shares the same underlying D-Bus connection.
#[derive(Debug, Clone)]
pub struct NetworkManager {
    conn: Connection,
}

impl NetworkManager {
    /// Creates a new `NetworkManager` connected to the system D-Bus.
    pub async fn new() -> Result<Self> {
        let conn = Connection::system().await?;
        Ok(Self { conn })
    }

    /// List all network devices managed by NetworkManager.
    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        list_devices(&self.conn).await
    }

    /// Lists all network devices managed by NetworkManager.
    pub async fn list_wireless_devices(&self) -> Result<Vec<Device>> {
        let devices = list_devices(&self.conn).await?;
        Ok(devices.into_iter().filter(|d| d.is_wireless()).collect())
    }

    /// List all wired (Ethernet) devices.
    pub async fn list_wired_devices(&self) -> Result<Vec<Device>> {
        let devices = list_devices(&self.conn).await?;
        Ok(devices.into_iter().filter(|d| d.is_wired()).collect())
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

    /// Connects to a wired (Ethernet) device.
    ///
    /// Finds the first available wired device and either activates an existing
    /// saved connection or creates a new one. The connection will activate
    /// when a cable is plugged in.
    ///
    /// # Errors
    ///
    /// Returns `ConnectionError::NoWiredDevice` if no wired device is found.
    pub async fn connect_wired(&self) -> Result<()> {
        connect_wired(&self.conn).await
    }

    /// Connects to a VPN using the provided credentials.
    ///
    /// Currently supports WireGuard VPN connections. The function checks for an
    /// existing saved VPN connection by name. If found, it activates the saved
    /// connection. If not found, it creates a new VPN connection with the provided
    /// credentials.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::{NetworkManager, VpnCredentials, VpnType, WireGuardPeer};
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    ///
    /// let creds = VpnCredentials {
    ///     vpn_type: VpnType::WireGuard,
    ///     name: "MyVPN".into(),
    ///     gateway: "vpn.example.com:51820".into(),
    ///     private_key: "your_private_key".into(),
    ///     address: "10.0.0.2/24".into(),
    ///     peers: vec![WireGuardPeer {
    ///         public_key: "peer_public_key".into(),
    ///         gateway: "vpn.example.com:51820".into(),
    ///         allowed_ips: vec!["0.0.0.0/0".into()],
    ///         preshared_key: None,
    ///         persistent_keepalive: Some(25),
    ///     }],
    ///     dns: Some(vec!["1.1.1.1".into()]),
    ///     mtu: None,
    ///     uuid: None,
    /// };
    ///
    /// nm.connect_vpn(creds).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - NetworkManager is not running or accessible
    /// - The credentials are invalid or incomplete
    /// - The VPN connection fails to activate
    pub async fn connect_vpn(&self, creds: VpnCredentials) -> Result<()> {
        connect_vpn(&self.conn, creds).await
    }

    /// Disconnects from an active VPN connection by name.
    ///
    /// Searches through active connections for a VPN matching the given name.
    /// If found, deactivates the connection. If not found or already disconnected,
    /// returns success.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::NetworkManager;
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    /// nm.disconnect_vpn("MyVPN").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn disconnect_vpn(&self, name: &str) -> Result<()> {
        disconnect_vpn(&self.conn, name).await
    }

    /// Lists all saved VPN connections.
    ///
    /// Returns a list of all VPN connection profiles saved in NetworkManager,
    /// including their name, type, and current state. Only VPN connections with
    /// recognized types (currently WireGuard) are returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::NetworkManager;
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    /// let vpns = nm.list_vpn_connections().await?;
    ///
    /// for vpn in vpns {
    ///     println!("{}: {:?}", vpn.name, vpn.vpn_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_vpn_connections(&self) -> Result<Vec<VpnConnection>> {
        list_vpn_connections(&self.conn).await
    }

    /// Forgets (deletes) a saved VPN connection by name.
    ///
    /// Searches through saved connections for a VPN matching the given name.
    /// If found, deletes the connection profile. If currently connected, the
    /// VPN will be disconnected first before deletion.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::NetworkManager;
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    /// nm.forget_vpn("MyVPN").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ConnectionError::NoSavedConnection` if no VPN with the given
    /// name is found.
    pub async fn forget_vpn(&self, name: &str) -> Result<()> {
        crate::core::vpn::forget_vpn(&self.conn, name).await
    }

    /// Gets detailed information about an active VPN connection.
    ///
    /// Retrieves comprehensive information about a VPN connection, including
    /// IP configuration, DNS servers, gateway, interface, and connection state.
    /// The VPN must be actively connected to retrieve this information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::NetworkManager;
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    /// let info = nm.get_vpn_info("MyVPN").await?;
    ///
    /// println!("VPN: {}", info.name);
    /// println!("Interface: {:?}", info.interface);
    /// println!("IP Address: {:?}", info.ip4_address);
    /// println!("DNS Servers: {:?}", info.dns_servers);
    /// println!("State: {:?}", info.state);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ConnectionError::NoVpnConnection` if the VPN is not found
    /// or not currently active.
    pub async fn get_vpn_info(&self, name: &str) -> Result<VpnConnectionInfo> {
        get_vpn_info(&self.conn, name).await
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

    /// Check if a network is connected
    pub async fn is_connected(&self, ssid: &str) -> Result<bool> {
        is_connected(&self.conn, ssid).await
    }

    /// Disconnects from the current network.
    ///
    /// If currently connected to a WiFi network, this will deactivate
    /// the connection and wait for the device to reach disconnected state.
    ///
    /// Returns `Ok(())` if disconnected successfully or if no active connection exists.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::NetworkManager;
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    /// nm.disconnect().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn disconnect(&self) -> Result<()> {
        disconnect(&self.conn).await
    }

    /// Returns the full `Network` object for the currently connected WiFi network.
    ///
    /// This provides detailed information about the active connection including
    /// signal strength, frequency, security type, and BSSID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::NetworkManager;
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    /// if let Some(network) = nm.current_network().await? {
    ///     println!("Connected to: {} ({}%)", network.ssid, network.strength.unwrap_or(0));
    /// } else {
    ///     println!("Not connected");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn current_network(&self) -> Result<Option<Network>> {
        current_network(&self.conn).await
    }

    /// Lists all saved connection profiles.
    ///
    /// Returns the names (IDs) of all saved connection profiles in NetworkManager,
    /// including WiFi, Ethernet, VPN, and other connection types.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::NetworkManager;
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    /// let connections = nm.list_saved_connections().await?;
    /// for name in connections {
    ///     println!("Saved connection: {}", name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_saved_connections(&self) -> Result<Vec<String>> {
        list_saved_connections(&self.conn).await
    }

    /// Finds a device by its interface name (e.g., "wlan0", "eth0").
    ///
    /// Returns the D-Bus object path of the device if found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nmrs::NetworkManager;
    ///
    /// # async fn example() -> nmrs::Result<()> {
    /// let nm = NetworkManager::new().await?;
    /// let device_path = nm.get_device_by_interface("wlan0").await?;
    /// println!("Device path: {}", device_path.as_str());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_device_by_interface(&self, name: &str) -> Result<zvariant::OwnedObjectPath> {
        get_device_by_interface(&self.conn, name).await
    }

    /// Returns the SSID of the currently connected network, if any.
    #[must_use]
    pub async fn current_ssid(&self) -> Option<String> {
        current_ssid(&self.conn).await
    }

    /// Returns the SSID and frequency of the current connection, if any.
    #[must_use]
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

    /// Forgets (deletes) a saved WiFi connection for the given SSID.
    ///
    /// If currently connected to this network, disconnects first, then deletes
    /// all saved connection profiles matching the SSID.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if at least one connection was deleted successfully.
    /// Returns `NoSavedConnection` if no matching connections were found.
    pub async fn forget(&self, ssid: &str) -> Result<()> {
        forget(&self.conn, ssid).await
    }
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
    pub async fn monitor_network_changes<F>(
        &self,
        shutdown: watch::Receiver<()>,
        callback: F,
    ) -> Result<()>
    where
        F: Fn() + 'static,
    {
        network_monitor::monitor_network_changes(&self.conn, shutdown, callback).await
    }

    /// Monitors device state changes in real-time.
    ///
    /// Subscribes to D-Bus signals for device state changes on all network
    /// devices (both wired and wireless). Invokes the callback whenever a
    /// device state changes (e.g., cable plugged in, device activated),
    /// enabling live UI updates without polling.
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
    ///         nm.monitor_device_changes(|| {
    ///             println!("Device state changed!");
    ///         }).await
    ///     }
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub async fn monitor_device_changes<F>(
        &self,
        shutdown: watch::Receiver<()>,
        callback: F,
    ) -> Result<()>
    where
        F: Fn() + 'static,
    {
        device_monitor::monitor_device_changes(&self.conn, shutdown, callback).await
    }
}
