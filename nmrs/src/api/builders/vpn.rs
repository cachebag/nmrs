//! VPN connection settings builders.
//!
//! This module provides functions to build NetworkManager settings dictionaries
//! for VPN connections. Supports:
//!
//! - **WireGuard** — Modern, high-performance VPN protocol
//! - **OpenVPN** — Widely-used open-source VPN protocol (via NM plugin)
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
//! let peer = WireGuardPeer::new(
//!     "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
//!     "vpn.example.com:51820",
//!     vec!["0.0.0.0/0".into()],
//! ).with_persistent_keepalive(25);
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
//! use nmrs::{VpnCredentials, VpnKind, WireGuardPeer, ConnectionOptions};
//!
//! let peer = WireGuardPeer::new(
//!     "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
//!     "vpn.example.com:51820",
//!     vec!["0.0.0.0/0".into()],
//! ).with_persistent_keepalive(25);
//!
//! let creds = VpnCredentials::new(
//!     VpnKind::WireGuard,
//!     "MyVPN",
//!     "vpn.example.com:51820",
//!     "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=",
//!     "10.0.0.2/24",
//!     vec![peer],
//! ).with_dns(vec!["1.1.1.1".into()]);
//!
//! let opts = ConnectionOptions::new(false);
//!
//! let settings = build_wireguard_connection(&creds, &opts).unwrap();
//! // Pass settings to NetworkManager's AddAndActivateConnection
//! ```
#![allow(deprecated)]

use std::collections::HashMap;
use zvariant::{Dict, Value, signature};

use super::wireguard_builder::WireGuardBuilder;
use crate::api::models::{
    ConnectionError, ConnectionOptions, OpenVpnAuthType, OpenVpnCompression, OpenVpnConfig,
    OpenVpnProxy, VpnCredentials,
};

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

/// Converts a list of string key-value pairs into a `zvariant::Dict` with
/// D-Bus signature `a{ss}`, which NetworkManager requires for `vpn.data`
/// and `vpn.secrets`.
fn string_pairs_to_dict(
    pairs: Vec<(String, String)>,
) -> Result<Dict<'static, 'static>, ConnectionError> {
    let sig = signature!("s");
    let mut dict = Dict::new(&sig, &sig);
    for (k, v) in pairs {
        dict.append(Value::from(k), Value::from(v)).map_err(|e| {
            ConnectionError::VpnFailed(format!("failed to append VPN setting: {e}"))
        })?;
    }
    Ok(dict)
}

/// Builds OpenVPN connection settings for NetworkManager.
///
/// Returns a settings dictionary suitable for `AddAndActivateConnection`.
/// OpenVPN uses the NM VPN plugin model: `connection.type = "vpn"` with
/// `vpn.service-type = "org.freedesktop.NetworkManager.openvpn"`.
/// All config lives in the flat `vpn.data` dict.
///
/// # Errors
///
/// - `ConnectionError::InvalidGateway` if `remote` is empty
/// - `ConnectionError::InvalidAddress` if a proxy port is zero
pub fn build_openvpn_connection(
    config: &OpenVpnConfig,
    opts: &ConnectionOptions,
) -> Result<HashMap<&'static str, HashMap<&'static str, Value<'static>>>, ConnectionError> {
    if config.remote.is_empty() {
        return Err(ConnectionError::InvalidGateway(
            "OpenVPN remote must not be empty".into(),
        ));
    }

    let uuid = config.uuid.unwrap_or_else(uuid::Uuid::new_v4).to_string();

    let mut connection: HashMap<&'static str, Value<'static>> = HashMap::new();
    connection.insert("type", Value::from("vpn"));
    connection.insert("id", Value::from(config.name.clone()));
    connection.insert("uuid", Value::from(uuid));
    connection.insert("autoconnect", Value::from(opts.autoconnect));
    if let Some(p) = opts.autoconnect_priority {
        connection.insert("autoconnect-priority", Value::from(p));
    }

    let mut vpn_data: Vec<(String, String)> = Vec::new();

    let remote = format!("{}:{}", config.remote, config.port);

    vpn_data.push(("remote".into(), remote));

    let connection_type = match config.auth_type {
        Some(OpenVpnAuthType::Password) => "password",
        Some(OpenVpnAuthType::Tls) => "tls",
        Some(OpenVpnAuthType::PasswordTls) => "password-tls",
        Some(OpenVpnAuthType::StaticKey) => "static-key",
        None => "tls",
    };
    vpn_data.push(("connection-type".into(), connection_type.into()));

    if config.tcp {
        vpn_data.push(("proto-tcp".into(), "yes".into()));
    }

    if let Some(ref username) = config.username {
        vpn_data.push(("username".into(), username.clone()));
    }
    if let Some(ref auth) = config.auth {
        vpn_data.push(("auth".into(), auth.clone()));
    }
    if let Some(ref cipher) = config.cipher {
        vpn_data.push(("cipher".into(), cipher.clone()));
    }
    if let Some(mtu) = config.mtu {
        vpn_data.push(("tunnel-mtu".into(), mtu.to_string()));
    }

    // certs
    if let Some(ref ca) = config.ca_cert {
        vpn_data.push(("ca".into(), ca.clone()));
    }
    if let Some(ref cert) = config.client_cert {
        vpn_data.push(("cert".into(), cert.clone()));
    }
    if let Some(ref key) = config.client_key {
        vpn_data.push(("key".into(), key.clone()));
    }

    if let Some(ref compression) = config.compression {
        #[allow(deprecated)]
        match compression {
            OpenVpnCompression::No => {
                vpn_data.push(("compress".into(), "no".into()));
            }
            OpenVpnCompression::Lzo => {
                vpn_data.push(("comp-lzo".into(), "yes".into()));
            }
            OpenVpnCompression::Lz4 => {
                vpn_data.push(("compress".into(), "lz4".into()));
            }
            OpenVpnCompression::Lz4V2 => {
                vpn_data.push(("compress".into(), "lz4-v2".into()));
            }
            OpenVpnCompression::Yes => {
                vpn_data.push(("compress".into(), "yes".into()));
            }
        }
    }

    // TLS hardening options
    if let Some(ref key) = config.tls_auth_key {
        vpn_data.push(("tls-auth".into(), key.clone()));
        if let Some(dir) = config.tls_auth_direction {
            vpn_data.push(("ta-dir".into(), dir.to_string()));
        }
    }
    // FIXME: surely, there must be a better way to do this
    if let Some(ref key) = config.tls_crypt {
        vpn_data.push(("tls-crypt".into(), key.clone()));
    }
    if let Some(ref key) = config.tls_crypt_v2 {
        vpn_data.push(("tls-crypt-v2".into(), key.clone()));
    }
    if let Some(ref ver) = config.tls_version_min {
        vpn_data.push(("tls-version-min".into(), ver.clone()));
    }
    if let Some(ref ver) = config.tls_version_max {
        vpn_data.push(("tls-version-max".into(), ver.clone()));
    }
    if let Some(ref cipher) = config.tls_cipher {
        vpn_data.push(("tls-cipher".into(), cipher.clone()));
    }
    if let Some(ref cert_type) = config.remote_cert_tls {
        vpn_data.push(("remote-cert-tls".into(), cert_type.clone()));
    }
    if let Some((ref name, ref name_type)) = config.verify_x509_name {
        vpn_data.push(("verify-x509-name".into(), name.clone()));
        vpn_data.push(("verify-x509-type".into(), name_type.clone()));
    }
    if let Some(ref path) = config.crl_verify {
        vpn_data.push(("crl-verify".into(), path.clone()));
    }

    if let Some(v) = config.ping {
        vpn_data.push(("ping".into(), v.to_string()));
    }
    if let Some(v) = config.ping_exit {
        vpn_data.push(("ping-exit".into(), v.to_string()));
    }
    if let Some(v) = config.ping_restart {
        vpn_data.push(("ping-restart".into(), v.to_string()));
    }
    if let Some(v) = config.reneg_seconds {
        vpn_data.push(("reneg-sec".into(), v.to_string()));
    }
    if let Some(v) = config.connect_timeout {
        vpn_data.push(("connect-timeout".into(), v.to_string()));
    }
    if let Some(ref s) = config.data_ciphers {
        vpn_data.push(("data-ciphers".into(), s.clone()));
    }
    if let Some(ref s) = config.data_ciphers_fallback {
        vpn_data.push(("data-ciphers-fallback".into(), s.clone()));
    }
    if config.ncp_disable {
        vpn_data.push(("ncp-disable".into(), "yes".into()));
    }
    // holy moly

    if let Some(ref proxy) = config.proxy {
        match proxy {
            OpenVpnProxy::Http {
                server,
                port,
                username,
                password,
                retry,
            } => {
                if *port == 0 {
                    return Err(ConnectionError::InvalidAddress(
                        "proxy port must not be zero".into(),
                    ));
                }
                vpn_data.push(("proxy-type".into(), "http".into()));
                vpn_data.push(("proxy-server".into(), server.clone()));
                vpn_data.push(("proxy-port".into(), port.to_string()));
                vpn_data.push((
                    "proxy-retry".into(),
                    if *retry { "yes" } else { "no" }.into(),
                ));
                if let Some(u) = username {
                    vpn_data.push(("http-proxy-username".into(), u.clone()));
                }
                if let Some(p) = password {
                    vpn_data.push(("http-proxy-password".into(), p.clone()));
                }
            }
            OpenVpnProxy::Socks {
                server,
                port,
                retry,
            } => {
                if *port == 0 {
                    return Err(ConnectionError::InvalidAddress(
                        "proxy port must not be zero".into(),
                    ));
                }
                vpn_data.push(("proxy-type".into(), "socks".into()));
                vpn_data.push(("proxy-server".into(), server.clone()));
                vpn_data.push(("proxy-port".into(), port.to_string()));
                vpn_data.push((
                    "proxy-retry".into(),
                    if *retry { "yes" } else { "no" }.into(),
                ));
            }
        }
    }

    let data_dict = string_pairs_to_dict(vpn_data)?;

    let mut vpn_secrets: Vec<(String, String)> = Vec::new();
    if let Some(ref password) = config.password {
        vpn_secrets.push(("password".into(), password.clone()));
    }
    if let Some(ref key_password) = config.key_password {
        vpn_secrets.push(("cert-pass".into(), key_password.clone()));
    }

    let mut vpn: HashMap<&'static str, Value<'static>> = HashMap::new();
    vpn.insert(
        "service-type",
        Value::from("org.freedesktop.NetworkManager.openvpn"),
    );
    vpn.insert("data", Value::from(data_dict));
    if !vpn_secrets.is_empty() {
        vpn.insert("secrets", Value::from(string_pairs_to_dict(vpn_secrets)?));
    }

    let mut ipv4: HashMap<&'static str, Value<'static>> = HashMap::new();
    ipv4.insert("method", Value::from("auto"));
    if config.redirect_gateway {
        ipv4.insert("never-default", Value::from(false));
    }
    if !config.routes.is_empty() {
        let route_data: Vec<HashMap<String, Value<'static>>> = config
            .routes
            .iter()
            .map(|route| {
                let mut route_dict = HashMap::new();
                route_dict.insert("dest".to_string(), Value::from(route.dest.clone()));
                route_dict.insert("prefix".to_string(), Value::from(route.prefix));
                if let Some(ref nh) = route.next_hop {
                    route_dict.insert("next-hop".to_string(), Value::from(nh.clone()));
                }
                if let Some(m) = route.metric {
                    route_dict.insert("metric".to_string(), Value::from(m));
                }
                route_dict
            })
            .collect();
        ipv4.insert("route-data", Value::from(route_data));
    }
    if let Some(dns) = &config.dns {
        let dns_array: Vec<Value> = dns.iter().map(|s| Value::from(s.clone())).collect();
        ipv4.insert("dns", Value::from(dns_array));
    }

    let mut ipv6: HashMap<&'static str, Value<'static>> = HashMap::new();
    ipv6.insert("method", Value::from("ignore"));

    let mut settings = HashMap::new();
    settings.insert("connection", connection);
    settings.insert("vpn", vpn);
    settings.insert("ipv4", ipv4);
    settings.insert("ipv6", ipv6);

    Ok(settings)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::{
        OpenVpnCompression, OpenVpnConfig, OpenVpnProxy, VpnKind, WireGuardPeer,
    };

    fn create_test_credentials() -> VpnCredentials {
        let peer = WireGuardPeer::new(
            "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
            "vpn.example.com:51820",
            vec!["0.0.0.0/0".into()],
        )
        .with_persistent_keepalive(25);

        VpnCredentials::new(
            VpnKind::WireGuard,
            "TestVPN",
            "vpn.example.com:51820",
            "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=",
            "10.0.0.2/24",
            vec![peer],
        )
        .with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()])
        .with_mtu(1420)
    }

    fn create_test_options() -> ConnectionOptions {
        ConnectionOptions::new(false)
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
        let extra_peer = WireGuardPeer::new(
            "xScVkH3fUGUVRvGLFcjkx+GGD7cf5eBVyN3Gh4FLjmI=",
            "peer2.example.com:51821",
            vec!["192.168.0.0/16".into()],
        )
        .with_preshared_key("PSKABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklm=");

        creds.peers.push(extra_peer);
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

    // --- OpenVPN tests ---
    fn create_openvpn_config() -> OpenVpnConfig {
        OpenVpnConfig::new("TestOpenVPN", "vpn.example.com", 1194, false)
            .with_ca_cert("/etc/openvpn/ca.crt")
            .with_client_cert("/etc/openvpn/client.crt")
            .with_client_key("/etc/openvpn/client.key")
    }

    #[test]
    fn builds_openvpn_connection() {
        let config = create_openvpn_config();
        let opts = create_test_options();
        let result = build_openvpn_connection(&config, &opts);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert!(settings.contains_key("connection"));
        assert!(settings.contains_key("vpn"));
        assert!(settings.contains_key("ipv4"));
        assert!(settings.contains_key("ipv6"));
    }

    #[test]
    fn openvpn_connection_type_is_vpn() {
        let config = create_openvpn_config();
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let conn = settings.get("connection").unwrap();
        assert_eq!(conn.get("type").unwrap(), &Value::from("vpn"));
    }

    #[test]
    fn openvpn_service_type_is_correct() {
        let config = create_openvpn_config();
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let vpn = settings.get("vpn").unwrap();
        assert_eq!(
            vpn.get("service-type").unwrap(),
            &Value::from("org.freedesktop.NetworkManager.openvpn")
        );
    }

    #[test]
    fn openvpn_rejects_empty_remote() {
        let mut config = create_openvpn_config();
        config.remote = "".into();
        let opts = create_test_options();
        let result = build_openvpn_connection(&config, &opts);
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn openvpn_compression_no() {
        let config = create_openvpn_config().with_compression(OpenVpnCompression::No);
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let vpn = settings.get("vpn").unwrap();
        // vpn.data is packed — just assert the section exists and no error
        assert!(vpn.contains_key("data"));
    }
    #[allow(deprecated)]
    #[test]
    fn openvpn_compression_lzo() {
        let config = create_openvpn_config().with_compression(OpenVpnCompression::Lzo);
        let opts = create_test_options();
        assert!(build_openvpn_connection(&config, &opts).is_ok());
    }

    #[test]
    fn openvpn_compression_lz4() {
        let config = create_openvpn_config().with_compression(OpenVpnCompression::Lz4);
        let opts = create_test_options();
        assert!(build_openvpn_connection(&config, &opts).is_ok());
    }

    #[test]
    fn openvpn_compression_lz4v2() {
        let config = create_openvpn_config().with_compression(OpenVpnCompression::Lz4V2);
        let opts = create_test_options();
        assert!(build_openvpn_connection(&config, &opts).is_ok());
    }

    #[test]
    fn openvpn_compression_yes() {
        let config = create_openvpn_config().with_compression(OpenVpnCompression::Yes);
        let opts = create_test_options();
        assert!(build_openvpn_connection(&config, &opts).is_ok());
    }

    #[test]
    fn openvpn_http_proxy() {
        let config = create_openvpn_config().with_proxy(OpenVpnProxy::Http {
            server: "proxy.example.com".into(),
            port: 8080,
            username: Some("user".into()),
            password: Some("pass".into()),
            retry: true,
        });
        let opts = create_test_options();
        assert!(build_openvpn_connection(&config, &opts).is_ok());
    }

    #[test]
    fn openvpn_http_proxy_no_credentials() {
        let config = create_openvpn_config().with_proxy(OpenVpnProxy::Http {
            server: "proxy.example.com".into(),
            port: 3128,
            username: None,
            password: None,
            retry: false,
        });
        let opts = create_test_options();
        assert!(build_openvpn_connection(&config, &opts).is_ok());
    }

    #[test]
    fn openvpn_socks_proxy() {
        let config = create_openvpn_config().with_proxy(OpenVpnProxy::Socks {
            server: "socks.example.com".into(),
            port: 1080,
            retry: false,
        });
        let opts = create_test_options();
        assert!(build_openvpn_connection(&config, &opts).is_ok());
    }

    #[test]
    fn openvpn_proxy_rejects_zero_port_http() {
        let config = create_openvpn_config().with_proxy(OpenVpnProxy::Http {
            server: "proxy.example.com".into(),
            port: 0,
            username: None,
            password: None,
            retry: false,
        });
        let opts = create_test_options();
        assert!(matches!(
            build_openvpn_connection(&config, &opts).unwrap_err(),
            ConnectionError::InvalidAddress(_)
        ));
    }

    #[test]
    fn openvpn_proxy_rejects_zero_port_socks() {
        let config = create_openvpn_config().with_proxy(OpenVpnProxy::Socks {
            server: "socks.example.com".into(),
            port: 0,
            retry: false,
        });
        let opts = create_test_options();
        assert!(matches!(
            build_openvpn_connection(&config, &opts).unwrap_err(),
            ConnectionError::InvalidAddress(_)
        ));
    }

    #[test]
    fn openvpn_with_dns() {
        let config = create_openvpn_config().with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()]);
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let ipv4 = settings.get("ipv4").unwrap();
        assert!(ipv4.contains_key("dns"));
    }

    #[test]
    fn openvpn_tcp_emits_proto_tcp() {
        let config = OpenVpnConfig::new("TcpVPN", "vpn.example.com", 443, true);
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let vpn = settings.get("vpn").unwrap();
        assert!(vpn.contains_key("data"));
    }

    #[test]
    fn openvpn_vpn_data_has_dict_signature() {
        let config = create_openvpn_config();
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let vpn = settings.get("vpn").unwrap();
        let data = vpn.get("data").unwrap();
        assert_eq!(
            data.value_signature().to_string(),
            "a{ss}",
            "vpn.data must be a{{ss}} for NetworkManager"
        );
    }

    fn get_vpn_data_value(
        settings: &HashMap<&str, HashMap<&str, Value>>,
        key: &str,
    ) -> Option<String> {
        let vpn = settings.get("vpn")?;
        let data = vpn.get("data")?;
        if let Value::Dict(dict) = data {
            let val: String = dict.get::<Value, String>(&Value::from(key)).ok()??;
            return Some(val);
        }
        None
    }

    #[test]
    fn openvpn_vpn_secrets_has_dict_signature() {
        let config = create_openvpn_config()
            .with_auth_type(OpenVpnAuthType::Password)
            .with_username("user")
            .with_password("secret");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let vpn = settings.get("vpn").unwrap();
        let secrets = vpn.get("secrets").unwrap();
        assert_eq!(
            secrets.value_signature().to_string(),
            "a{ss}",
            "vpn.secrets must be a{{ss}} for NetworkManager"
        );
    }

    #[test]
    fn openvpn_tls_auth_key_and_direction() {
        let config = create_openvpn_config().with_tls_auth("/etc/openvpn/ta.key", Some(1));
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "tls-auth").as_deref(),
            Some("/etc/openvpn/ta.key")
        );
        assert_eq!(
            get_vpn_data_value(&settings, "ta-dir").as_deref(),
            Some("1")
        );
    }

    #[test]
    fn openvpn_tls_auth_key_without_direction() {
        let config = create_openvpn_config().with_tls_auth("/etc/openvpn/ta.key", None);
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "tls-auth").as_deref(),
            Some("/etc/openvpn/ta.key")
        );
        assert!(get_vpn_data_value(&settings, "ta-dir").is_none());
    }

    #[test]
    fn openvpn_tls_crypt() {
        let config = create_openvpn_config().with_tls_crypt("/etc/openvpn/tls-crypt.key");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "tls-crypt").as_deref(),
            Some("/etc/openvpn/tls-crypt.key")
        );
    }

    #[test]
    fn openvpn_tls_crypt_v2() {
        let config = create_openvpn_config().with_tls_crypt_v2("/etc/openvpn/tls-crypt-v2.key");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "tls-crypt-v2").as_deref(),
            Some("/etc/openvpn/tls-crypt-v2.key")
        );
    }

    #[test]
    fn openvpn_tls_version_min() {
        let config = create_openvpn_config().with_tls_version_min("1.2");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "tls-version-min").as_deref(),
            Some("1.2")
        );
    }

    #[test]
    fn openvpn_tls_version_max() {
        let config = create_openvpn_config().with_tls_version_max("1.3");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "tls-version-max").as_deref(),
            Some("1.3")
        );
    }

    #[test]
    fn openvpn_tls_cipher() {
        let config =
            create_openvpn_config().with_tls_cipher("TLS-ECDHE-RSA-WITH-AES-256-GCM-SHA384");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "tls-cipher").as_deref(),
            Some("TLS-ECDHE-RSA-WITH-AES-256-GCM-SHA384")
        );
    }

    #[test]
    fn openvpn_remote_cert_tls() {
        let config = create_openvpn_config().with_remote_cert_tls("server");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "remote-cert-tls").as_deref(),
            Some("server")
        );
    }

    #[test]
    fn openvpn_verify_x509_name() {
        let config = create_openvpn_config().with_verify_x509_name("vpn.example.com", "name");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "verify-x509-name").as_deref(),
            Some("vpn.example.com")
        );
        assert_eq!(
            get_vpn_data_value(&settings, "verify-x509-type").as_deref(),
            Some("name")
        );
    }

    #[test]
    fn openvpn_crl_verify() {
        let config = create_openvpn_config().with_crl_verify("/etc/openvpn/crl.pem");
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "crl-verify").as_deref(),
            Some("/etc/openvpn/crl.pem")
        );
    }

    #[test]
    fn openvpn_tls_options_absent_by_default() {
        let config = create_openvpn_config();
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert!(get_vpn_data_value(&settings, "tls-auth").is_none());
        assert!(get_vpn_data_value(&settings, "ta-dir").is_none());
        assert!(get_vpn_data_value(&settings, "tls-crypt").is_none());
        assert!(get_vpn_data_value(&settings, "tls-crypt-v2").is_none());
        assert!(get_vpn_data_value(&settings, "tls-version-min").is_none());
        assert!(get_vpn_data_value(&settings, "tls-version-max").is_none());
        assert!(get_vpn_data_value(&settings, "tls-cipher").is_none());
        assert!(get_vpn_data_value(&settings, "remote-cert-tls").is_none());
        assert!(get_vpn_data_value(&settings, "verify-x509-name").is_none());
        assert!(get_vpn_data_value(&settings, "crl-verify").is_none());
    }

    #[test]
    fn openvpn_resilience_keys_in_vpn_data() {
        let config = create_openvpn_config()
            .with_ping(10)
            .with_ping_exit(60)
            .with_ping_restart(120)
            .with_reneg_seconds(3600)
            .with_connect_timeout(30);
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(get_vpn_data_value(&settings, "ping").as_deref(), Some("10"));
        assert_eq!(
            get_vpn_data_value(&settings, "ping-exit").as_deref(),
            Some("60")
        );
        assert_eq!(
            get_vpn_data_value(&settings, "ping-restart").as_deref(),
            Some("120")
        );
        assert_eq!(
            get_vpn_data_value(&settings, "reneg-sec").as_deref(),
            Some("3600")
        );
        assert_eq!(
            get_vpn_data_value(&settings, "connect-timeout").as_deref(),
            Some("30")
        );
    }

    #[test]
    fn openvpn_data_ciphers_and_ncp_disable() {
        let config = create_openvpn_config()
            .with_data_ciphers("AES-256-GCM:AES-128-GCM")
            .with_data_ciphers_fallback("AES-256-GCM")
            .with_ncp_disable(true);
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        assert_eq!(
            get_vpn_data_value(&settings, "data-ciphers").as_deref(),
            Some("AES-256-GCM:AES-128-GCM")
        );
        assert_eq!(
            get_vpn_data_value(&settings, "data-ciphers-fallback").as_deref(),
            Some("AES-256-GCM")
        );
        assert_eq!(
            get_vpn_data_value(&settings, "ncp-disable").as_deref(),
            Some("yes")
        );
    }

    #[test]
    fn openvpn_ipv4_route_data() {
        use crate::api::models::VpnRoute;
        let config = create_openvpn_config()
            .with_routes(vec![VpnRoute::new("10.0.0.0", 24).next_hop("192.168.1.1")]);
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let ipv4 = settings.get("ipv4").unwrap();
        let rd = ipv4.get("route-data").unwrap();
        let Value::Array(arr) = rd else {
            panic!("route-data must be an array");
        };
        assert_eq!(arr.iter().count(), 1, "expected one static route");
    }

    #[test]
    fn openvpn_redirect_gateway_sets_never_default() {
        let config = create_openvpn_config().with_redirect_gateway(true);
        let opts = create_test_options();
        let settings = build_openvpn_connection(&config, &opts).unwrap();
        let ipv4 = settings.get("ipv4").unwrap();
        assert_eq!(ipv4.get("never-default"), Some(&Value::from(false)));
    }
}
