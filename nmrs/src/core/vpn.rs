//! Core VPN connection management logic.
//!
//! This module contains internal implementation for managing VPN connections
//! through NetworkManager, including connecting, disconnecting, listing, and
//! deleting VPN profiles.
//!
//! Currently supports:
//! - WireGuard connections (NetworkManager connection.type == "wireguard")
//!
//! These functions are not part of the public API and should be accessed
//! through the [`NetworkManager`][crate::NetworkManager] interface.

use log::{debug, info, warn};
use std::collections::HashMap;
use zbus::Connection;
use zvariant::OwnedObjectPath;

use crate::api::models::{
    ConnectionOptions, DeviceState, VpnConnection, VpnConnectionInfo, VpnCredentials, VpnType,
};
use crate::builders::build_wireguard_connection;
use crate::core::state_wait::wait_for_connection_activation;
use crate::dbus::{NMActiveConnectionProxy, NMProxy};
use crate::Result;

/// Connects to a WireGuard connection.
///
/// This function checks for an existing saved connection by name.
/// If found, it activates the saved connection. If not found, it creates
/// a new WireGuard connection using the provided credentials.
/// The function waits for the connection to reach the activated state
/// before returning.
///
/// WireGuard activations do not require binding to an underlying device.
/// Use "/" so NetworkManager auto-selects.
pub(crate) async fn connect_vpn(conn: &Connection, creds: VpnCredentials) -> Result<()> {
    debug!("Connecting to VPN: {}", creds.name);

    let nm = NMProxy::new(conn).await?;

    // Check saved connections
    let saved =
        crate::core::connection_settings::get_saved_connection_path(conn, &creds.name).await?;

    // For WireGuard activation, always use "/" as device path - NetworkManager will auto-select
    let vpn_device_path = OwnedObjectPath::try_from("/").unwrap();
    let specific_object = OwnedObjectPath::try_from("/").unwrap();

    let active_conn = if let Some(saved_path) = saved {
        debug!("Activating existent VPN connection");
        nm.activate_connection(saved_path, vpn_device_path.clone(), specific_object.clone())
            .await?
    } else {
        debug!("Creating new VPN connection");
        let opts = ConnectionOptions {
            autoconnect: false,
            autoconnect_priority: None,
            autoconnect_retries: None,
        };

        let settings = build_wireguard_connection(&creds, &opts)?;

        // Use Settings API to add connection first, then activate separately
        // This avoids NetworkManager's device validation when using add_and_activate_connection
        let settings_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path("/org/freedesktop/NetworkManager/Settings")?
            .interface("org.freedesktop.NetworkManager.Settings")?
            .build()
            .await?;

        debug!("Adding connection via Settings API");
        let add_reply = settings_proxy
            .call_method("AddConnection", &(settings,))
            .await?;
        let conn_path: OwnedObjectPath = add_reply.body().deserialize()?;
        debug!("Connection added, activating VPN connection");

        nm.activate_connection(conn_path, vpn_device_path, specific_object)
            .await?
    };

    wait_for_connection_activation(conn, &active_conn).await?;
    debug!("Connection reached Activated state, waiting briefly...");

    match NMActiveConnectionProxy::builder(conn).path(active_conn.clone()) {
        Ok(builder) => match builder.build().await {
            Ok(active_conn_check) => {
                let final_state = active_conn_check.state().await?;
                let state = crate::api::models::ActiveConnectionState::from(final_state);
                debug!("Connection state after delay: {:?}", state);

                match state {
                    crate::api::models::ActiveConnectionState::Activated => {
                        info!("Successfully connected to VPN: {}", creds.name);
                        Ok(())
                    }
                    crate::api::models::ActiveConnectionState::Deactivated => {
                        warn!("Connection deactivated immediately after activation");
                        Err(crate::api::models::ConnectionError::ActivationFailed(
                            crate::api::models::ConnectionStateReason::Unknown,
                        ))
                    }
                    _ => {
                        warn!("Connection in unexpected state: {:?}", state);
                        Err(crate::api::models::ConnectionError::Stuck(format!(
                            "connection in state {:?}",
                            state
                        )))
                    }
                }
            }
            Err(e) => {
                warn!("Failed to build active connection proxy after delay: {}", e);
                Err(crate::api::models::ConnectionError::ActivationFailed(
                    crate::api::models::ConnectionStateReason::Unknown,
                ))
            }
        },
        Err(e) => {
            warn!(
                "Failed to create active connection proxy builder after delay: {}",
                e
            );
            Err(crate::api::models::ConnectionError::ActivationFailed(
                crate::api::models::ConnectionStateReason::Unknown,
            ))
        }
    }
}

/// Disconnects from a connection by name.
///
/// Searches through active connections for a WireGuard connection matching the given name.
/// If found, deactivates the connection. If not found, assumes already
/// disconnected and returns success.
pub(crate) async fn disconnect_vpn(conn: &Connection, name: &str) -> Result<()> {
    debug!("Disconnecting VPN: {name}");

    let nm = NMProxy::new(conn).await?;
    let active_conns = match nm.active_connections().await {
        Ok(conns) => conns,
        Err(e) => {
            debug!("Failed to get active connections: {}", e);
            info!("Disconnected VPN: {name} (could not verify active state)");
            return Ok(());
        }
    };

    for ac_path in active_conns {
        let ac_proxy: zbus::Proxy<'_> = match zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(ac_path.clone())?
            .interface("org.freedesktop.NetworkManager.Connection.Active")?
            .build()
            .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        let conn_path: OwnedObjectPath = match ac_proxy.call_method("Connection", &()).await {
            Ok(msg) => match msg.body().deserialize::<OwnedObjectPath>() {
                Ok(path) => path,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let cproxy: zbus::Proxy<'_> = match zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(conn_path.clone())?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(msg) => msg,
            Err(_) => continue,
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
            match body.deserialize() {
                Ok(map) => map,
                Err(_) => continue,
            };

        if let Some(conn_sec) = settings_map.get("connection") {
            let id_match = conn_sec
                .get("id")
                .and_then(|v| match v {
                    zvariant::Value::Str(s) => Some(s.as_str() == name),
                    _ => None,
                })
                .unwrap_or(false);

            let is_wireguard = conn_sec
                .get("type")
                .and_then(|v| match v {
                    zvariant::Value::Str(s) => Some(s.as_str() == "wireguard"),
                    _ => None,
                })
                .unwrap_or(false);

            if id_match && is_wireguard {
                debug!("Found active WireGuard connection, deactivating: {name}");
                let _ = nm.deactivate_connection(ac_path).await;
                info!("Successfully disconnected VPN: {name}");
                return Ok(());
            }
        }
    }

    info!("Disconnected VPN: {name} (not active)");
    Ok(())
}

/// Lists all saved WireGuard connections with their current state.
///
/// Returns connections where `connection.type == "wireguard"`.
/// For active connections, populates `state` and `interface` from the active connection.
pub(crate) async fn list_vpn_connections(conn: &Connection) -> Result<Vec<VpnConnection>> {
    let nm = NMProxy::new(conn).await?;

    let settings: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.NetworkManager")?
        .path("/org/freedesktop/NetworkManager/Settings")?
        .interface("org.freedesktop.NetworkManager.Settings")?
        .build()
        .await?;

    let list_reply = settings.call_method("ListConnections", &()).await?;
    let saved_conns: Vec<OwnedObjectPath> = list_reply.body().deserialize()?;

    // Map active WireGuard connection id -> (state, interface)
    let active_conns = nm.active_connections().await?;
    let mut active_wg_map: HashMap<String, (DeviceState, Option<String>)> = HashMap::new();

    for ac_path in active_conns {
        let ac_proxy: zbus::Proxy<'_> = match zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(ac_path.clone())?
            .interface("org.freedesktop.NetworkManager.Connection.Active")?
            .build()
            .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        let conn_path: OwnedObjectPath = match ac_proxy.call_method("Connection", &()).await {
            Ok(msg) => match msg.body().deserialize::<OwnedObjectPath>() {
                Ok(p) => p,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let cproxy: zbus::Proxy<'_> = match zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(conn_path)?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
            match body.deserialize() {
                Ok(m) => m,
                Err(_) => continue,
            };

        let conn_sec = match settings_map.get("connection") {
            Some(s) => s,
            None => continue,
        };

        let id = match conn_sec.get("id") {
            Some(zvariant::Value::Str(s)) => s.as_str().to_string(),
            _ => continue,
        };

        let conn_type = match conn_sec.get("type") {
            Some(zvariant::Value::Str(s)) => s.as_str(),
            _ => continue,
        };

        if conn_type != "wireguard" {
            continue;
        }

        let state = if let Ok(state_val) = ac_proxy.get_property::<u32>("State").await {
            DeviceState::from(state_val)
        } else {
            DeviceState::Other(0)
        };

        let interface = if let Ok(dev_paths) = ac_proxy
            .get_property::<Vec<OwnedObjectPath>>("Devices")
            .await
        {
            if let Some(dev_path) = dev_paths.first() {
                match zbus::proxy::Builder::<zbus::Proxy>::new(conn)
                    .destination("org.freedesktop.NetworkManager")?
                    .path(dev_path.clone())?
                    .interface("org.freedesktop.NetworkManager.Device")?
                    .build()
                    .await
                {
                    Ok(dev_proxy) => dev_proxy.get_property::<String>("Interface").await.ok(),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        active_wg_map.insert(id, (state, interface));
    }

    let mut wg_conns = Vec::new();

    for cpath in saved_conns {
        let cproxy: zbus::Proxy<'_> = match zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(cpath.clone())?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
            match body.deserialize() {
                Ok(m) => m,
                Err(_) => continue,
            };

        let conn_sec = match settings_map.get("connection") {
            Some(s) => s,
            None => continue,
        };

        let id = match conn_sec.get("id") {
            Some(zvariant::Value::Str(s)) => s.as_str().to_string(),
            _ => continue,
        };

        let conn_type = match conn_sec.get("type") {
            Some(zvariant::Value::Str(s)) => s.as_str(),
            _ => continue,
        };

        if conn_type != "wireguard" {
            continue;
        }

        let (state, interface) = active_wg_map
            .get(&id)
            .cloned()
            .unwrap_or((DeviceState::Other(0), None));

        wg_conns.push(VpnConnection {
            name: id,
            vpn_type: VpnType::WireGuard,
            interface,
            state,
        });
    }

    Ok(wg_conns)
}

/// Forgets (deletes) a saved WireGuard connection by name.
///
/// If currently connected, the connection will be disconnected first before deletion.
pub(crate) async fn forget_vpn(conn: &Connection, name: &str) -> Result<()> {
    debug!("Starting forget operation for VPN: {name}");

    let _ = disconnect_vpn(conn, name).await;

    let settings: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.NetworkManager")?
        .path("/org/freedesktop/NetworkManager/Settings")?
        .interface("org.freedesktop.NetworkManager.Settings")?
        .build()
        .await?;

    let list_reply = settings.call_method("ListConnections", &()).await?;
    let conns: Vec<OwnedObjectPath> = list_reply.body().deserialize()?;

    for cpath in conns {
        let cproxy: zbus::Proxy<'_> = match zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(cpath.clone())?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        if let Ok(msg) = cproxy.call_method("GetSettings", &()).await {
            let body = msg.body();
            let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
                body.deserialize()?;

            if let Some(conn_sec) = settings_map.get("connection") {
                let id_ok = conn_sec
                    .get("id")
                    .and_then(|v| match v {
                        zvariant::Value::Str(s) => Some(s.as_str() == name),
                        _ => None,
                    })
                    .unwrap_or(false);

                let type_ok = conn_sec
                    .get("type")
                    .and_then(|v| match v {
                        zvariant::Value::Str(s) => Some(s.as_str() == "wireguard"),
                        _ => None,
                    })
                    .unwrap_or(false);

                if id_ok && type_ok {
                    debug!("Found WireGuard connection, deleting: {name}");
                    cproxy.call_method("Delete", &()).await?;
                    info!("Successfully deleted VPN connection: {name}");
                    return Ok(());
                }
            }
        }
    }

    debug!("No saved VPN connection found for '{name}'");
    Err(crate::api::models::ConnectionError::NoSavedConnection)
}

/// Gets detailed information about a WireGuard connection.
///
/// The connection must be in the active connections list to retrieve full details.
pub(crate) async fn get_vpn_info(conn: &Connection, name: &str) -> Result<VpnConnectionInfo> {
    let nm = NMProxy::new(conn).await?;
    let active_conns = nm.active_connections().await?;

    for ac_path in active_conns {
        let ac_proxy: zbus::Proxy<'_> = match zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(ac_path.clone())?
            .interface("org.freedesktop.NetworkManager.Connection.Active")?
            .build()
            .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        let conn_path: OwnedObjectPath = match ac_proxy.call_method("Connection", &()).await {
            Ok(msg) => match msg.body().deserialize::<OwnedObjectPath>() {
                Ok(p) => p,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let cproxy: zbus::Proxy<'_> = match zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(conn_path)?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await
        {
            Ok(p) => p,
            Err(_) => continue,
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(m) => m,
            Err(_) => continue,
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
            match body.deserialize() {
                Ok(m) => m,
                Err(_) => continue,
            };

        let conn_sec = match settings_map.get("connection") {
            Some(s) => s,
            None => continue,
        };

        let id = match conn_sec.get("id") {
            Some(zvariant::Value::Str(s)) => s.as_str(),
            _ => continue,
        };

        let conn_type = match conn_sec.get("type") {
            Some(zvariant::Value::Str(s)) => s.as_str(),
            _ => continue,
        };

        if conn_type != "wireguard" || id != name {
            continue;
        }

        // WireGuard type is known by connection.type
        let vpn_type = VpnType::WireGuard;

        // ActiveConnection state
        let state_val: u32 = ac_proxy.get_property("State").await?;
        let state = DeviceState::from(state_val);

        // Device/interface
        let dev_paths: Vec<OwnedObjectPath> = ac_proxy.get_property("Devices").await?;
        let interface = if let Some(dev_path) = dev_paths.first() {
            let dev_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
                .destination("org.freedesktop.NetworkManager")?
                .path(dev_path.clone())?
                .interface("org.freedesktop.NetworkManager.Device")?
                .build()
                .await?;
            Some(dev_proxy.get_property::<String>("Interface").await?)
        } else {
            None
        };

        // Best-effort endpoint extraction from wireguard.peers (nmcli-style string)
        // This is not guaranteed to exist or be populated.
        let gateway = settings_map
            .get("wireguard")
            .and_then(|wg_sec| wg_sec.get("peers"))
            .and_then(|v| match v {
                zvariant::Value::Str(s) => Some(s.as_str().to_string()),
                _ => None,
            })
            .and_then(|peers| {
                // peers: "pubkey endpoint=host:port allowed-ips=... , pubkey2 ..."
                let first = peers.split(',').next()?.trim().to_string();
                for tok in first.split_whitespace() {
                    if let Some(rest) = tok.strip_prefix("endpoint=") {
                        return Some(rest.to_string());
                    }
                }
                None
            });

        // IPv4 config
        let ip4_path: OwnedObjectPath = ac_proxy.get_property("Ip4Config").await?;
        let (ip4_address, dns_servers) = if ip4_path.as_str() != "/" {
            let ip4_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
                .destination("org.freedesktop.NetworkManager")?
                .path(ip4_path)?
                .interface("org.freedesktop.NetworkManager.IP4Config")?
                .build()
                .await?;

            let ip4_address = if let Ok(addr_array) = ip4_proxy
                .get_property::<Vec<HashMap<String, zvariant::Value>>>("AddressData")
                .await
            {
                addr_array.first().and_then(|addr_map| {
                    let address = addr_map.get("address").and_then(|v| match v {
                        zvariant::Value::Str(s) => Some(s.as_str().to_string()),
                        _ => None,
                    })?;
                    let prefix = addr_map.get("prefix").and_then(|v| match v {
                        zvariant::Value::U32(p) => Some(*p),
                        _ => None,
                    })?;
                    Some(format!("{}/{}", address, prefix))
                })
            } else {
                None
            };

            let dns_servers =
                if let Ok(dns_array) = ip4_proxy.get_property::<Vec<u32>>("Nameservers").await {
                    dns_array
                        .iter()
                        .map(|ip| {
                            format!(
                                "{}.{}.{}.{}",
                                ip & 0xFF,
                                (ip >> 8) & 0xFF,
                                (ip >> 16) & 0xFF,
                                (ip >> 24) & 0xFF
                            )
                        })
                        .collect()
                } else {
                    vec![]
                };

            (ip4_address, dns_servers)
        } else {
            (None, vec![])
        };

        // IPv6 config parsing not implemented
        let ip6_address = None;

        return Ok(VpnConnectionInfo {
            name: id.to_string(),
            vpn_type,
            state,
            interface,
            gateway,
            ip4_address,
            ip6_address,
            dns_servers,
        });
    }

    Err(crate::api::models::ConnectionError::NoVpnConnection)
}
