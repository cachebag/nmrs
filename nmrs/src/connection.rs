use futures_timer::Delay;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use zbus::Connection;
use zvariant::OwnedObjectPath;

use crate::Result;
use crate::connection_settings::{delete_connection, get_saved_connection_path};
use crate::constants::{device_state, device_type, retries, timeouts};
use crate::models::{ConnectionError, ConnectionOptions, WifiSecurity};
use crate::network_info::current_ssid;
use crate::proxies::{NMAccessPointProxy, NMDeviceProxy, NMProxy, NMWirelessProxy};
use crate::state_wait::wait_for_connection_state;
use crate::utils::decode_ssid_or_empty;
use crate::wifi_builders::build_wifi_connection;

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

    if let Some(active) = current_ssid(conn).await {
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
    let dev_proxy = NMDeviceProxy::builder(conn)
        .path(wifi_device.clone())?
        .build()
        .await?;
    debug!("Waiting for connection to complete...");
    wait_for_connection_state(&dev_proxy).await?;

    info!("Connection request for '{ssid}' submitted successfully");

    Ok(())
}

/// Forgets (deletes) all saved connections for a network.
///
/// If currently connected to this network, disconnects first, then deletes
/// all saved connection profiles matching the SSID. Matches are found by
/// both the connection ID and the wireless SSID bytes.
///
/// Returns `NoSavedConnection` if no matching connections were found.
pub(crate) async fn forget(conn: &Connection, ssid: &str) -> Result<()> {
    use std::collections::HashMap;
    use zvariant::{OwnedObjectPath, Value};

    debug!("Starting forget operation for: {ssid}");

    let nm = NMProxy::new(conn).await?;

    let devices = nm.get_devices().await?;
    for dev_path in &devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(dev_path.clone())?
            .build()
            .await?;
        if dev.device_type().await? != device_type::WIFI {
            continue;
        }

        let wifi = NMWirelessProxy::builder(conn)
            .path(dev_path.clone())?
            .build()
            .await?;
        if let Ok(ap_path) = wifi.active_access_point().await
            && ap_path.as_str() != "/"
        {
            let ap = NMAccessPointProxy::builder(conn)
                .path(ap_path.clone())?
                .build()
                .await?;
            if let Ok(bytes) = ap.ssid().await
                && decode_ssid_or_empty(&bytes) == ssid
            {
                debug!("Disconnecting from active network: {ssid}");
                let dev_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
                    .destination("org.freedesktop.NetworkManager")?
                    .path(dev_path.clone())?
                    .interface("org.freedesktop.NetworkManager.Device")?
                    .build()
                    .await?;

                match dev_proxy.call_method("Disconnect", &()).await {
                    Ok(_) => debug!("Disconnect call succeeded"),
                    Err(e) => warn!("Disconnect call failed: {e}"),
                }

                debug!("About to enter wait loop...");
                for i in 0..retries::FORGET_MAX_RETRIES {
                    Delay::new(timeouts::forget_poll_interval()).await;
                    match dev.state().await {
                        Ok(current_state) => {
                            debug!("Wait loop {i}: device state = {current_state}");
                            if current_state == device_state::DISCONNECTED
                                || current_state == device_state::UNAVAILABLE
                            {
                                debug!("Device reached disconnected state");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get device state in wait loop {i}: {e}");
                            break;
                        }
                    }
                }
                debug!("Wait loop completed");
            }
        }
    }

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

            if let Some(conn_sec) = settings_map.get("connection")
                && let Some(Value::Str(id)) = conn_sec.get("id")
                && id.as_str() == ssid
            {
                should_delete = true;
                debug!("Found connection by ID: {id}");
            }

            if let Some(wifi_sec) = settings_map.get("802-11-wireless")
                && let Some(Value::Array(arr)) = wifi_sec.get("ssid")
            {
                let mut raw = Vec::new();
                for v in arr.iter() {
                    if let Ok(b) = u8::try_from(v.clone()) {
                        raw.push(b);
                    }
                }
                if decode_ssid_or_empty(&raw) == ssid {
                    should_delete = true;
                    debug!("Found connection by SSID match");
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
        info!("Successfully deleted {deleted_count} connection(s) for '{ssid}'");
        Ok(())
    } else {
        debug!("No saved connections found for '{ssid}'");
        Err(ConnectionError::NoSavedConnection)
    }
}

/// Disconnects a Wi-Fi device and waits for it to reach disconnected state.
///
/// Calls the Disconnect method on the device and polls until the device
/// state becomes Disconnected or Unavailable, or the retry limit is reached.
pub(crate) async fn disconnect_wifi_device(
    conn: &Connection,
    dev_path: &OwnedObjectPath,
) -> Result<()> {
    let dev = NMDeviceProxy::builder(conn)
        .path(dev_path.clone())?
        .build()
        .await?;

    let raw: zbus::proxy::Proxy = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.NetworkManager")?
        .path(dev_path.clone())?
        .interface("org.freedesktop.NetworkManager.Device")?
        .build()
        .await?;

    let _ = raw.call_method("Disconnect", &()).await;

    for _ in 0..retries::DISCONNECT_MAX_RETRIES {
        Delay::new(timeouts::disconnect_poll_interval()).await;
        match dev.state().await {
            Ok(s) if s == device_state::DISCONNECTED || s == device_state::UNAVAILABLE => {
                break;
            }
            Ok(_) => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Delay::new(timeouts::disconnect_final_delay()).await;

    match dev.state().await {
        Ok(s) if s == device_state::DISCONNECTED || s == device_state::UNAVAILABLE => Ok(()),
        Ok(s) => Err(ConnectionError::Stuck(format!("{s}"))),
        Err(e) => Err(e.into()),
    }
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
    let access_points = wifi.get_all_access_points().await?;

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
    if let Some(active) = current_ssid(conn).await {
        debug!("Disconnecting from {active}");

        if let Ok(conns) = nm.active_connections().await {
            for conn_path in conns {
                let _ = nm.deactivate_connection(conn_path).await;
            }
        }

        disconnect_wifi_device(conn, wifi_device).await?;
    }

    Ok(())
}

/// Attempts to connect using a saved connection profile.
///
/// Activates the saved connection. If activation succeeds but the device
/// ends up in a disconnected state (indicating invalid saved settings),
/// deletes the saved connection and creates a fresh one with the provided
/// credentials.
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

            Delay::new(timeouts::disconnect_final_delay()).await;

            let dev_check = NMDeviceProxy::builder(conn)
                .path(wifi_device.clone())?
                .build()
                .await?;

            let check_state = dev_check.state().await?;

            if check_state == device_state::DISCONNECTED {
                warn!("Connection activated but device stuck in Disconnected state");
                warn!("Saved connection has invalid settings, deleting and retrying");

                let _ = nm.deactivate_connection(active_conn).await;

                let _ = delete_connection(conn, saved.clone()).await;

                let opts = ConnectionOptions {
                    autoconnect: true,
                    autoconnect_priority: None,
                    autoconnect_retries: None,
                };

                let settings = build_wifi_connection(ap.as_str(), creds, &opts);

                debug!("Creating fresh connection with corrected settings");
                nm.add_and_activate_connection(settings, wifi_device.clone(), ap.clone())
                    .await
                    .map_err(|e| {
                        error!("Fresh connection also failed: {e}");
                        e
                    })?;
            }
        }

        Err(e) => {
            warn!("activate_connection() failed: {e}");
            warn!(
                "Saved connection may be corrupted, deleting and retrying with fresh connection"
            );

            let _ = delete_connection(conn, saved.clone()).await;

            let opts = ConnectionOptions {
                autoconnect: true,
                autoconnect_priority: None,
                autoconnect_retries: None,
            };

            let settings = build_wifi_connection(ap.as_str(), creds, &opts);

            nm.add_and_activate_connection(settings, wifi_device.clone(), ap.clone())
                .await
                .map_err(|e| {
                    error!("Fresh connection also failed: {e}");
                    e
                })?;
        }
    }

    Ok(())
}

/// Creates a new connection profile and activates it.
///
/// Builds connection settings from the provided credentials, ensures the
/// device is disconnected, then calls AddAndActivateConnection to create
/// and activate the connection in one step.
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

    match nm
        .add_and_activate_connection(settings, wifi_device.clone(), ap.clone())
        .await
    {
        Ok(_) => debug!("add_and_activate_connection() succeeded"),
        Err(e) => {
            error!("add_and_activate_connection() failed: {e}");
            return Err(e.into());
        }
    }

    Delay::new(timeouts::disconnect_poll_interval()).await;

    let dev_proxy = NMDeviceProxy::builder(conn)
        .path(wifi_device.clone())?
        .build()
        .await?;

    let initial_state = dev_proxy.state().await?;
    debug!("Dev state after build_and_activate_new(): {initial_state:?}");
    debug!("Waiting for connection to complete...");
    wait_for_connection_state(&dev_proxy).await?;

    info!("Connection request for '{ssid}' submitted successfully");

    Ok(())
}

/// Triggers a Wi-Fi scan and finds the target access point.
///
/// Requests a scan, waits for it to complete, then searches for an
/// access point matching the target SSID.
async fn scan_and_resolve_ap(
    conn: &Connection,
    wifi: &NMWirelessProxy<'_>,
    ssid: &str,
) -> Result<OwnedObjectPath> {
    match wifi.request_scan(HashMap::new()).await {
        Ok(_) => debug!("Scan requested successfully"),
        Err(e) => warn!("Scan request failed: {e}"),
    }

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
    if let Some(path) = saved {
        if creds.is_psk()
            && let WifiSecurity::WpaPsk { psk } = creds
        {
            if psk.trim().is_empty() {
                return Ok(SavedDecision::UseSaved(path));
            }
            return Ok(SavedDecision::RebuildFresh);
        }
        return Ok(SavedDecision::UseSaved(path));
    }

    if creds.is_psk()
        && let WifiSecurity::WpaPsk { psk } = creds
        && psk.trim().is_empty()
    {
        return Err(ConnectionError::NoSavedConnection);
    }

    Ok(SavedDecision::RebuildFresh)
}
