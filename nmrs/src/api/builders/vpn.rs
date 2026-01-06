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
//! # Connection Builder API
//!
//! Consider using the fluent builder API added in 1.3.0:
//!
//! ```rust
//! use nmrs::builders::WireGuardBuilder;
//! use nmrs::WireGuardPeer;
//!
//! let peer = WireGuardPeer {
//!     public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".into(),
//!     gateway: "vpn.example.com:51820".into(),
//!     allowed_ips: vec!["0.0.0.0/0".into()],
//!     preshared_key: None,
//!     persistent_keepalive: Some(25),
//! };
//!
//! let settings = WireGuardBuilder::new("MyVPN")
//!     .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
//!     .address("10.0.0.2/24")
//!     .add_peer(peer)
//!     .dns(vec!["1.1.1.1".into()])
//!     .build()
//!     .expect("Failed to build WireGuard connection");
//! ```
//!
//! # Legacy Function API
//!
//! The `build_wireguard_connection` function is maintained for backward compatibility:
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
use zvariant::Value;

use super::wireguard_builder::WireGuardBuilder;
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
///
/// # Note
///
/// This function is maintained for backward compatibility. For new code,
/// consider using `WireGuardBuilder` for a more ergonomic API.
pub fn build_wireguard_connection(
    creds: &VpnCredentials,
    opts: &ConnectionOptions,
) -> Result<HashMap<&'static str, HashMap<&'static str, Value<'static>>>, ConnectionError> {
    let mut builder = WireGuardBuilder::new(&creds.name)
        .private_key(&creds.private_key)
        .address(&creds.address)
        .add_peers(creds.peers.iter().cloned())
        .options(opts);

    if let Some(uuid) = creds.uuid {
        builder = builder.uuid(uuid);
    }

    if let Some(dns) = &creds.dns {
        builder = builder.dns(dns.clone());
    }

    if let Some(mtu) = creds.mtu {
        builder = builder.mtu(mtu);
    }

    builder.build()
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
        assert!(settings.contains_key("wireguard"));
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
        assert_eq!(conn_type, &Value::from("wireguard"));

        let id = connection.get("id").unwrap();
        assert_eq!(id, &Value::from("TestVPN"));
    }

    #[test]
    fn vpn_section_has_wireguard_service_type() {
        let creds = create_test_credentials();
        let opts = create_test_options();

        let settings = build_wireguard_connection(&creds, &opts).unwrap();
        let vpn = settings.get("wireguard").unwrap();

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

    // Gateway validation tests for peer gateways
    // These test that validation is properly delegated to WireGuardBuilder

    #[test]
    fn rejects_peer_with_empty_gateway() {
        let mut creds = create_test_credentials();
        creds.peers[0].gateway = "".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_peer_gateway_without_port() {
        let mut creds = create_test_credentials();
        creds.peers[0].gateway = "vpn.example.com".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_peer_gateway_with_invalid_port() {
        let mut creds = create_test_credentials();
        creds.peers[0].gateway = "vpn.example.com:99999".into();
        let opts = create_test_options();

        let result = build_wireguard_connection(&creds, &opts);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_peer_gateway_with_zero_port() {
        let mut creds = create_test_credentials();
        creds.peers[0].gateway = "vpn.example.com:0".into();
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
