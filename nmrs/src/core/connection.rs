use futures_timer::Delay;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use zbus::Connection;
use zvariant::OwnedObjectPath;

use crate::api::builders::wifi::{build_ethernet_connection, build_wifi_connection};
use crate::api::models::{ConnectionError, ConnectionOptions, WifiSecurity};
use crate::core::connection_settings::{delete_connection, get_saved_connection_path};
use crate::core::state_wait::{wait_for_connection_activation, wait_for_device_disconnect};
use crate::dbus::{NMAccessPointProxy, NMDeviceProxy, NMProxy, NMWiredProxy, NMWirelessProxy};
use crate::monitoring::transport::ActiveTransport;
use crate::monitoring::wifi::Wifi;
use crate::types::constants::{device_state, device_type, timeouts};
use crate::util::utils::decode_ssid_or_empty;
use crate::Result;

/// Decision on whether to reuse a saved connection or create a fresh one.
enum SavedDecision {
    /// Reuse the saved connection at this path.
    UseSaved(OwnedObjectPath),
    /// Delete any saved connection and create a new one with fresh credentials.
    RebuildFresh,
}

/// Connects to a Wi-Fi network.
///
/// This is the main entry point for establishing a Wi-Fi connection. The flow:
/// 1. Check for an existing saved connection for this SSID
/// 2. Decide whether to reuse it or create fresh (based on credentials)
/// 3. Find the Wi-Fi device and target access point
/// 4. Either activate the saved connection or create and activate a new one
/// 5. Wait for the connection to reach the activated state
///
/// If a saved connection exists but fails, it will be deleted and a fresh
/// connection will be attempted with the provided credentials.
pub(crate) async fn connect(conn: &Connection, ssid: &str, creds: WifiSecurity) -> Result<()> {
    debug!(
        "Connecting to '{}' | secured={} is_psk={} is_eap={}",
        ssid,
        creds.secured(),
        creds.is_psk(),
        creds.is_eap()
    );

    let nm = NMProxy::new(conn).await?;

    let saved_raw = get_saved_connection_path(conn, ssid).await?;
    let decision = decide_saved_connection(saved_raw, &creds)?;

    let wifi_device = find_wifi_device(conn, &nm).await?;
    debug!("Found WiFi device: {}", wifi_device.as_str());

    let wifi = NMWirelessProxy::builder(conn)
        .path(wifi_device.clone())?
        .build()
        .await?;

    if let Some(active) = Wifi::current(conn).await {
        debug!("Currently connected to: {active}");
        if active == ssid {
            debug!("Already connected to {active}, skipping connect()");
            return Ok(());
        }
    } else {
        debug!("Not currently connected to any network");
    }

    let specific_object = scan_and_resolve_ap(conn, &wifi, ssid).await?;

    match decision {
        SavedDecision::UseSaved(saved) => {
            ensure_disconnected(conn, &nm, &wifi_device).await?;
            connect_via_saved(conn, &nm, &wifi_device, &specific_object, &creds, saved).await?;
        }
        SavedDecision::RebuildFresh => {
            build_and_activate_new(conn, &nm, &wifi_device, &specific_object, ssid, creds).await?;
        }
    }

    // Connection activation is now handled within connect_via_saved() and
    // build_and_activate_new() using signal-based monitoring
    info!("Successfully connected to '{ssid}'");

    Ok(())
}

/// Connects to a wired (Ethernet) device.
///
/// This is the main entry point for establishing a wired connection. The flow:
/// 1. Find a wired device
/// 2. Check for an existing saved connection
/// 3. Either activate the saved connection or create and activate a new one
/// 4. Wait for the connection to reach the activated state
///
/// Ethernet connections are typically simpler than Wi-Fi - no scanning or
/// access points needed. The connection will activate when a cable is plugged in.
pub(crate) async fn connect_wired(conn: &Connection) -> Result<()> {
    debug!("Connecting to wired device");

    let nm = NMProxy::new(conn).await?;

    let wired_device = find_wired_device(conn, &nm).await?;
    debug!("Found wired device: {}", wired_device.as_str());

    // Check if already connected
    let dev = NMDeviceProxy::builder(conn)
        .path(wired_device.clone())?
        .build()
        .await?;
    let current_state = dev.state().await?;
    if current_state == device_state::ACTIVATED {
        debug!("Wired device already activated, skipping connect()");
        return Ok(());
    }

    // Check for saved connection (by interface name)
    let interface = dev.interface().await?;
    let saved = get_saved_connection_path(conn, &interface).await?;

    // For Ethernet, we use "/" as the specific_object (no access point needed)
    let specific_object = OwnedObjectPath::try_from("/").unwrap();

    match saved {
        Some(saved_path) => {
            debug!("Activating saved wired connection: {}", saved_path.as_str());
            let active_conn = nm
                .activate_connection(saved_path, wired_device.clone(), specific_object)
                .await?;
            wait_for_connection_activation(conn, &active_conn).await?;
        }
        None => {
            debug!("No saved connection found, creating new wired connection");
            let opts = ConnectionOptions {
                autoconnect: true,
                autoconnect_priority: None,
                autoconnect_retries: None,
            };

            let settings = build_ethernet_connection(&interface, &opts);
            let (_, active_conn) = nm
                .add_and_activate_connection(settings, wired_device.clone(), specific_object)
                .await?;
            wait_for_connection_activation(conn, &active_conn).await?;
        }
    }

    if let Ok(wired) = NMWiredProxy::builder(conn)
        .path(wired_device.clone())?
        .build()
        .await
    {
        if let Ok(speed) = wired.speed().await {
            info!("Connected to wired device at {speed} Mb/s");
        }
    }

    info!("Successfully connected to wired device");
    Ok(())
}

/// Generic function to forget (delete) connections by name and optionally by device type.
///
/// This handles disconnection if currently active, then deletes the connection profile(s).
/// Can be used for WiFi, Bluetooth, or any NetworkManager connection type.
///
/// # Arguments
///
/// * `conn` - D-Bus connection
/// * `name` - Connection name/identifier to forget
/// * `device_filter` - Optional device type filter (e.g., `Some(device_type::BLUETOOTH)`)
///
/// # Returns
///
/// Returns `Ok(())` if at least one connection was deleted successfully.
/// Returns `NoSavedConnection` if no matching connections were found.
pub(crate) async fn forget_by_name_and_type(
    conn: &Connection,
    name: &str,
    device_filter: Option<u32>,
) -> Result<()> {
    use std::collections::HashMap;
    use zvariant::{OwnedObjectPath, Value};

    debug!(
        "Starting forget operation for: {name} (device filter: {:?})",
        device_filter
    );

    let nm = NMProxy::new(conn).await?;

    // Disconnect if currently active
    let devices = nm.get_devices().await?;
    for dev_path in &devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(dev_path.clone())?
            .build()
            .await?;

        let dev_type = dev.device_type().await?;

        // Skip if device type doesn't match our filter
        if let Some(filter) = device_filter {
            if dev_type != filter {
                continue;
            }
        }

        // Handle WiFi-specific disconnect logic
        if dev_type == device_type::WIFI {
            let wifi = NMWirelessProxy::builder(conn)
                .path(dev_path.clone())?
                .build()
                .await?;
            if let Ok(ap_path) = wifi.active_access_point().await {
                if ap_path.as_str() != "/" {
                    let ap = NMAccessPointProxy::builder(conn)
                        .path(ap_path.clone())?
                        .build()
                        .await?;
                    if let Ok(bytes) = ap.ssid().await {
                        if decode_ssid_or_empty(&bytes) == name {
                            debug!("Disconnecting from active WiFi network: {name}");
                            if let Err(e) = disconnect_wifi_and_wait(conn, dev_path).await {
                                warn!("Disconnect wait failed: {e}");
                                let final_state = dev.state().await?;
                                if final_state != device_state::DISCONNECTED
                                    && final_state != device_state::UNAVAILABLE
                                {
                                    error!(
                                        "Device still connected (state: {final_state}), cannot safely delete"
                                    );
                                    return Err(ConnectionError::Stuck(format!(
                                        "disconnect failed, device in state {final_state}"
                                    )));
                                }
                                debug!("Device confirmed disconnected, proceeding with deletion");
                            }
                            debug!("WiFi disconnect phase completed");
                        }
                    }
                }
            }
        }
        // Handle Bluetooth-specific disconnect logic
        else if dev_type == device_type::BLUETOOTH {
            // Check if this Bluetooth device is currently active
            let state = dev.state().await?;
            if state != device_state::DISCONNECTED && state != device_state::UNAVAILABLE {
                debug!("Disconnecting from active Bluetooth device: {name}");
                if let Err(e) =
                    crate::core::bluetooth::disconnect_bluetooth_and_wait(conn, dev_path).await
                {
                    warn!("Bluetooth disconnect failed: {e}");
                    let final_state = dev.state().await?;
                    if final_state != device_state::DISCONNECTED
                        && final_state != device_state::UNAVAILABLE
                    {
                        error!(
                            "Bluetooth device still connected (state: {final_state}), cannot safely delete"
                        );
                        return Err(ConnectionError::Stuck(format!(
                            "disconnect failed, device in state {final_state}"
                        )));
                    }
                }
                debug!("Bluetooth disconnect phase completed");
            }
        }
    }

    // Delete connection profiles (generic, works for all types)
    debug!("Starting connection deletion phase...");

    let settings: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.NetworkManager")?
        .path("/org/freedesktop/NetworkManager/Settings")?
        .interface("org.freedesktop.NetworkManager.Settings")?
        .build()
        .await?;

    let list_reply = settings.call_method("ListConnections", &()).await?;
    let conns: Vec<OwnedObjectPath> = list_reply.body().deserialize()?;

    let mut deleted_count = 0;

    for cpath in conns {
        let cproxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(cpath.clone())?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await?;

        if let Ok(msg) = cproxy.call_method("GetSettings", &()).await {
            let body = msg.body();
            let settings_map: HashMap<String, HashMap<String, Value>> = body.deserialize()?;

            let mut should_delete = false;

            // Match by connection ID (works for all connection types)
            if let Some(conn_sec) = settings_map.get("connection") {
                if let Some(Value::Str(id)) = conn_sec.get("id") {
                    if id.as_str() == name {
                        should_delete = true;
                        debug!("Found connection by ID: {id}");
                    }
                }
            }

            // Additional WiFi-specific matching by SSID
            if let Some(wifi_sec) = settings_map.get("802-11-wireless") {
                if let Some(Value::Array(arr)) = wifi_sec.get("ssid") {
                    let mut raw = Vec::new();
                    for v in arr.iter() {
                        if let Ok(b) = u8::try_from(v.clone()) {
                            raw.push(b);
                        }
                    }
                    if decode_ssid_or_empty(&raw) == name {
                        should_delete = true;
                        debug!("Found WiFi connection by SSID match");
                    }
                }
            }

            // Matching by bdaddr for Bluetooth connections
            if let Some(bt_sec) = settings_map.get("bluetooth") {
                if let Some(Value::Str(bdaddr)) = bt_sec.get("bdaddr") {
                    if bdaddr.as_str() == name {
                        should_delete = true;
                        debug!("Found Bluetooth connection by bdaddr match");
                    }
                }
            }

            if let Some(wsec) = settings_map.get("802-11-wireless-security") {
                let missing_psk = !wsec.contains_key("psk");
                let empty_psk = matches!(wsec.get("psk"), Some(Value::Str(s)) if s.is_empty());

                if (missing_psk || empty_psk) && should_delete {
                    debug!("Connection has missing/empty PSK, will delete");
                }
            }

            if should_delete {
                match cproxy.call_method("Delete", &()).await {
                    Ok(_) => {
                        deleted_count += 1;
                        debug!("Deleted connection: {}", cpath.as_str());
                    }
                    Err(e) => {
                        warn!("Failed to delete connection {}: {}", cpath.as_str(), e);
                    }
                }
            }
        }
    }

    if deleted_count > 0 {
        info!("Successfully deleted {deleted_count} connection(s) for '{name}'");
        Ok(())
    } else {
        debug!("No saved connections found for '{name}'");

        // For Bluetooth, it's normal to have no NetworkManager connection profile if the device is only paired in BlueZ.
        if device_filter == Some(device_type::BLUETOOTH) {
            debug!("Bluetooth device '{name}' has no NetworkManager connection profile (device may only be paired in BlueZ)");
            Ok(())
        } else {
            Err(ConnectionError::NoSavedConnection)
        }
    }
}

/// Disconnects a Wi-Fi device and waits for it to reach disconnected state.
///
/// Calls the Disconnect method on the device and waits for the `StateChanged`
/// signal to indicate the device has reached Disconnected or Unavailable state.
/// This is more efficient than polling and responds immediately when the
/// device disconnects.
pub(crate) async fn disconnect_wifi_and_wait(
    conn: &Connection,
    dev_path: &OwnedObjectPath,
) -> Result<()> {
    let dev = NMDeviceProxy::builder(conn)
        .path(dev_path.clone())?
        .build()
        .await?;

    // Check if already disconnected
    let current_state = dev.state().await?;
    if current_state == device_state::DISCONNECTED || current_state == device_state::UNAVAILABLE {
        debug!("Device already disconnected");
        return Ok(());
    }

    let raw: zbus::proxy::Proxy = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.NetworkManager")?
        .path(dev_path.clone())?
        .interface("org.freedesktop.NetworkManager.Device")?
        .build()
        .await?;

    debug!("Sending disconnect request");
    let _ = raw.call_method("Disconnect", &()).await;

    // Wait for disconnect using signal-based monitoring
    wait_for_device_disconnect(&dev).await?;

    // Brief stabilization delay
    Delay::new(timeouts::stabilization_delay()).await;

    Ok(())
}

/// Find the first wired (Ethernet) device on the system.
///
/// Iterates through all NetworkManager devices and returns the first one
/// with device type `ETHERNET`. Returns `NoWiredDevice` if none found.
pub(crate) async fn find_wired_device(
    conn: &Connection,
    nm: &NMProxy<'_>,
) -> Result<OwnedObjectPath> {
    let devices = nm.get_devices().await?;

    for dp in devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;
        if dev.device_type().await? == device_type::ETHERNET {
            return Ok(dp);
        }
    }
    Err(ConnectionError::NoWiredDevice)
}

/// Finds the first Wi-Fi device on the system.
///
/// Iterates through all NetworkManager devices and returns the first one
/// with device type `WIFI`. Returns `NoWifiDevice` if none found.
async fn find_wifi_device(conn: &Connection, nm: &NMProxy<'_>) -> Result<OwnedObjectPath> {
    let devices = nm.get_devices().await?;

    for dp in devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;
        if dev.device_type().await? == device_type::WIFI {
            return Ok(dp);
        }
    }
    Err(ConnectionError::NoWifiDevice)
}

/// Finds an access point by SSID.
///
/// Searches through all visible access points on the wireless device
/// and returns the path of the first one matching the target SSID.
/// Returns `NotFound` if no matching access point is visible.
async fn find_ap(
    conn: &Connection,
    wifi: &NMWirelessProxy<'_>,
    target_ssid: &str,
) -> Result<OwnedObjectPath> {
    let access_points = wifi.access_points().await?;

    for ap_path in access_points {
        let ap = NMAccessPointProxy::builder(conn)
            .path(ap_path.clone())?
            .build()
            .await?;

        let ssid_bytes = ap.ssid().await?;
        let ssid = decode_ssid_or_empty(&ssid_bytes);

        if ssid == target_ssid {
            return Ok(ap_path);
        }
    }

    Err(ConnectionError::NotFound)
}

/// Ensures the Wi-Fi device is disconnected before attempting a new connection.
///
/// If currently connected to any network, deactivates all active connections
/// and waits for the device to reach disconnected state.
async fn ensure_disconnected(
    conn: &Connection,
    nm: &NMProxy<'_>,
    wifi_device: &OwnedObjectPath,
) -> Result<()> {
    if let Some(active) = Wifi::current(conn).await {
        debug!("Disconnecting from {active}");

        if let Ok(conns) = nm.active_connections().await {
            for conn_path in conns {
                let _ = nm.deactivate_connection(conn_path).await;
            }
        }

        disconnect_wifi_and_wait(conn, wifi_device).await?;
    }

    Ok(())
}

/// Attempts to connect using a saved connection profile.
///
/// Activates the saved connection and monitors the activation state using
/// D-Bus signals. If activation fails (device disconnects or enters failed
/// state), deletes the saved connection and creates a fresh one with the
/// provided credentials.
///
/// This handles cases where saved passwords are outdated or corrupted.
async fn connect_via_saved(
    conn: &Connection,
    nm: &NMProxy<'_>,
    wifi_device: &OwnedObjectPath,
    ap: &OwnedObjectPath,
    creds: &WifiSecurity,
    saved: OwnedObjectPath,
) -> Result<()> {
    debug!("Activating saved connection: {}", saved.as_str());

    match nm
        .activate_connection(saved.clone(), wifi_device.clone(), ap.clone())
        .await
    {
        Ok(active_conn) => {
            debug!(
                "activate_connection() succeeded, active connection: {}",
                active_conn.as_str()
            );

            // Wait for connection activation using signal-based monitoring
            match wait_for_connection_activation(conn, &active_conn).await {
                Ok(()) => {
                    debug!("Saved connection activated successfully");
                }
                Err(e) => {
                    warn!("Saved connection activation failed: {e}");
                    warn!("Deleting saved connection and retrying with fresh credentials");

                    let _ = nm.deactivate_connection(active_conn).await;
                    let _ = delete_connection(conn, saved.clone()).await;

                    let opts = ConnectionOptions {
                        autoconnect: true,
                        autoconnect_priority: None,
                        autoconnect_retries: None,
                    };

                    let settings = build_wifi_connection(ap.as_str(), creds, &opts);

                    debug!("Creating fresh connection with corrected settings");
                    let (_, new_active_conn) = nm
                        .add_and_activate_connection(settings, wifi_device.clone(), ap.clone())
                        .await
                        .map_err(|e| {
                            error!("Fresh connection also failed: {e}");
                            e
                        })?;

                    // Wait for the fresh connection to activate
                    wait_for_connection_activation(conn, &new_active_conn).await?;
                }
            }
        }

        Err(e) => {
            warn!("activate_connection() failed: {e}");
            warn!("Saved connection may be corrupted, deleting and retrying with fresh connection");

            let _ = delete_connection(conn, saved.clone()).await;

            let opts = ConnectionOptions {
                autoconnect: true,
                autoconnect_priority: None,
                autoconnect_retries: None,
            };

            let settings = build_wifi_connection(ap.as_str(), creds, &opts);

            let (_, active_conn) = nm
                .add_and_activate_connection(settings, wifi_device.clone(), ap.clone())
                .await
                .map_err(|e| {
                    error!("Fresh connection also failed: {e}");
                    e
                })?;

            // Wait for the fresh connection to activate
            wait_for_connection_activation(conn, &active_conn).await?;
        }
    }

    Ok(())
}

/// Creates a new connection profile and activates it.
///
/// Builds connection settings from the provided credentials, ensures the
/// device is disconnected, then calls AddAndActivateConnection to create
/// and activate the connection in one step. Monitors activation using
/// D-Bus signals for immediate feedback on success or failure.
async fn build_and_activate_new(
    conn: &Connection,
    nm: &NMProxy<'_>,
    wifi_device: &OwnedObjectPath,
    ap: &OwnedObjectPath,
    ssid: &str,
    creds: WifiSecurity,
) -> Result<()> {
    let opts = ConnectionOptions {
        autoconnect: true,
        autoconnect_retries: None,
        autoconnect_priority: None,
    };

    let settings = build_wifi_connection(ssid, &creds, &opts);

    debug!("Creating new connection, settings: \n{settings:#?}");

    ensure_disconnected(conn, nm, wifi_device).await?;

    let (_, active_conn) = match nm
        .add_and_activate_connection(settings, wifi_device.clone(), ap.clone())
        .await
    {
        Ok(paths) => {
            debug!(
                "add_and_activate_connection() succeeded, active connection: {}",
                paths.1.as_str()
            );
            paths
        }
        Err(e) => {
            error!("add_and_activate_connection() failed: {e}");
            return Err(e.into());
        }
    };

    debug!("Waiting for connection activation using signal monitoring...");

    // Wait for connection activation using the ActiveConnection signals
    wait_for_connection_activation(conn, &active_conn).await?;

    info!("Connection to '{ssid}' activated successfully");

    Ok(())
}

/// Triggers a Wi-Fi scan and finds the target access point.
///
/// Requests a scan, waits briefly for results, then searches for an
/// access point matching the target SSID. The wait time is shorter than
/// polling-based approaches since we just need the scan to populate
/// initial results.
async fn scan_and_resolve_ap(
    conn: &Connection,
    wifi: &NMWirelessProxy<'_>,
    ssid: &str,
) -> Result<OwnedObjectPath> {
    match wifi.request_scan(HashMap::new()).await {
        Ok(_) => debug!("Scan requested successfully"),
        Err(e) => warn!("Scan request failed: {e}"),
    }

    // Brief wait for scan results to populate
    Delay::new(timeouts::scan_wait()).await;
    debug!("Scan wait complete");

    let ap = find_ap(conn, wifi, ssid).await?;
    debug!("Matched target SSID '{ssid}'");
    Ok(ap)
}

/// Decides whether to use a saved connection or create a fresh one.
///
/// Decision logic:
/// - If a saved connection exists and credentials are empty PSK, use saved
///   (user wants to connect with stored password)
/// - If a saved connection exists but new PSK credentials provided, rebuild
///   (user is updating the password)
/// - If no saved connection and PSK is empty, error (can't connect without password)
/// - Otherwise, create a fresh connection
fn decide_saved_connection(
    saved: Option<OwnedObjectPath>,
    creds: &WifiSecurity,
) -> Result<SavedDecision> {
    match saved {
        Some(_) if matches!(creds, WifiSecurity::WpaPsk { psk } if !psk.trim().is_empty()) => {
            Ok(SavedDecision::RebuildFresh)
        }

        Some(path) => Ok(SavedDecision::UseSaved(path)),

        None if matches!(creds, WifiSecurity::WpaPsk { psk } if psk.trim().is_empty()) => {
            Err(ConnectionError::NoSavedConnection)
        }

        None => Ok(SavedDecision::RebuildFresh),
    }
}
