//! VPN connection settings builders.
//!
//! Currently supports building settings for WireGuard VPN connections.
//! The resulting dictionary can be passed directly to
//! `AddAndActivateConnection` on the NetworkManager D-Bus API.
//!
//! Most users should call [`NetworkManager::connect_vpn`][crate::NetworkManager::connect_vpn]
//! instead of using these builders directly. This module is intended for
//! advanced use cases where you need low-level control over the settings.

use std::collections::HashMap;
use uuid::Uuid;
use zvariant::Value;

use crate::api::models::{ConnectionError, ConnectionOptions, VpnCredentials};

/// Builds WireGuard VPN connection settings.
///
/// Returns a complete NetworkManager settings dictionary suitable for
/// `AddAndActivateConnection`.
///
/// # Errors
///
/// - `ConnectionError::InvalidPeers` if no peers are provided
/// - `ConnectionError::InvalidAddress` if the address is missing or malformed
pub fn build_wireguard_connection(
    creds: &VpnCredentials,
    opts: &ConnectionOptions,
) -> Result<HashMap<&'static str, HashMap<&'static str, Value<'static>>>, ConnectionError> {
    if creds.peers.is_empty() {
        return Err(ConnectionError::InvalidPeers("No peers provided".into()));
    }

    let mut conn = HashMap::new();

    // [connection] section
    let mut connection = HashMap::new();
    connection.insert("type", Value::from("vpn"));
    connection.insert("id", Value::from(creds.name.clone()));

    let uuid = creds.uuid.unwrap_or_else(|| {
        Uuid::new_v5(
            &Uuid::NAMESPACE_DNS,
            format!("wg:{}@{}", creds.name, creds.gateway).as_bytes(),
        )
    });
    connection.insert("uuid", Value::from(uuid.to_string()));
    connection.insert("autoconnect", Value::from(opts.autoconnect));

    if let Some(p) = opts.autoconnect_priority {
        connection.insert("autoconnect-priority", Value::from(p));
    }
    if let Some(r) = opts.autoconnect_retries {
        connection.insert("autoconnect-retries", Value::from(r));
    }

    conn.insert("connection", connection);

    // [vpn] section
    let mut vpn = HashMap::new();
    vpn.insert(
        "service-type",
        Value::from("org.freedesktop.NetworkManager.wireguard"),
    );

    // WireGuard-specific data
    let mut data: HashMap<String, Value<'static>> = HashMap::new();
    data.insert("private-key".into(), Value::from(creds.private_key.clone()));

    for (i, peer) in creds.peers.iter().enumerate() {
        let prefix = format!("peer.{i}.");
        data.insert(
            format!("{prefix}public-key"),
            Value::from(peer.public_key.clone()),
        );
        data.insert(
            format!("{prefix}endpoint"),
            Value::from(peer.gateway.clone()),
        );
        data.insert(
            format!("{prefix}allowed-ips"),
            Value::from(peer.allowed_ips.join(",")),
        );

        if let Some(psk) = &peer.preshared_key {
            data.insert(format!("{prefix}preshared-key"), Value::from(psk.clone()));
        }

        if let Some(ka) = peer.persistent_keepalive {
            data.insert(format!("{prefix}persistent-keepalive"), Value::from(ka));
        }
    }

    vpn.insert("data", Value::from(data));
    conn.insert("vpn", vpn);

    // [ipv4] section
    let mut ipv4 = HashMap::new();
    ipv4.insert("method", Value::from("manual"));

    // Parse address (example: "10.0.0.2/24")
    let (ip, prefix) = creds
        .address
        .split_once('/')
        .ok_or_else(|| ConnectionError::InvalidAddress("missing address".into()))?;

    let prefix: u32 = prefix
        .parse()
        .map_err(|_| ConnectionError::InvalidAddress("invalid address".into()))?;

    let addresses = vec![vec![
        Value::from(ip.to_string()),
        Value::from(prefix),
        Value::from("0.0.0.0"),
    ]];
    ipv4.insert("address-data", Value::from(addresses));

    if let Some(dns) = &creds.dns {
        let dns_vec: Vec<String> = dns.to_vec();
        ipv4.insert("dns", Value::from(dns_vec));
    }

    if let Some(mtu) = creds.mtu {
        ipv4.insert("mtu", Value::from(mtu));
    }

    conn.insert("ipv4", ipv4);

    // [ipv6] section (required but typically ignored for WireGuard)
    let mut ipv6 = HashMap::new();
    ipv6.insert("method", Value::from("ignore"));
    conn.insert("ipv6", ipv6);

    Ok(conn)
}
