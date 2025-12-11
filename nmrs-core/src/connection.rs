use futures_timer::Delay;
use std::collections::HashMap;
use zbus::{Connection, Result};
use zvariant::OwnedObjectPath;

use crate::connection_settings::{delete_connection, get_saved_connection_path};
use crate::constants::{device_state, device_type, retries, timeouts};
use crate::models::{ConnectionOptions, WifiSecurity};
use crate::network_info::current_ssid;
use crate::proxies::{NMAccessPointProxy, NMDeviceProxy, NMProxy, NMWirelessProxy};
use crate::state_wait::wait_for_connection_state;
use crate::utils::decode_ssid_or_empty;
use crate::wifi_builders::build_wifi_connection;

pub(crate) async fn connect(conn: &Connection, ssid: &str, creds: WifiSecurity) -> Result<()> {
    println!(
        "Connecting to '{}' | secured={} is_psk={} is_eap={}",
        ssid,
        creds.secured(),
        creds.is_psk(),
        creds.is_eap()
    );

    let nm = NMProxy::new(conn).await?;

    let saved_conn_path = get_saved_connection_path(conn, ssid).await?;

    let use_saved_connection = if let Some(conn_path) = &saved_conn_path {
        // If PSK is empty, we're trying to use saved credentials
        if creds.is_psk() {
            if let WifiSecurity::WpaPsk { psk } = &creds {
                if psk.trim().is_empty() {
                    eprintln!("Using saved connection at: {}", conn_path.as_str());
                    true
                } else {
                    eprintln!(
                        "Have saved connection but new password provided, deleting old and creating new"
                    );
                    let _ = delete_connection(conn, conn_path.clone()).await;
                    false
                }
            } else {
                false
            }
        } else {
            // For open or EAP, use saved if available
            eprintln!("Using saved connection at: {}", conn_path.as_str());
            true
        }
    } else {
        // No saved connection
        if creds.is_psk()
            && let WifiSecurity::WpaPsk { psk } = &creds
            && psk.trim().is_empty()
        {
            return Err(zbus::Error::Failure(
                "No saved connection and PSK is empty".into(),
            ));
        }

        false
    };

    let devices = nm.get_devices().await?;
    let mut wifi_device: Option<OwnedObjectPath> = None;

    for dp in devices {
        let dev = NMDeviceProxy::builder(conn)
            .path(dp.clone())?
            .build()
            .await?;
        if dev.device_type().await? == device_type::WIFI {
            wifi_device = Some(dp.clone());
            eprintln!("   Found WiFi device: {dp}");
            break;
        }
    }

    let wifi_device = wifi_device.ok_or(zbus::Error::Failure("no Wi-Fi device found".into()))?;

    let wifi = NMWirelessProxy::builder(conn)
        .path(wifi_device.clone())?
        .build()
        .await?;

    if let Some(active) = current_ssid(conn).await {
        eprintln!("Currently connected to: {active}");
        if active == ssid {
            eprintln!("Already connected to {active}, skipping connect()");
            return Ok(());
        }
    } else {
        eprintln!("Not currently connected to any network");
    }

    match wifi.request_scan(HashMap::new()).await {
        Ok(_) => eprintln!("Scan requested successfully"),
        Err(e) => eprintln!("Scan request FAILED: {e}"),
    }
    Delay::new(timeouts::scan_wait()).await;
    eprintln!("Scan wait complete");

    let mut ap_path: Option<OwnedObjectPath> = None;
    for ap in wifi.get_all_access_points().await? {
        let apx = NMAccessPointProxy::builder(conn)
            .path(ap.clone())?
            .build()
            .await?;
        let ssid_bytes = apx.ssid().await?;
        let ap_ssid = decode_ssid_or_empty(&ssid_bytes);
        eprintln!("Found AP: '{ap_ssid}'");
        if ap_ssid == ssid {
            ap_path = Some(ap.clone());
            eprintln!("Matched target SSID");
            break;
        }
    }

    if ap_path.is_none() {
        return Err(zbus::Error::Failure(format!("Network '{ssid}' not found")));
    }

    let specific_object = ap_path.unwrap();

    if use_saved_connection {
        if let Some(active) = current_ssid(conn).await {
            eprintln!("Disconnecting from {active}");
            if let Ok(conns) = nm.active_connections().await {
                for conn_path in conns {
                    let _ = nm.deactivate_connection(conn_path).await;
                }
            }
            disconnect_wifi_device(conn, &wifi_device).await?
        }

        let conn_path = saved_conn_path.unwrap();
        eprintln!("Activating saved connection: {}", conn_path.as_str());

        match nm
            .activate_connection(
                conn_path.clone(),
                wifi_device.clone(),
                specific_object.clone(),
            )
            .await
        {
            Ok(active_conn) => {
                eprintln!(
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
                    eprintln!("Connection activated but device stuck in Disconnected state");
                    eprintln!("Saved connection has invalid settings, deleting and retrying");

                    let _ = nm.deactivate_connection(active_conn).await;

                    let _ = delete_connection(conn, conn_path).await;

                    let opts = ConnectionOptions {
                        autoconnect: true,
                        autoconnect_priority: None,
                        autoconnect_retries: None,
                    };

                    let settings = build_wifi_connection(ssid, &creds, &opts);

                    eprintln!("Creating fresh connection with corrected settings");
                    match nm
                        .add_and_activate_connection(settings, wifi_device.clone(), specific_object)
                        .await
                    {
                        Ok(_) => eprintln!("Fresh connection created successfully"),
                        Err(e) => {
                            eprintln!("Fresh connection also failed: {e}");
                            return Err(e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("activate_connection() failed: {e}");
                eprintln!(
                    "Saved connection may be corrupted, deleting and retrying with fresh connection"
                );

                let _ = delete_connection(conn, conn_path).await;

                let opts = ConnectionOptions {
                    autoconnect: true,
                    autoconnect_priority: None,
                    autoconnect_retries: None,
                };

                let settings = build_wifi_connection(ssid, &creds, &opts);

                eprintln!("Creating fresh connection after saved connection failed");
                return match nm
                    .add_and_activate_connection(settings, wifi_device.clone(), specific_object)
                    .await
                {
                    Ok(_) => {
                        eprintln!("Successfully created fresh connection");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("Fresh connection also failed: {e}");
                        Err(e)
                    }
                };
            }
        }
    } else {
        let opts = ConnectionOptions {
            autoconnect: true,
            autoconnect_priority: None,
            autoconnect_retries: None,
        };

        let settings = build_wifi_connection(ssid, &creds, &opts);

        println!("Creating new connection, settings: \n{settings:#?}");

        if let Some(active) = current_ssid(conn).await {
            eprintln!("Disconnecting from {active}.");
            if let Ok(conns) = nm.active_connections().await {
                for conn_path in conns {
                    let _ = nm.deactivate_connection(conn_path).await;
                }
            }
            disconnect_wifi_device(conn, &wifi_device).await?;
        }

        match nm
            .add_and_activate_connection(settings, wifi_device.clone(), specific_object)
            .await
        {
            Ok(_) => eprintln!("add_and_activate_connection() succeeded"),
            Err(e) => {
                eprintln!("add_and_activate_connection() failed: {e}");
                return Err(e);
            }
        }
    }

    Delay::new(timeouts::disconnect_poll_interval()).await;

    let dev_proxy = NMDeviceProxy::builder(conn)
        .path(wifi_device.clone())?
        .build()
        .await?;

    let initial_state = dev_proxy.state().await?;
    eprintln!("Dev state after connect(): {initial_state:?}");

    eprintln!("Waiting for connection to complete...");
    wait_for_connection_state(&dev_proxy).await?;

    eprintln!("---Connection request for '{ssid}' submitted successfully---");

    Ok(())
}
pub(crate) async fn forget(conn: &Connection, ssid: &str) -> zbus::Result<()> {
    use std::collections::HashMap;
    use zvariant::{OwnedObjectPath, Value};

    eprintln!("Starting forget operation for: {ssid}");

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
                eprintln!("Disconnecting from active network: {ssid}");
                let dev_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
                    .destination("org.freedesktop.NetworkManager")?
                    .path(dev_path.clone())?
                    .interface("org.freedesktop.NetworkManager.Device")?
                    .build()
                    .await?;

                match dev_proxy.call_method("Disconnect", &()).await {
                    Ok(_) => eprintln!("Disconnect call succeeded"),
                    Err(e) => eprintln!("Disconnect call failed: {e}"),
                }

                eprintln!("About to enter wait loop...");
                for i in 0..retries::FORGET_MAX_RETRIES {
                    Delay::new(timeouts::forget_poll_interval()).await;
                    match dev.state().await {
                        Ok(current_state) => {
                            eprintln!("Wait loop {i}: device state = {current_state}");
                            if current_state == device_state::DISCONNECTED
                                || current_state == device_state::UNAVAILABLE
                            {
                                eprintln!("Device reached disconnected state");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get device state in wait loop {i}: {e}");
                            break;
                        }
                    }
                }
                eprintln!("Wait loop completed");
            }
        }
    }

    eprintln!("Starting connection deletion phase...");

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
                eprintln!("Found connection by ID: {id}");
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
                    eprintln!("Found connection by SSID match");
                }
            }

            if let Some(wsec) = settings_map.get("802-11-wireless-security") {
                let missing_psk = !wsec.contains_key("psk");
                let empty_psk = matches!(wsec.get("psk"), Some(Value::Str(s)) if s.is_empty());

                if (missing_psk || empty_psk) && should_delete {
                    eprintln!("Connection has missing/empty PSK, will delete");
                }
            }

            if should_delete {
                match cproxy.call_method("Delete", &()).await {
                    Ok(_) => {
                        deleted_count += 1;
                        eprintln!("Deleted connection: {}", cpath.as_str());
                    }
                    Err(e) => {
                        eprintln!("Failed to delete connection {}: {}", cpath.as_str(), e);
                    }
                }
            }
        }
    }

    if deleted_count > 0 {
        eprintln!("Successfully deleted {deleted_count} connection(s) for '{ssid}'");
        Ok(())
    } else {
        eprintln!("No saved connections found for '{ssid}'");
        Err(zbus::Error::Failure(format!(
            "No saved connection for {ssid}"
        )))
    }
}

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
            Err(e) => return Err(e),
        }
    }

    Delay::new(timeouts::disconnect_final_delay()).await;

    match dev.state().await {
        Ok(s) if s == device_state::DISCONNECTED || s == device_state::UNAVAILABLE => Ok(()),
        Ok(s) => Err(zbus::Error::Failure(format!("device stuck in state {s}"))),
        Err(e) => Err(e),
    }
}
