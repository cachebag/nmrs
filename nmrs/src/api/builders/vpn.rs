//! VPN connection settings builders.
//!
//! This module provides functions to build NetworkManager settings dictionaries
//! for VPN connections. Currently supports:
//!
//! - **WireGuard** - Modern, high-performance VPN protocol
//!
//! # Usage
//!
//! Most users should call [`NetworkManager::connect_vpn`][crate::NetworkManager::connect_vpn]
//! instead of using these builders directly. This module is intended for
//! advanced use cases where you need low-level control over the connection settings.
//!
//! # Example
//!
//! ```rust
//! use nmrs::builders::build_wireguard_connection;
//! use nmrs::{VpnCredentials, VpnType, WireGuardPeer, ConnectionOptions};
//!
//! let creds = VpnCredentials {
//!     vpn_type: VpnType::WireGuard,
//!     name: "MyVPN".into(),
//!     gateway: "vpn.example.com:51820".into(),
//!     // Valid WireGuard private key (44 chars base64)
//!     private_key: "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".into(),
//!     address: "10.0.0.2/24".into(),
//!     peers: vec![WireGuardPeer {
//!         // Valid WireGuard public key (44 chars base64)
//!         public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".into(),
//!         gateway: "vpn.example.com:51820".into(),
//!         allowed_ips: vec!["0.0.0.0/0".into()],
//!         preshared_key: None,
//!         persistent_keepalive: Some(25),
//!     }],
//!     dns: Some(vec!["1.1.1.1".into()]),
//!     mtu: None,
//!     uuid: None,
//! };
//!
//! let opts = ConnectionOptions {
//!     autoconnect: false,
//!     autoconnect_priority: None,
//!     autoconnect_retries: None,
//! };
//!
//! let settings = build_wireguard_connection(&creds, &opts).unwrap();
//! // Pass settings to NetworkManager's AddAndActivateConnection
//! ```

use std::collections::HashMap;
use uuid::Uuid;
use zvariant::Value;

use crate::api::models::{ConnectionError, ConnectionOptions, VpnCredentials};

/// Validates a WireGuard key (private or public).
///
/// WireGuard keys are 32-byte values encoded in base64, resulting in 44 characters
/// (including padding).
fn validate_wireguard_key(key: &str, key_type: &str) -> Result<(), ConnectionError> {
    // Basic validation: should be non-empty and reasonable length
    if key.trim().is_empty() {
        return Err(ConnectionError::InvalidPrivateKey(format!(
            "{} cannot be empty",
            key_type
        )));
    }

    // WireGuard keys are 32 bytes, base64 encoded = 44 chars with padding
    // We'll be lenient and allow 43-45 characters
    let len = key.trim().len();
    if !(40..=50).contains(&len) {
        return Err(ConnectionError::InvalidPrivateKey(format!(
            "{} has invalid length: {} (expected ~44 characters)",
            key_type, len
        )));
    }

    // Check if it's valid base64 (contains only base64 characters)
    let is_valid_base64 = key
        .trim()
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');

    if !is_valid_base64 {
        return Err(ConnectionError::InvalidPrivateKey(format!(
            "{} contains invalid base64 characters",
            key_type
        )));
    }

    Ok(())
}

/// Validates an IP address with CIDR notation (e.g., "10.0.0.2/24").
fn validate_address(address: &str) -> Result<(String, u32), ConnectionError> {
    let (ip, prefix) = address.split_once('/').ok_or_else(|| {
        ConnectionError::InvalidAddress(format!(
            "missing CIDR prefix (e.g., '10.0.0.2/24'): {}",
            address
        ))
    })?;

    // Validate IP address format (basic check)
    if ip.trim().is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "IP address cannot be empty".into(),
        ));
    }

    // Parse CIDR prefix
    let prefix: u32 = prefix
        .parse()
        .map_err(|_| ConnectionError::InvalidAddress(format!("invalid CIDR prefix: {}", prefix)))?;

    // Validate prefix range (IPv4: 0-32, IPv6: 0-128)
    // We'll accept up to 128 to support IPv6
    if prefix > 128 {
        return Err(ConnectionError::InvalidAddress(format!(
            "CIDR prefix too large: {} (max 128)",
            prefix
        )));
    }

    // Basic IPv4 validation (if it contains dots)
    if ip.contains('.') {
        let octets: Vec<&str> = ip.split('.').collect();
        if octets.len() != 4 {
            return Err(ConnectionError::InvalidAddress(format!(
                "invalid IPv4 address: {}",
                ip
            )));
        }

        for octet in octets {
            let num: u32 = octet.parse().map_err(|_| {
                ConnectionError::InvalidAddress(format!("invalid IPv4 octet: {}", octet))
            })?;
            if num > 255 {
                return Err(ConnectionError::InvalidAddress(format!(
                    "IPv4 octet out of range: {}",
                    num
                )));
            }
        }

        if prefix > 32 {
            return Err(ConnectionError::InvalidAddress(format!(
                "IPv4 CIDR prefix too large: {} (max 32)",
                prefix
            )));
        }
    }

    Ok((ip.to_string(), prefix))
}

/// Validates a VPN gateway endpoint (should be in "host:port" format).
fn validate_gateway(gateway: &str) -> Result<(), ConnectionError> {
    if gateway.trim().is_empty() {
        return Err(ConnectionError::InvalidGateway(
            "gateway cannot be empty".into(),
        ));
    }

    // Should contain a colon for port
    if !gateway.contains(':') {
        return Err(ConnectionError::InvalidGateway(format!(
            "gateway must be in 'host:port' format: {}",
            gateway
        )));
    }

    let parts: Vec<&str> = gateway.rsplitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(ConnectionError::InvalidGateway(format!(
            "invalid gateway format: {}",
            gateway
        )));
    }

    // Validate port
    let port_str = parts[0];
    let port: u16 = port_str.parse().map_err(|_| {
        ConnectionError::InvalidGateway(format!("invalid port number: {}", port_str))
    })?;

    if port == 0 {
        return Err(ConnectionError::InvalidGateway("port cannot be 0".into()));
    }

    Ok(())
}

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
    // Validate peers
    if creds.peers.is_empty() {
        return Err(ConnectionError::InvalidPeers("No peers provided".into()));
    }

    // Validate private key
    validate_wireguard_key(&creds.private_key, "Private key")?;

    // Validate gateway
    validate_gateway(&creds.gateway)?;

    // Validate address
    let (ip, prefix) = validate_address(&creds.address)?;

    // Validate each peer
    for (i, peer) in creds.peers.iter().enumerate() {
        validate_wireguard_key(&peer.public_key, &format!("Peer {} public key", i))?;
        validate_gateway(&peer.gateway)?;

        if peer.allowed_ips.is_empty() {
            return Err(ConnectionError::InvalidPeers(format!(
                "Peer {} has no allowed IPs",
                i
            )));
        }
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

    // Use already validated address
    let addresses = vec![vec![
        Value::from(ip),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::{VpnType, WireGuardPeer};

    fn create_test_credentials() -> VpnCredentials {
        VpnCredentials {
            vpn_type: VpnType::WireGuard,
            name: "TestVPN".into(),
            gateway: "vpn.example.com:51820".into(),
            private_key: "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".into(),
            address: "10.0.0.2/24".into(),
            peers: vec![WireGuardPeer {
                public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".into(),
                gateway: "vpn.example.com:51820".into(),
                allowed_ips: vec!["0.0.0.0/0".into()],
                preshared_key: None,
                persistent_keepalive: Some(25),
            }],
            dns: Some(vec!["1.1.1.1".into(), "8.8.8.8".into()]),
            mtu: Some(1420),
            uuid: None,
        }
    }

    fn create_test_options() -> ConnectionOptions {
        ConnectionOptions {
            autoconnect: false,
            autoconnect_priority: None,
            autoconnect_retries: None,
        }
    }

    #[test]
    fn builds_wireguard_connection() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts);
        assert!(settings.is_ok());

        let settings = settings.unwrap();
        assert!(settings.contains_key("connection"));
        assert!(settings.contains_key("vpn"));
        assert!(settings.contains_key("ipv4"));
        assert!(settings.contains_key("ipv6"));
    }

    #[test]
    fn connection_section_has_correct_type() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let connection = settings.get("connection").unwrap();

        let conn_type = connection.get("type").unwrap();
        assert_eq!(conn_type, &Value::from("vpn"));

        let id = connection.get("id").unwrap();
        assert_eq!(id, &Value::from("TestVPN"));
    }

    #[test]
    fn vpn_section_has_wireguard_service_type() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let vpn = settings.get("vpn").unwrap();

        let service_type = vpn.get("service-type").unwrap();
        assert_eq!(
            service_type,
            &Value::from("org.freedesktop.NetworkManager.wireguard")
        );
    }

    #[test]
    fn ipv4_section_is_manual() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let ipv4 = settings.get("ipv4").unwrap();

        let method = ipv4.get("method").unwrap();
        assert_eq!(method, &Value::from("manual"));
    }

    #[test]
    fn ipv6_section_is_ignored() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let ipv6 = settings.get("ipv6").unwrap();

        let method = ipv6.get("method").unwrap();
        assert_eq!(method, &Value::from("ignore"));
    }

    #[test]
    fn rejects_empty_peers() {
        let mut creds = create_test_credentials();
        creds.peers = vec![];
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidPeers(_)
        ));
    }

    #[test]
    fn rejects_invalid_address_format() {
        let mut creds = create_test_credentials();
        creds.address = "invalid".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidAddress(_)
        ));
    }

    #[test]
    fn rejects_address_without_cidr() {
        let mut creds = create_test_credentials();
        creds.address = "10.0.0.2".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidAddress(_)
        ));
    }

    #[test]
    fn accepts_ipv6_address() {
        let mut creds = create_test_credentials();
        creds.address = "fd00::2/64".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn handles_multiple_peers() {
        let mut creds = create_test_credentials();
        creds.peers.push(WireGuardPeer {
            public_key: "xScVkH3fUGUVRvGLFcjkx+GGD7cf5eBVyN3Gh4FLjmI=".into(),
            gateway: "peer2.example.com:51821".into(),
            allowed_ips: vec!["192.168.0.0/16".into()],
            preshared_key: Some("PSKABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklm=".into()),
            persistent_keepalive: None,
        });
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn handles_optional_dns() {
        let mut creds = create_test_credentials();
        creds.dns = None;
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn handles_optional_mtu() {
        let mut creds = create_test_credentials();
        creds.mtu = None;
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn includes_dns_when_provided() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let ipv4 = settings.get("ipv4").unwrap();

        assert!(ipv4.contains_key("dns"));
    }

    #[test]
    fn includes_mtu_when_provided() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let ipv4 = settings.get("ipv4").unwrap();

        assert!(ipv4.contains_key("mtu"));
    }

    #[test]
    fn respects_autoconnect_option() {
        let creds = create_test_credentials();
        let mut opts = create_test_options();
        opts.autoconnect = true;

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let connection = settings.get("connection").unwrap();

        let autoconnect = connection.get("autoconnect").unwrap();
        assert_eq!(autoconnect, &Value::from(true));
    }

    #[test]
    fn includes_autoconnect_priority_when_provided() {
        let creds = create_test_credentials();
        let mut opts = create_test_options();
        opts.autoconnect_priority = Some(10);

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let connection = settings.get("connection").unwrap();

        assert!(connection.contains_key("autoconnect-priority"));
    }

    #[test]
    fn generates_uuid_when_not_provided() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let connection = settings.get("connection").unwrap();

        assert!(connection.contains_key("uuid"));
    }

    #[test]
    fn uses_provided_uuid() {
        let mut creds = create_test_credentials();
        let test_uuid = uuid::Uuid::new_v4();
        creds.uuid = Some(test_uuid);
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let connection = settings.get("connection").unwrap();

        let uuid = connection.get("uuid").unwrap();
        assert_eq!(uuid, &Value::from(test_uuid.to_string()));
    }

    #[test]
    fn peer_with_preshared_key() {
        let mut creds = create_test_credentials();
        creds.peers[0].preshared_key = Some("PSKABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklm=".into());
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn peer_without_keepalive() {
        let mut creds = create_test_credentials();
        creds.peers[0].persistent_keepalive = None;
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn multiple_allowed_ips_for_peer() {
        let mut creds = create_test_credentials();
        creds.peers[0].allowed_ips =
            vec!["0.0.0.0/0".into(), "::/0".into(), "192.168.1.0/24".into()];
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_ok());
    }

    // Validation tests

    #[test]
    fn rejects_empty_private_key() {
        let mut creds = create_test_credentials();
        creds.private_key = "".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidPrivateKey(_)
        ));
    }

    #[test]
    fn rejects_short_private_key() {
        let mut creds = create_test_credentials();
        creds.private_key = "tooshort".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidPrivateKey(_)
        ));
    }

    #[test]
    fn rejects_invalid_private_key_characters() {
        let mut creds = create_test_credentials();
        creds.private_key = "this is not base64 encoded!!!!!!!!!!!!!!!!!!".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidPrivateKey(_)
        ));
    }

    #[test]
    fn rejects_empty_gateway() {
        let mut creds = create_test_credentials();
        creds.gateway = "".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_gateway_without_port() {
        let mut creds = create_test_credentials();
        creds.gateway = "vpn.example.com".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_gateway_with_invalid_port() {
        let mut creds = create_test_credentials();
        creds.gateway = "vpn.example.com:99999".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_gateway_with_zero_port() {
        let mut creds = create_test_credentials();
        creds.gateway = "vpn.example.com:0".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_invalid_ipv4_address() {
        let mut creds = create_test_credentials();
        creds.address = "999.999.999.999/24".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidAddress(_)
        ));
    }

    #[test]
    fn rejects_ipv4_with_invalid_prefix() {
        let mut creds = create_test_credentials();
        creds.address = "10.0.0.2/999".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidAddress(_)
        ));
    }

    #[test]
    fn rejects_peer_with_empty_allowed_ips() {
        let mut creds = create_test_credentials();
        creds.peers[0].allowed_ips = vec![];
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidPeers(_)
        ));
    }

    #[test]
    fn rejects_peer_with_invalid_public_key() {
        let mut creds = create_test_credentials();
        creds.peers[0].public_key = "invalid!@#$key".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        // Should get InvalidPrivateKey error (we use same validation for both)
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidPrivateKey(_)
        ));
    }

    #[test]
    fn accepts_valid_ipv4_addresses() {
        let test_cases = vec![
            "10.0.0.2/24",
            "192.168.1.100/32",
            "172.16.0.1/16",
            "1.1.1.1/8",
        ];

        for address in test_cases {
            let mut creds = create_test_credentials();
            creds.address = address.into();
            let opts = create_test_options();

            let result = build_wireguard_connection(&creds, &opts);
            assert!(
                result.is_ok(),
                "Should accept valid IPv4 address: {}",
                address
            );
        }
    }

    #[test]
    fn accepts_standard_wireguard_ports() {
        let test_cases = vec![
            "vpn.example.com:51820",
            "192.168.1.1:51821",
            "test.local:12345",
        ];

        for gateway in test_cases {
            let mut creds = create_test_credentials();
            creds.gateway = gateway.into();
            let opts = create_test_options();

            let result = build_wireguard_connection(&creds, &opts);
            assert!(result.is_ok(), "Should accept valid gateway: {}", gateway);
        }
    }
}
