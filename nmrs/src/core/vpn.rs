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
#![allow(deprecated)]

use log::{debug, info, warn};
use std::collections::HashMap;
use zbus::Connection;
use zvariant::OwnedObjectPath;

use crate::Result;
use crate::api::models::{
    ConnectionError, ConnectionOptions, DeviceState, TimeoutConfig, VpnConfig, VpnConnection,
    VpnConnectionInfo, VpnCredentials, VpnDetails, VpnType,
};
use crate::builders::{build_openvpn_connection, build_wireguard_connection};
use crate::core::state_wait::wait_for_connection_activation;
use crate::dbus::{NMActiveConnectionProxy, NMProxy};
use crate::models::VpnConfiguration;
use crate::util::utils::{extract_connection_state_reason, nm_proxy, settings_proxy};
use crate::util::validation::{
    validate_connection_name, validate_openvpn_config, validate_vpn_credentials,
};

// Detects the VPN type from a raw NM connection settings map.
// WireGuard: connection.type == "wireguard"
// OpenVPN:   connection.type == "vpn" + vpn.service-type == "org.freedesktop.NetworkManager.openvpn"
fn detect_vpn_type(
    settings: &HashMap<String, HashMap<String, zvariant::Value<'_>>>,
) -> Option<VpnType> {
    let conn = settings.get("connection")?;
    let conn_type = match conn.get("type")? {
        zvariant::Value::Str(s) => s.as_str(),
        _ => return None,
    };

    match conn_type {
        "wireguard" => Some(VpnType::WireGuard),
        "vpn" => {
            let vpn = settings.get("vpn")?;
            let service = match vpn.get("service-type")? {
                zvariant::Value::Str(s) => s.as_str(),
                _ => return None,
            };
            if service == "org.freedesktop.NetworkManager.openvpn" {
                Some(VpnType::OpenVpn)
            } else {
                None
            }
        }
        _ => None,
    }
}

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
pub(crate) async fn connect_vpn(
    conn: &Connection,
    config: VpnConfiguration,
    timeout_config: Option<TimeoutConfig>,
) -> Result<()> {
    // Validate VPN credentials before attempting connection
    let name = config.name().to_string();
    debug!("Connecting to VPN: {}", name);

    let nm = NMProxy::new(conn).await?;

    // Check saved connections
    let saved = crate::core::connection_settings::get_saved_connection_path(conn, &name).await?;

    // For WireGuard activation, always use "/" as device path - NetworkManager will auto-select
    let vpn_device_path = OwnedObjectPath::default();
    let specific_object = OwnedObjectPath::default();

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

        let settings = match config {
            VpnConfiguration::WireGuard(ref wg) => {
                let creds: VpnCredentials = wg.clone().into();
                validate_vpn_credentials(&creds)?;
                build_wireguard_connection(&creds, &opts)?
            }
            VpnConfiguration::OpenVpn(ref ovpn) => {
                validate_openvpn_config(ovpn)?;
                build_openvpn_connection(ovpn, &opts)?
            }
        };

        let settings_api = settings_proxy(conn).await?;

        debug!("Adding connection via Settings API");
        let add_reply = settings_api
            .call_method("AddConnection", &(settings,))
            .await?;
        let conn_path: OwnedObjectPath = add_reply.body().deserialize()?;
        debug!("Connection added, activating VPN connection");

        nm.activate_connection(conn_path, vpn_device_path, specific_object)
            .await?
    };

    let timeout = timeout_config.map(|c| c.connection_timeout);
    wait_for_connection_activation(conn, &active_conn, timeout).await?;
    debug!("Connection reached Activated state, waiting briefly...");

    match NMActiveConnectionProxy::builder(conn).path(active_conn.clone()) {
        Ok(builder) => match builder.build().await {
            Ok(active_conn_check) => {
                let final_state = active_conn_check.state().await?;
                let state = crate::api::models::ActiveConnectionState::from(final_state);
                debug!("Connection state after delay: {:?}", state);

                match state {
                    crate::api::models::ActiveConnectionState::Activated => {
                        info!("Successfully connected to VPN: {}", name);
                        Ok(())
                    }
                    crate::api::models::ActiveConnectionState::Deactivated => {
                        warn!("Connection deactivated immediately after activation");
                        let reason = extract_connection_state_reason(conn, &active_conn).await;
                        Err(crate::api::models::ConnectionError::ActivationFailed(
                            reason,
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
                let reason = extract_connection_state_reason(conn, &active_conn).await;
                Err(crate::api::models::ConnectionError::ActivationFailed(
                    reason,
                ))
            }
        },
        Err(e) => {
            warn!(
                "Failed to create active connection proxy builder after delay: {}",
                e
            );
            let reason = extract_connection_state_reason(conn, &active_conn).await;
            Err(crate::api::models::ConnectionError::ActivationFailed(
                reason,
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
    // Validate connection name
    validate_connection_name(name)?;

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
        let ac_proxy = match nm_proxy(
            conn,
            ac_path.clone(),
            "org.freedesktop.NetworkManager.Connection.Active",
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Failed to create proxy for active connection {}: {}",
                    ac_path, e
                );
                continue;
            }
        };

        let conn_path: OwnedObjectPath = match ac_proxy.call_method("Connection", &()).await {
            Ok(msg) => match msg.body().deserialize::<OwnedObjectPath>() {
                Ok(path) => path,
                Err(e) => {
                    warn!(
                        "Failed to deserialize connection path for {}: {}",
                        ac_path, e
                    );
                    continue;
                }
            },
            Err(e) => {
                warn!("Failed to get Connection property from {}: {}", ac_path, e);
                continue;
            }
        };

        let cproxy = match nm_proxy(
            conn,
            conn_path.clone(),
            "org.freedesktop.NetworkManager.Settings.Connection",
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Failed to create proxy for connection settings {}: {}",
                    conn_path, e
                );
                continue;
            }
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to get settings for connection {}: {}", conn_path, e);
                continue;
            }
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
            match body.deserialize() {
                Ok(map) => map,
                Err(e) => {
                    warn!("Failed to deserialize settings for {}: {}", conn_path, e);
                    continue;
                }
            };

        if let Some(conn_sec) = settings_map.get("connection") {
            let id_match = conn_sec
                .get("id")
                .and_then(|v| match v {
                    zvariant::Value::Str(s) => Some(s.as_str() == name),
                    _ => None,
                })
                .unwrap_or(false);

            let is_vpn = detect_vpn_type(&settings_map).is_some();

            if id_match && is_vpn {
                debug!("Found active VPN connection, deactivating: {name}");
                match nm.deactivate_connection(ac_path.clone()).await {
                    Ok(_) => info!("Successfully disconnected VPN: {name}"),
                    Err(e) => warn!("Failed to deactivate connection {}: {}", ac_path, e),
                }
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

    let settings = nm_proxy(
        conn,
        "/org/freedesktop/NetworkManager/Settings",
        "org.freedesktop.NetworkManager.Settings",
    )
    .await?;

    let list_reply = settings
        .call_method("ListConnections", &())
        .await
        .map_err(|e| ConnectionError::DbusOperation {
            context: "failed to list saved connections".to_string(),
            source: e,
        })?;

    let saved_conns: Vec<OwnedObjectPath> = list_reply.body().deserialize()?;

    // Map active WireGuard connection id -> (state, interface)
    let active_conns = nm.active_connections().await?;
    let mut active_wg_map: HashMap<String, (DeviceState, Option<String>)> = HashMap::new();

    for ac_path in active_conns {
        let ac_proxy = match nm_proxy(
            conn,
            ac_path.clone(),
            "org.freedesktop.NetworkManager.Connection.Active",
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Failed to create proxy for active connection {}: {}",
                    ac_path, e
                );
                continue;
            }
        };

        let conn_path: OwnedObjectPath = match ac_proxy.call_method("Connection", &()).await {
            Ok(msg) => match msg.body().deserialize::<OwnedObjectPath>() {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "Failed to deserialize connection path for {}: {}",
                        ac_path, e
                    );
                    continue;
                }
            },
            Err(e) => {
                warn!("Failed to get Connection property from {}: {}", ac_path, e);
                continue;
            }
        };

        let cproxy = match nm_proxy(
            conn,
            conn_path.clone(),
            "org.freedesktop.NetworkManager.Settings.Connection",
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Failed to create proxy for connection settings {}: {}",
                    conn_path, e
                );
                continue;
            }
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to get settings for connection {}: {}", conn_path, e);
                continue;
            }
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
            match body.deserialize() {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to deserialize settings for {}: {}", conn_path, e);
                    continue;
                }
            };

        let conn_sec = match settings_map.get("connection") {
            Some(s) => s,
            None => continue,
        };

        let id = match conn_sec.get("id") {
            Some(zvariant::Value::Str(s)) => s.as_str().to_string(),
            _ => continue,
        };

        if detect_vpn_type(&settings_map).is_none() {
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
                match nm_proxy(
                    conn,
                    dev_path.clone(),
                    "org.freedesktop.NetworkManager.Device",
                )
                .await
                {
                    Ok(dev_proxy) => match dev_proxy.get_property::<String>("Interface").await {
                        Ok(iface) => Some(iface),
                        Err(e) => {
                            debug!(
                                "Failed to get interface name for VPN device {}: {}",
                                dev_path, e
                            );
                            None
                        }
                    },
                    Err(e) => {
                        debug!("Failed to create device proxy for {}: {}", dev_path, e);
                        None
                    }
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
        let cproxy = match nm_proxy(
            conn,
            cpath.clone(),
            "org.freedesktop.NetworkManager.Settings.Connection",
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Failed to create proxy for saved connection {}: {}",
                    cpath, e
                );
                continue;
            }
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(m) => m,
            Err(e) => {
                warn!(
                    "Failed to get settings for saved connection {}: {}",
                    cpath, e
                );
                continue;
            }
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
            match body.deserialize() {
                Ok(m) => m,
                Err(e) => {
                    warn!(
                        "Failed to deserialize settings for saved connection {}: {}",
                        cpath, e
                    );
                    continue;
                }
            };

        let conn_sec = match settings_map.get("connection") {
            Some(s) => s,
            None => continue,
        };

        let id = match conn_sec.get("id") {
            Some(zvariant::Value::Str(s)) => s.as_str().to_string(),
            _ => continue,
        };

        let Some(vpn_type) = detect_vpn_type(&settings_map) else {
            continue;
        };

        let (state, interface) = active_wg_map
            .get(&id)
            .cloned()
            .unwrap_or((DeviceState::Other(0), None));

        wg_conns.push(VpnConnection {
            name: id,
            vpn_type,
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
    validate_connection_name(name)?;

    debug!("Starting forget operation for VPN: {name}");

    match disconnect_vpn(conn, name).await {
        Ok(_) => debug!("VPN disconnected before deletion"),
        Err(e) => warn!(
            "Failed to disconnect VPN before deletion (may already be disconnected): {}",
            e
        ),
    }

    let settings = nm_proxy(
        conn,
        "/org/freedesktop/NetworkManager/Settings",
        "org.freedesktop.NetworkManager.Settings",
    )
    .await?;

    let list_reply = settings.call_method("ListConnections", &()).await?;
    let conns: Vec<OwnedObjectPath> = list_reply.body().deserialize()?;

    for cpath in conns {
        let cproxy = match nm_proxy(
            conn,
            cpath.clone(),
            "org.freedesktop.NetworkManager.Settings.Connection",
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!("Failed to create proxy for connection {}: {}", cpath, e);
                continue;
            }
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to get settings for connection {}: {}", cpath, e);
                continue;
            }
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> = body.deserialize()?;

        let id_ok = settings_map
            .get("connection")
            .and_then(|c| c.get("id"))
            .and_then(|v| match v {
                zvariant::Value::Str(s) => Some(s.as_str() == name),
                _ => None,
            })
            .unwrap_or(false);

        let vpn_type = detect_vpn_type(&settings_map);

        if id_ok && vpn_type.is_some() {
            debug!("Found VPN connection, deleting: {name}");
            cproxy.call_method("Delete", &()).await.map_err(|e| {
                ConnectionError::DbusOperation {
                    context: format!("failed to delete VPN connection '{}'", name),
                    source: e,
                }
            })?;
            info!("Successfully deleted VPN connection: {name}");

            if vpn_type == Some(VpnType::OpenVpn)
                && let Err(e) = crate::util::cert_store::cleanup_certs(name)
            {
                warn!("Failed to remove nmrs cert directory for '{}': {}", name, e);
            }
            return Ok(());
        }
    }

    debug!("No saved VPN connection found for '{name}'");
    Ok(())
}

/// Gets detailed information about a WireGuard connection.
///
/// The connection must be in the active connections list to retrieve full details.
pub(crate) async fn get_vpn_info(conn: &Connection, name: &str) -> Result<VpnConnectionInfo> {
    // Validate connection name
    validate_connection_name(name)?;

    let nm = NMProxy::new(conn).await?;
    let active_conns = nm.active_connections().await?;

    for ac_path in active_conns {
        let ac_proxy = match nm_proxy(
            conn,
            ac_path.clone(),
            "org.freedesktop.NetworkManager.Connection.Active",
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Failed to create proxy for active connection {}: {}",
                    ac_path, e
                );
                continue;
            }
        };

        let conn_path: OwnedObjectPath = match ac_proxy.call_method("Connection", &()).await {
            Ok(msg) => match msg.body().deserialize::<OwnedObjectPath>() {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "Failed to deserialize connection path for {}: {}",
                        ac_path, e
                    );
                    continue;
                }
            },
            Err(e) => {
                warn!("Failed to get Connection property from {}: {}", ac_path, e);
                continue;
            }
        };

        let cproxy = match nm_proxy(
            conn,
            conn_path.clone(),
            "org.freedesktop.NetworkManager.Settings.Connection",
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Failed to create proxy for connection settings {}: {}",
                    conn_path, e
                );
                continue;
            }
        };

        let msg = match cproxy.call_method("GetSettings", &()).await {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to get settings for connection {}: {}", conn_path, e);
                continue;
            }
        };

        let body = msg.body();
        let settings_map: HashMap<String, HashMap<String, zvariant::Value>> =
            match body.deserialize() {
                Ok(m) => m,
                Err(e) => {
                    warn!("Failed to deserialize settings for {}: {}", conn_path, e);
                    continue;
                }
            };

        let conn_sec = match settings_map.get("connection") {
            Some(s) => s,
            None => continue,
        };

        let id = match conn_sec.get("id") {
            Some(zvariant::Value::Str(s)) => s.as_str(),
            _ => continue,
        };

        let Some(vpn_type) = detect_vpn_type(&settings_map) else {
            continue;
        };

        if id != name {
            continue;
        }

        // ActiveConnection state
        let state_val: u32 = ac_proxy.get_property("State").await?;
        let state = DeviceState::from(state_val);

        // Device/interface
        let dev_paths: Vec<OwnedObjectPath> = ac_proxy.get_property("Devices").await?;
        let interface = if let Some(dev_path) = dev_paths.first() {
            let dev_proxy = nm_proxy(
                conn,
                dev_path.clone(),
                "org.freedesktop.NetworkManager.Device",
            )
            .await?;
            Some(dev_proxy.get_property::<String>("Interface").await?)
        } else {
            None
        };

        // Best-effort endpoint extraction from the connection settings.
        // WireGuard reads from wireguard.peers (nmcli-style string).
        // OpenVPN reads from vpn.data["remote"] (a{ss} on the D-Bus wire).
        // Neither is guaranteed to be populated.
        let gateway = match vpn_type {
            VpnType::WireGuard => settings_map
                .get("wireguard")
                .and_then(|wg_sec| wg_sec.get("peers"))
                .and_then(|v| match v {
                    zvariant::Value::Str(s) => Some(s.as_str().to_string()),
                    _ => None,
                })
                .and_then(|peers| {
                    let first = peers.split(',').next()?.trim().to_string();
                    for tok in first.split_whitespace() {
                        if let Some(rest) = tok.strip_prefix("endpoint=") {
                            return Some(rest.to_string());
                        }
                    }
                    None
                }),
            VpnType::OpenVpn => extract_openvpn_gateway(&settings_map),
        };

        // IPv4 config
        let ip4_path: OwnedObjectPath = ac_proxy.get_property("Ip4Config").await?;
        let (ip4_address, dns_servers) = if ip4_path.as_str() != "/" {
            let ip4_proxy =
                nm_proxy(conn, ip4_path, "org.freedesktop.NetworkManager.IP4Config").await?;

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
                        zvariant::Value::U32(p) => Some(p),
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

        // IPv6 config
        let ip6_path: OwnedObjectPath = ac_proxy.get_property("Ip6Config").await?;
        let ip6_address = if ip6_path.as_str() != "/" {
            let ip6_proxy =
                nm_proxy(conn, ip6_path, "org.freedesktop.NetworkManager.IP6Config").await?;

            if let Ok(addr_array) = ip6_proxy
                .get_property::<Vec<HashMap<String, zvariant::Value>>>("AddressData")
                .await
            {
                addr_array.first().and_then(|addr_map| {
                    let address = addr_map.get("address").and_then(|v| match v {
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
            }
        } else {
            None
        };

        let details = match vpn_type {
            VpnType::WireGuard => extract_wireguard_details(&settings_map),
            VpnType::OpenVpn => extract_openvpn_details(&settings_map),
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
            details,
        });
    }

    Err(crate::api::models::ConnectionError::NoVpnConnection)
}

// Extracts the remote gateway from an OpenVPN connection's settings map.
//
// NM stores OpenVPN options in vpn.data as a{ss} on the D-Bus wire, which
// zvariant deserialises as Value::Dict(Dict). The "remote" key holds the
// server address (e.g. "vpn.example.com:1194").
fn extract_openvpn_gateway(
    settings_map: &HashMap<String, HashMap<String, zvariant::Value<'_>>>,
) -> Option<String> {
    let zvariant::Value::Dict(dict) = settings_map.get("vpn")?.get("data")? else {
        return None;
    };
    dict.iter().find_map(|(k, v)| match (k, v) {
        (zvariant::Value::Str(k), zvariant::Value::Str(v)) if k.as_str() == "remote" => {
            Some(v.to_string())
        }
        _ => None,
    })
}

/// Extracts a string value from an OpenVPN `vpn.data` dict by key.
fn extract_openvpn_data_value(
    settings_map: &HashMap<String, HashMap<String, zvariant::Value<'_>>>,
    key: &str,
) -> Option<String> {
    let zvariant::Value::Dict(dict) = settings_map.get("vpn")?.get("data")? else {
        return None;
    };
    dict.iter().find_map(|(k, v)| match (k, v) {
        (zvariant::Value::Str(k_str), zvariant::Value::Str(v_str)) if k_str.as_str() == key => {
            Some(v_str.to_string())
        }
        _ => None,
    })
}

fn extract_openvpn_details(
    settings_map: &HashMap<String, HashMap<String, zvariant::Value<'_>>>,
) -> Option<VpnDetails> {
    let remote_raw = extract_openvpn_data_value(settings_map, "remote")?;

    let (remote, port) = if let Some(idx) = remote_raw.rfind(':') {
        let host = remote_raw[..idx].to_string();
        let port = remote_raw[idx + 1..].parse::<u16>().unwrap_or(1194);
        (host, port)
    } else {
        (remote_raw, 1194)
    };

    let protocol =
        if extract_openvpn_data_value(settings_map, "proto-tcp").as_deref() == Some("yes") {
            "tcp".to_string()
        } else {
            "udp".to_string()
        };

    let cipher = extract_openvpn_data_value(settings_map, "cipher");
    let auth = extract_openvpn_data_value(settings_map, "auth");

    let compression = extract_openvpn_data_value(settings_map, "compress")
        .or_else(|| extract_openvpn_data_value(settings_map, "comp-lzo").map(|_| "lzo".into()));

    Some(VpnDetails::OpenVpn {
        remote,
        port,
        protocol,
        cipher,
        auth,
        compression,
    })
}

fn extract_wireguard_details(
    settings_map: &HashMap<String, HashMap<String, zvariant::Value<'_>>>,
) -> Option<VpnDetails> {
    let wg_sec = settings_map.get("wireguard")?;

    let public_key = wg_sec.get("public-key").and_then(|v| match v {
        zvariant::Value::Str(s) => Some(s.to_string()),
        _ => None,
    });

    let endpoint = wg_sec
        .get("peers")
        .and_then(|v| match v {
            zvariant::Value::Str(s) => Some(s.as_str().to_string()),
            _ => None,
        })
        .and_then(|peers| {
            let first = peers.split(',').next()?.trim().to_string();
            for tok in first.split_whitespace() {
                if let Some(rest) = tok.strip_prefix("endpoint=") {
                    return Some(rest.to_string());
                }
            }
            None
        });

    Some(VpnDetails::WireGuard {
        public_key,
        endpoint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn openvpn_settings_with_data(
        data: HashMap<String, String>,
    ) -> HashMap<String, HashMap<String, zvariant::Value<'static>>> {
        let dict = zvariant::Dict::from(data);
        let vpn_sec = HashMap::from([("data".to_string(), zvariant::Value::Dict(dict))]);
        HashMap::from([("vpn".to_string(), vpn_sec)])
    }

    #[test]
    fn openvpn_gateway_extracted_from_vpn_data() {
        let data = HashMap::from([("remote".to_string(), "vpn.example.com:1194".to_string())]);
        let settings = openvpn_settings_with_data(data);
        assert_eq!(
            extract_openvpn_gateway(&settings),
            Some("vpn.example.com:1194".to_string())
        );
    }

    #[test]
    fn openvpn_gateway_none_when_remote_key_absent() {
        let data = HashMap::from([("dev".to_string(), "tun".to_string())]);
        let settings = openvpn_settings_with_data(data);
        assert_eq!(extract_openvpn_gateway(&settings), None);
    }

    #[test]
    fn openvpn_gateway_none_when_vpn_section_absent() {
        let settings: HashMap<String, HashMap<String, zvariant::Value<'static>>> =
            HashMap::from([("connection".to_string(), HashMap::new())]);
        assert_eq!(extract_openvpn_gateway(&settings), None);
    }

    #[test]
    fn openvpn_gateway_none_when_data_key_absent() {
        let vpn_sec = HashMap::from([(
            "service-type".to_string(),
            zvariant::Value::Str("org.freedesktop.NetworkManager.openvpn".into()),
        )]);
        let settings = HashMap::from([("vpn".to_string(), vpn_sec)]);
        assert_eq!(extract_openvpn_gateway(&settings), None);
    }

    #[test]
    fn openvpn_details_full() {
        let data = HashMap::from([
            ("remote".to_string(), "vpn.example.com:1194".to_string()),
            ("proto-tcp".to_string(), "yes".to_string()),
            ("cipher".to_string(), "AES-256-GCM".to_string()),
            ("auth".to_string(), "SHA256".to_string()),
            ("compress".to_string(), "lz4-v2".to_string()),
        ]);
        let settings = openvpn_settings_with_data(data);
        let details = extract_openvpn_details(&settings).unwrap();
        match details {
            VpnDetails::OpenVpn {
                remote,
                port,
                protocol,
                cipher,
                auth,
                compression,
            } => {
                assert_eq!(remote, "vpn.example.com");
                assert_eq!(port, 1194);
                assert_eq!(protocol, "tcp");
                assert_eq!(cipher, Some("AES-256-GCM".into()));
                assert_eq!(auth, Some("SHA256".into()));
                assert_eq!(compression, Some("lz4-v2".into()));
            }
            _ => panic!("expected OpenVpn variant"),
        }
    }

    #[test]
    fn openvpn_details_minimal() {
        let data = HashMap::from([("remote".to_string(), "vpn.example.com:443".to_string())]);
        let settings = openvpn_settings_with_data(data);
        let details = extract_openvpn_details(&settings).unwrap();
        match details {
            VpnDetails::OpenVpn {
                remote,
                port,
                protocol,
                cipher,
                auth,
                compression,
            } => {
                assert_eq!(remote, "vpn.example.com");
                assert_eq!(port, 443);
                assert_eq!(protocol, "udp");
                assert!(cipher.is_none());
                assert!(auth.is_none());
                assert!(compression.is_none());
            }
            _ => panic!("expected OpenVpn variant"),
        }
    }

    #[test]
    fn openvpn_details_none_when_no_remote() {
        let data = HashMap::from([("cipher".to_string(), "AES-256-GCM".to_string())]);
        let settings = openvpn_settings_with_data(data);
        assert!(extract_openvpn_details(&settings).is_none());
    }

    #[test]
    fn openvpn_details_remote_without_port() {
        let data = HashMap::from([("remote".to_string(), "vpn.example.com".to_string())]);
        let settings = openvpn_settings_with_data(data);
        let details = extract_openvpn_details(&settings).unwrap();
        match details {
            VpnDetails::OpenVpn { remote, port, .. } => {
                assert_eq!(remote, "vpn.example.com");
                assert_eq!(port, 1194);
            }
            _ => panic!("expected OpenVpn variant"),
        }
    }

    #[test]
    fn openvpn_details_comp_lzo_fallback() {
        let data = HashMap::from([
            ("remote".to_string(), "vpn.example.com:1194".to_string()),
            ("comp-lzo".to_string(), "yes".to_string()),
        ]);
        let settings = openvpn_settings_with_data(data);
        let details = extract_openvpn_details(&settings).unwrap();
        match details {
            VpnDetails::OpenVpn { compression, .. } => {
                assert_eq!(compression, Some("lzo".into()));
            }
            _ => panic!("expected OpenVpn variant"),
        }
    }

    fn wireguard_settings(
        pairs: Vec<(&str, zvariant::Value<'static>)>,
    ) -> HashMap<String, HashMap<String, zvariant::Value<'static>>> {
        let wg_sec: HashMap<String, zvariant::Value<'static>> =
            pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        HashMap::from([("wireguard".to_string(), wg_sec)])
    }

    #[test]
    fn wireguard_details_full() {
        let settings = wireguard_settings(vec![
            (
                "public-key",
                zvariant::Value::Str("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".into()),
            ),
            (
                "peers",
                zvariant::Value::Str("endpoint=vpn.example.com:51820 allowed-ips=0.0.0.0/0".into()),
            ),
        ]);
        let details = extract_wireguard_details(&settings).unwrap();
        match details {
            VpnDetails::WireGuard {
                public_key,
                endpoint,
            } => {
                assert_eq!(
                    public_key,
                    Some("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".into())
                );
                assert_eq!(endpoint, Some("vpn.example.com:51820".into()));
            }
            _ => panic!("expected WireGuard variant"),
        }
    }

    #[test]
    fn wireguard_details_no_public_key() {
        let settings = wireguard_settings(vec![(
            "peers",
            zvariant::Value::Str("endpoint=vpn.example.com:51820".into()),
        )]);
        let details = extract_wireguard_details(&settings).unwrap();
        match details {
            VpnDetails::WireGuard {
                public_key,
                endpoint,
            } => {
                assert!(public_key.is_none());
                assert_eq!(endpoint, Some("vpn.example.com:51820".into()));
            }
            _ => panic!("expected WireGuard variant"),
        }
    }

    #[test]
    fn wireguard_details_none_when_no_section() {
        let settings: HashMap<String, HashMap<String, zvariant::Value<'static>>> =
            HashMap::from([("connection".to_string(), HashMap::new())]);
        assert!(extract_wireguard_details(&settings).is_none());
    }
}
