//! Core VPN connection management logic.
//!
//! This module contains internal implementation for managing VPN connections
//! through NetworkManager, including connecting, disconnecting, listing, and
//! deleting VPN profiles.
//!
//! Currently supports:
//! - WireGuard VPN connections
//!
//! These functions are not part of the public API and should be accessed
//! through the [`NetworkManager`][crate::NetworkManager] interface.

use log::{debug, info};
use std::collections::HashMap;
use zbus::Connection;
use zvariant::OwnedObjectPath;

use crate::api::models::{
    ConnectionOptions, DeviceState, VpnConnection, VpnConnectionInfo, VpnCredentials, VpnType,
};
use crate::builders::build_wireguard_connection;
use crate::core::state_wait::wait_for_connection_activation;
use crate::dbus::NMProxy;
use crate::Result;

/// Connects to a VPN using WireGuard.
///
/// This function checks for an existing saved VPN connection by name.
/// If found, it activates the saved connection. If not found, it creates
/// a new WireGuard VPN connection using the provided credentials.
/// The function waits for the connection to reach the activated state
/// before returning.
///
/// VPN connections do not have a specific device or access point,
/// so empty object paths are used for those parameters.
pub(crate) async fn connect_vpn(conn: &Connection, creds: VpnCredentials) -> Result<()> {
    debug!("Connecting to VPN: {}", creds.name);

    let nm = NMProxy::new(conn).await?;

    // Check saved connections
    let saved =
        crate::core::connection_settings::get_saved_connection_path(conn, &creds.name).await?;

    // VPNs do not have a device path or specific_object
    // So we use an empty path for both instead
    let d_path = OwnedObjectPath::try_from("/").unwrap();
    let specific_object = OwnedObjectPath::try_from("/").unwrap();

    let active_conn = if let Some(saved_path) = saved {
        debug!("Activated existent VPN connection");
        nm.activate_connection(saved_path, d_path, specific_object)
            .await?
    } else {
        debug!("Creating new VPN connection");
        let opts = ConnectionOptions {
            autoconnect: false,
            autoconnect_priority: None,
            autoconnect_retries: None,
        };

        let settings = build_wireguard_connection(&creds, &opts);
        let (_, active_conn) = nm
            .add_and_activate_connection(settings?, d_path, specific_object)
            .await?;
        active_conn
    };

    wait_for_connection_activation(conn, &active_conn).await?;

    info!("Successfully connected to VPN: {}", creds.name);
    Ok(())
}

/// Disconnects from a VPN by name.
///
/// Searches through active connections for a VPN matching the given name.
/// If found, deactivates the connection. If not found, assumes already
/// disconnected and returns success.
pub(crate) async fn disconnect_vpn(conn: &Connection, name: &str) -> Result<()> {
    debug!("Disconnecting VPN: {name}");

    let nm = NMProxy::new(conn).await?;
    let active_conns = match nm.active_connections().await {
        Ok(conns) => conns,
        Err(e) => {
            debug!("Failed to get active connections: {}", e);
            // If we can't get active connections, assume VPN is not active
            info!("Disconnected VPN: {name} (could not verify active state)");
            return Ok(());
        }
    };

    for ac_path in active_conns {
        let ac_proxy_result = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")
            .map(|b| b.path(ac_path.clone()))
            .and_then(|r| r)
            .map(|b| b.interface("org.freedesktop.NetworkManager.Connection.Active"))
            .and_then(|r| r);

        let ac_proxy: zbus::Proxy<'_> = match ac_proxy_result {
            Ok(builder) => match builder.build().await {
                Ok(proxy) => proxy,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let conn_path = match ac_proxy.call_method("Connection", &()).await {
            Ok(msg) => match msg.body().deserialize::<OwnedObjectPath>() {
                Ok(path) => path,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        let cproxy_result = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")
            .map(|b| b.path(conn_path.clone()))
            .and_then(|r| r)
            .map(|b| b.interface("org.freedesktop.NetworkManager.Settings.Connection"))
            .and_then(|r| r);

        let cproxy: zbus::Proxy<'_> = match cproxy_result {
            Ok(builder) => match builder.build().await {
                Ok(proxy) => proxy,
                Err(_) => continue,
            },
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
            if let Some(zvariant::Value::Str(id)) = conn_sec.get("id") {
                if id.as_str() == name {
                    debug!("Found active VPN connection, deactivating: {name}");
                    let _ = nm.deactivate_connection(ac_path).await; // Ignore errors on deactivation
                    info!("Successfully disconnected VPN: {name}");
                    return Ok(());
                }
            }
        }
    }

    info!("Disconnected VPN: {name} (not active)");
    Ok(())
}

/// Lists all saved VPN connections with their current state.
///
/// Queries NetworkManager's saved connection settings and returns a list of
/// all VPN connections, including their name, type, current state, and interface.
/// Only returns VPN connections with recognized VPN types (currently WireGuard).
///
/// For active VPN connections, this function populates the `state` and `interface`
/// fields by querying active connections.
pub(crate) async fn list_vpn_connections(conn: &Connection) -> Result<Vec<VpnConnection>> {
    let nm = NMProxy::new(conn).await?;

    let settings: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.NetworkManager")?
        .path("/org/freedesktop/NetworkManager/Settings")?
        .interface("org.freedesktop.NetworkManager.Settings")?
        .build()
        .await?;

    let list_reply = settings.call_method("ListConnections", &()).await?;
    let body = list_reply.body();
    let saved_conns: Vec<OwnedObjectPath> = body.deserialize()?;

    // Get active connections to populate state/interface
    let active_conns = nm.active_connections().await?;
    let mut active_vpn_map: HashMap<String, (DeviceState, Option<String>)> = HashMap::new();

    for ac_path in active_conns {
        let ac_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(ac_path.clone())?
            .interface("org.freedesktop.NetworkManager.Connection.Active")?
            .build()
            .await?;

        // Get the connection path
        if let Ok(conn_msg) = ac_proxy.call_method("Connection", &()).await {
            if let Ok(conn_path) = conn_msg.body().deserialize::<OwnedObjectPath>() {
                // Get connection settings to find the name
                let cproxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
                    .destination("org.freedesktop.NetworkManager")?
                    .path(conn_path)?
                    .interface("org.freedesktop.NetworkManager.Settings.Connection")?
                    .build()
                    .await?;

                if let Ok(msg) = cproxy.call_method("GetSettings", &()).await {
                    if let Ok(settings_map) = msg
                        .body()
                        .deserialize::<HashMap<String, HashMap<String, zvariant::Value>>>()
                    {
                        if let Some(conn_sec) = settings_map.get("connection") {
                            if let Some(zvariant::Value::Str(id)) = conn_sec.get("id") {
                                if let Some(zvariant::Value::Str(conn_type)) = conn_sec.get("type")
                                {
                                    if conn_type.as_str() == "vpn" {
                                        // Get state
                                        let state = if let Ok(state_val) =
                                            ac_proxy.get_property::<u32>("State").await
                                        {
                                            DeviceState::from(state_val)
                                        } else {
                                            DeviceState::Other(0)
                                        };

                                        // Get devices (which includes interface info)
                                        let interface = if let Ok(dev_paths) = ac_proxy
                                            .get_property::<Vec<OwnedObjectPath>>("Devices")
                                            .await
                                        {
                                            if let Some(dev_path) = dev_paths.first() {
                                                // Get device interface name
                                                match zbus::proxy::Builder::<zbus::Proxy>::new(conn)
                                                    .destination("org.freedesktop.NetworkManager")?
                                                    .path(dev_path.clone())?
                                                    .interface(
                                                        "org.freedesktop.NetworkManager.Device",
                                                    )?
                                                    .build()
                                                    .await
                                                {
                                                    Ok(dev_proxy) => dev_proxy
                                                        .get_property::<String>("Interface")
                                                        .await
                                                        .ok(),
                                                    Err(_) => None,
                                                }
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        };

                                        active_vpn_map.insert(id.to_string(), (state, interface));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut vpn_conns = Vec::new();

    for cpath in saved_conns {
        let cproxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(cpath.clone())?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await?;

        let msg = cproxy.call_method("GetSettings", &()).await?;
        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> = body.deserialize()?;

        if let Some(conn_sec) = settings_map.get("connection") {
            if let Some(zvariant::Value::Str(id)) = conn_sec.get("id") {
                if let Some(zvariant::Value::Str(conn_type)) = conn_sec.get("type") {
                    if conn_type.as_str() == "vpn" {
                        // Extract VPN service-type and convert to VpnType enum
                        let vpn_type = settings_map
                            .get("vpn")
                            .and_then(|vpn_sec| vpn_sec.get("service-type"))
                            .and_then(|v| match v {
                                zvariant::Value::Str(s) => {
                                    // Match against known service types
                                    match s.as_str() {
                                        "org.freedesktop.NetworkManager.wireguard" => {
                                            Some(VpnType::WireGuard)
                                        }
                                        _ => None, // Unknown VPN types are skipped for now
                                    }
                                }
                                _ => None,
                            });

                        // Only add VPN connections with recognized types
                        if let Some(vpn_type) = vpn_type {
                            let name = id.to_string();
                            let (state, interface) = active_vpn_map
                                .get(&name)
                                .cloned()
                                .unwrap_or((DeviceState::Other(0), None));

                            vpn_conns.push(VpnConnection {
                                name,
                                vpn_type,
                                interface,
                                state,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(vpn_conns)
}

/// Forgets (deletes) a saved VPN connection by name.
///
/// Searches through saved connections for a VPN matching the given name.
/// If found, deletes the connection profile. If not found, returns
/// `NoSavedConnection` error. If currently connected, the VPN will be
/// disconnected first before deletion.
pub(crate) async fn forget_vpn(conn: &Connection, name: &str) -> Result<()> {
    debug!("Starting forget operation for VPN: {name}");

    // First, disconnect if currently active
    let _ = disconnect_vpn(conn, name).await;

    let settings: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
        .destination("org.freedesktop.NetworkManager")?
        .path("/org/freedesktop/NetworkManager/Settings")?
        .interface("org.freedesktop.NetworkManager.Settings")?
        .build()
        .await?;

    let list_reply = settings.call_method("ListConnections", &()).await?;
    let body = list_reply.body();
    let conns: Vec<OwnedObjectPath> = body.deserialize()?;

    for cpath in conns {
        let cproxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(cpath.clone())?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await?;

        if let Ok(msg) = cproxy.call_method("GetSettings", &()).await {
            let body = msg.body();
            let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
                body.deserialize()?;

            if let Some(conn_sec) = settings_map.get("connection") {
                if let Some(zvariant::Value::Str(id)) = conn_sec.get("id") {
                    if let Some(zvariant::Value::Str(conn_type)) = conn_sec.get("type") {
                        if conn_type.as_str() == "vpn" && id.as_str() == name {
                            debug!("Found VPN connection, deleting: {name}");
                            cproxy.call_method("Delete", &()).await?;
                            info!("Successfully deleted VPN connection: {name}");
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    debug!("No saved VPN connection found for '{name}'");
    Err(crate::api::models::ConnectionError::NoSavedConnection)
}

/// Gets detailed information about a VPN connection.
///
/// Queries NetworkManager for comprehensive information about a VPN connection,
/// including IP configuration, DNS servers, and connection state. The VPN must
/// be in the active connections list to retrieve full details.
///
/// # Arguments
///
/// * `conn` - D-Bus connection
/// * `name` - The name of the VPN connection
///
/// # Returns
///
/// Returns `VpnConnectionInfo` with detailed connection information, or an
/// error if the VPN is not found or not active.
pub(crate) async fn get_vpn_info(conn: &Connection, name: &str) -> Result<VpnConnectionInfo> {
    let nm = NMProxy::new(conn).await?;
    let active_conns = nm.active_connections().await?;

    for ac_path in active_conns {
        let ac_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(ac_path.clone())?
            .interface("org.freedesktop.NetworkManager.Connection.Active")?
            .build()
            .await?;

        // Get the connection path
        let conn_msg = ac_proxy.call_method("Connection", &()).await?;
        let conn_path: OwnedObjectPath = conn_msg.body().deserialize()?;

        let cproxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
            .destination("org.freedesktop.NetworkManager")?
            .path(conn_path)?
            .interface("org.freedesktop.NetworkManager.Settings.Connection")?
            .build()
            .await?;

        let msg = cproxy.call_method("GetSettings", &()).await?;
        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> = body.deserialize()?;

        if let Some(conn_sec) = settings_map.get("connection") {
            if let Some(zvariant::Value::Str(id)) = conn_sec.get("id") {
                if let Some(zvariant::Value::Str(conn_type)) = conn_sec.get("type") {
                    if conn_type.as_str() == "vpn" && id.as_str() == name {
                        // Found the VPN connection, get details
                        let vpn_type = settings_map
                            .get("vpn")
                            .and_then(|vpn_sec| vpn_sec.get("service-type"))
                            .and_then(|v| match v {
                                zvariant::Value::Str(s) => match s.as_str() {
                                    "org.freedesktop.NetworkManager.wireguard" => {
                                        Some(VpnType::WireGuard)
                                    }
                                    _ => None,
                                },
                                _ => None,
                            })
                            .ok_or_else(|| crate::api::models::ConnectionError::NoVpnConnection)?;

                        // Get state
                        let state_val: u32 = ac_proxy.get_property("State").await?;
                        let state = DeviceState::from(state_val);

                        // Get interface
                        let dev_paths: Vec<OwnedObjectPath> =
                            ac_proxy.get_property("Devices").await?;
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

                        // Get gateway from VPN settings
                        let gateway = settings_map
                            .get("vpn")
                            .and_then(|vpn_sec| vpn_sec.get("data"))
                            .and_then(|data| match data {
                                zvariant::Value::Dict(dict) => {
                                    // Try to find gateway/endpoint in the data
                                    for entry in dict.iter() {
                                        let (key_val, value_val) = entry;
                                        if let zvariant::Value::Str(key) = key_val {
                                            if key.as_str().contains("endpoint")
                                                || key.as_str().contains("gateway")
                                            {
                                                if let zvariant::Value::Str(val) = value_val {
                                                    return Some(val.as_str().to_string());
                                                }
                                            }
                                        }
                                    }
                                    None
                                }
                                _ => None,
                            });

                        // Get IP4 configuration
                        let ip4_path: OwnedObjectPath = ac_proxy.get_property("Ip4Config").await?;
                        let (ip4_address, dns_servers) = if ip4_path.as_str() != "/" {
                            let ip4_proxy: zbus::Proxy<'_> = zbus::proxy::Builder::new(conn)
                                .destination("org.freedesktop.NetworkManager")?
                                .path(ip4_path)?
                                .interface("org.freedesktop.NetworkManager.IP4Config")?
                                .build()
                                .await?;

                            // Get address data
                            let ip4_address = if let Ok(addr_array) = ip4_proxy
                                .get_property::<Vec<HashMap<String, zvariant::Value>>>(
                                    "AddressData",
                                )
                                .await
                            {
                                addr_array.first().and_then(|addr_map| {
                                    let address =
                                        addr_map.get("address").and_then(|v| match v {
                                            zvariant::Value::Str(s) => Some(s.as_str().to_string()),
                                            _ => None,
                                        })?;
                                    let prefix = addr_map.get("prefix").and_then(|v| match v {
                                        zvariant::Value::U32(p) => Some(p),
                                        _ => None,
                                    })?;
                                    Some(format!("{}/{}", address, prefix))
                                })
                            } else {
                                None
                            };

                            // Get DNS servers
                            let dns_servers = if let Ok(dns_array) =
                                ip4_proxy.get_property::<Vec<u32>>("Nameservers").await
                            {
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

                        // Get IP6 configuration
                        // Note: IPv6 address parsing is not yet implemented.
                        // This is a known limitation documented in VpnConnectionInfo.
                        let ip6_path: OwnedObjectPath = ac_proxy.get_property("Ip6Config").await?;
                        let ip6_address = if ip6_path.as_str() != "/" {
                            // TODO: Implement IPv6 address parsing
                            None
                        } else {
                            None
                        };

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
                }
            }
        }
    }

    Err(crate::api::models::ConnectionError::NoVpnConnection)
}
