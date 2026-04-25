//! Input validation utilities for NetworkManager operations.
//!
//! This module provides validation functions for various inputs to ensure
//! they meet NetworkManager's requirements before attempting D-Bus operations.

#![allow(deprecated)]

use crate::api::models::{
    ConnectionError, OpenVpnAuthType, OpenVpnConfig, OpenVpnProxy, VpnCredentials, WifiSecurity,
    WireGuardPeer,
};

/// Maximum SSID length in bytes (802.11 standard).
const MAX_SSID_BYTES: usize = 32;

/// WireGuard key length in bytes (before base64 encoding).
const WIREGUARD_KEY_BYTES: usize = 32;

/// WireGuard key length in base64 characters (with padding).
const WIREGUARD_KEY_BASE64_LEN: usize = 44;

/// Minimum WPA-PSK password length (WPA standard).
const MIN_WPA_PSK_LENGTH: usize = 8;

/// Maximum WPA-PSK password length (WPA standard).
const MAX_WPA_PSK_LENGTH: usize = 63;

/// Validates an SSID or connection name string.
///
/// # Rules
/// - Must not be empty (unless explicitly allowed for hidden networks)
/// - Must not exceed 32 bytes when encoded as UTF-8
/// - Should not contain only whitespace
///
/// # Errors
/// Returns `ConnectionError::InvalidAddress` if the SSID is invalid.
pub fn validate_ssid(ssid: &str) -> Result<(), ConnectionError> {
    // Check if empty
    if ssid.is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "SSID cannot be empty".to_string(),
        ));
    }

    // Check if only whitespace
    if ssid.trim().is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "SSID cannot be only whitespace".to_string(),
        ));
    }

    // Check byte length (802.11 standard allows up to 32 bytes)
    if ssid.len() > MAX_SSID_BYTES {
        return Err(ConnectionError::InvalidAddress(format!(
            "SSID too long: {} bytes (max {} bytes)",
            ssid.len(),
            MAX_SSID_BYTES
        )));
    }

    Ok(())
}

/// Validates a connection name (for VPN, etc.).
///
/// Similar to SSID validation but allows slightly more flexibility.
/// Used for VPN connection names and other non-WiFi connection names.
///
/// # Rules
/// - Must not be empty
/// - Should not contain only whitespace
/// - Must not exceed 255 bytes (reasonable limit for connection names)
///
/// # Errors
/// Returns `ConnectionError::InvalidAddress` if the name is invalid.
pub fn validate_connection_name(name: &str) -> Result<(), ConnectionError> {
    // Check if empty
    if name.is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "Connection name cannot be empty".to_string(),
        ));
    }

    // Check if only whitespace
    if name.trim().is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "Connection name cannot be only whitespace".to_string(),
        ));
    }

    // Check byte length (reasonable limit for connection names)
    if name.len() > 255 {
        return Err(ConnectionError::InvalidAddress(format!(
            "Connection name too long: {} bytes (max 255 bytes)",
            name.len()
        )));
    }

    Ok(())
}

/// Validates WiFi security credentials.
///
/// # Rules
/// - WPA-PSK: Password must be 8-63 characters (WPA standard)
/// - WPA-EAP: Identity and password must not be empty
/// - Open: No validation needed
///
/// # Errors
/// Returns appropriate `ConnectionError` if credentials are invalid.
pub fn validate_wifi_security(security: &WifiSecurity) -> Result<(), ConnectionError> {
    match security {
        WifiSecurity::Open => Ok(()),

        WifiSecurity::WpaPsk { psk } => {
            // Allow empty PSK only if user wants to use saved credentials
            if psk.is_empty() {
                return Ok(());
            }

            let psk_len = psk.len();

            if psk_len < MIN_WPA_PSK_LENGTH {
                return Err(ConnectionError::InvalidAddress(format!(
                    "WPA-PSK password too short: {} characters (minimum {} characters)",
                    psk_len, MIN_WPA_PSK_LENGTH
                )));
            }

            if psk_len > MAX_WPA_PSK_LENGTH {
                return Err(ConnectionError::InvalidAddress(format!(
                    "WPA-PSK password too long: {} characters (maximum {} characters)",
                    psk_len, MAX_WPA_PSK_LENGTH
                )));
            }

            Ok(())
        }

        WifiSecurity::WpaEap { opts } => {
            // Validate identity
            if opts.identity.trim().is_empty() {
                return Err(ConnectionError::InvalidAddress(
                    "EAP identity cannot be empty".to_string(),
                ));
            }

            // Validate password
            if opts.password.is_empty() {
                return Err(ConnectionError::InvalidAddress(
                    "EAP password cannot be empty".to_string(),
                ));
            }

            // Validate anonymous identity if provided
            if let Some(ref anon_id) = opts.anonymous_identity
                && anon_id.trim().is_empty()
            {
                return Err(ConnectionError::InvalidAddress(
                    "EAP anonymous identity cannot be empty if provided".to_string(),
                ));
            }

            // Validate domain suffix match if provided
            if let Some(ref domain) = opts.domain_suffix_match
                && domain.trim().is_empty()
            {
                return Err(ConnectionError::InvalidAddress(
                    "EAP domain suffix match cannot be empty if provided".to_string(),
                ));
            }

            // Validate CA cert path if provided
            if let Some(ref ca_cert) = opts.ca_cert_path {
                if ca_cert.trim().is_empty() {
                    return Err(ConnectionError::InvalidAddress(
                        "EAP CA certificate path cannot be empty if provided".to_string(),
                    ));
                }
                // Check if it starts with file:// as required by NetworkManager
                if !ca_cert.starts_with("file://") {
                    return Err(ConnectionError::InvalidAddress(
                        "EAP CA certificate path must start with 'file://'".to_string(),
                    ));
                }
            }

            Ok(())
        }
    }
}

/// Validates a WireGuard private or public key.
///
/// # Rules
/// - Must be valid base64
/// - Must decode to exactly 32 bytes
/// - Must be 44 characters long (base64 with padding)
///
/// # Errors
/// Returns `ConnectionError::InvalidPrivateKey` or `InvalidPublicKey` if invalid.
fn validate_wireguard_key(key: &str, key_type: &str) -> Result<(), ConnectionError> {
    if key.is_empty() {
        return Err(ConnectionError::InvalidPrivateKey(format!(
            "{} cannot be empty",
            key_type
        )));
    }

    // Check length (base64 encoded 32 bytes = 44 chars with padding)
    if key.len() != WIREGUARD_KEY_BASE64_LEN {
        return Err(ConnectionError::InvalidPrivateKey(format!(
            "{} must be {} characters (base64 encoded), got {}",
            key_type,
            WIREGUARD_KEY_BASE64_LEN,
            key.len()
        )));
    }

    // Validate base64 and length
    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, key) {
        Ok(decoded) => {
            if decoded.len() != WIREGUARD_KEY_BYTES {
                return Err(ConnectionError::InvalidPrivateKey(format!(
                    "{} must decode to {} bytes, got {}",
                    key_type,
                    WIREGUARD_KEY_BYTES,
                    decoded.len()
                )));
            }
            Ok(())
        }
        Err(e) => Err(ConnectionError::InvalidPrivateKey(format!(
            "{} is not valid base64: {}",
            key_type, e
        ))),
    }
}

/// Validates a WireGuard peer configuration.
///
/// # Rules
/// - Public key must be valid base64 and 32 bytes
/// - Gateway must be in "host:port" format
/// - Allowed IPs must be valid CIDR notation
/// - Preshared key (if provided) must be valid base64 and 32 bytes
///
/// # Errors
/// Returns appropriate `ConnectionError` if peer configuration is invalid.
fn validate_wireguard_peer(peer: &WireGuardPeer) -> Result<(), ConnectionError> {
    // Validate public key
    validate_wireguard_key(&peer.public_key, "Peer public key")?;

    // Validate gateway (should be host:port)
    if peer.gateway.is_empty() {
        return Err(ConnectionError::InvalidGateway(
            "Peer gateway cannot be empty".to_string(),
        ));
    }

    if !peer.gateway.contains(':') {
        return Err(ConnectionError::InvalidGateway(format!(
            "Peer gateway must be in 'host:port' format, got '{}'",
            peer.gateway
        )));
    }

    // Validate port number
    if let Some(port_str) = peer.gateway.split(':').next_back()
        && port_str.parse::<u16>().is_err()
    {
        return Err(ConnectionError::InvalidGateway(format!(
            "Invalid port number in gateway '{}'",
            peer.gateway
        )));
    }

    // Validate allowed IPs
    if peer.allowed_ips.is_empty() {
        return Err(ConnectionError::InvalidPeers(
            "Peer must have at least one allowed IP range".to_string(),
        ));
    }

    for allowed_ip in &peer.allowed_ips {
        validate_cidr(allowed_ip)?;
    }

    // Validate preshared key if provided
    if let Some(ref psk) = peer.preshared_key {
        validate_wireguard_key(psk, "Peer preshared key")?;
    }

    // Validate persistent keepalive if provided
    if let Some(keepalive) = peer.persistent_keepalive {
        if keepalive == 0 {
            return Err(ConnectionError::InvalidPeers(
                "Persistent keepalive must be greater than 0 if specified".to_string(),
            ));
        }
        if keepalive > 65535 {
            return Err(ConnectionError::InvalidPeers(format!(
                "Persistent keepalive too large: {} (max 65535)",
                keepalive
            )));
        }
    }

    Ok(())
}

/// Validates CIDR notation (e.g., "10.0.0.0/24" or "2001:db8::/32").
///
/// # Errors
/// Returns `ConnectionError::InvalidAddress` if CIDR is invalid.
fn validate_cidr(cidr: &str) -> Result<(), ConnectionError> {
    if cidr.is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "CIDR notation cannot be empty".to_string(),
        ));
    }

    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Err(ConnectionError::InvalidAddress(format!(
            "Invalid CIDR notation '{}' (must be 'address/prefix')",
            cidr
        )));
    }

    let address = parts[0];
    let prefix = parts[1];

    let prefix_num = prefix.parse::<u8>().map_err(|_| {
        ConnectionError::InvalidAddress(format!(
            "Invalid prefix length '{}' in CIDR '{}'",
            prefix, cidr
        ))
    })?;

    if address.contains(':') {
        // IPv6
        if prefix_num > 128 {
            return Err(ConnectionError::InvalidAddress(format!(
                "IPv6 prefix length {} is too large (max 128)",
                prefix_num
            )));
        }
        // Basic IPv6 validation (contains colons and hex digits)
        if !address.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
            return Err(ConnectionError::InvalidAddress(format!(
                "Invalid IPv6 address '{}'",
                address
            )));
        }
    } else {
        // IPv4
        if prefix_num > 32 {
            return Err(ConnectionError::InvalidAddress(format!(
                "IPv4 prefix length {} is too large (max 32)",
                prefix_num
            )));
        }
        // Validate IPv4 format
        let octets: Vec<&str> = address.split('.').collect();
        if octets.len() != 4 {
            return Err(ConnectionError::InvalidAddress(format!(
                "Invalid IPv4 address '{}' (must have 4 octets)",
                address
            )));
        }
        for octet in octets {
            let num = octet.parse::<u16>().map_err(|_| {
                ConnectionError::InvalidAddress(format!("Invalid IPv4 octet '{}'", octet))
            })?;
            if num > 255 {
                return Err(ConnectionError::InvalidAddress(format!(
                    "IPv4 octet {} is too large (max 255)",
                    num
                )));
            }
        }
    }

    Ok(())
}

/// Validates VPN credentials.
///
/// # Rules
/// - Name must not be empty
/// - Gateway must be in "host:port" format
/// - Private key must be valid base64 and 32 bytes
/// - Address must be valid CIDR notation
/// - At least one peer must be configured
/// - All peers must be valid
/// - DNS servers (if provided) must be valid IP addresses
/// - MTU (if provided) must be reasonable (576-9000)
///
/// # Errors
/// Returns appropriate `ConnectionError` if credentials are invalid.
pub fn validate_vpn_credentials(creds: &VpnCredentials) -> Result<(), ConnectionError> {
    // Validate name
    validate_connection_name(&creds.name)?;

    // Validate gateway
    if creds.gateway.is_empty() {
        return Err(ConnectionError::InvalidGateway(
            "VPN gateway cannot be empty".to_string(),
        ));
    }

    if !creds.gateway.contains(':') {
        return Err(ConnectionError::InvalidGateway(format!(
            "VPN gateway must be in 'host:port' format, got '{}'",
            creds.gateway
        )));
    }

    // Validate port number
    if let Some(port_str) = creds.gateway.split(':').next_back()
        && port_str.parse::<u16>().is_err()
    {
        return Err(ConnectionError::InvalidGateway(format!(
            "Invalid port number in gateway '{}'",
            creds.gateway
        )));
    }

    // Validate private key
    validate_wireguard_key(&creds.private_key, "Private key")?;

    // Validate address (must be CIDR notation)
    validate_cidr(&creds.address)?;

    // Validate peers
    if creds.peers.is_empty() {
        return Err(ConnectionError::InvalidPeers(
            "VPN must have at least one peer configured".to_string(),
        ));
    }

    for (i, peer) in creds.peers.iter().enumerate() {
        validate_wireguard_peer(peer).map_err(|e| match e {
            ConnectionError::InvalidPeers(msg) => {
                ConnectionError::InvalidPeers(format!("Peer {}: {}", i, msg))
            }
            ConnectionError::InvalidGateway(msg) => {
                ConnectionError::InvalidGateway(format!("Peer {}: {}", i, msg))
            }
            ConnectionError::InvalidPublicKey(msg) => {
                ConnectionError::InvalidPublicKey(format!("Peer {}: {}", i, msg))
            }
            other => other,
        })?;
    }

    // Validate DNS servers if provided
    if let Some(ref dns_servers) = creds.dns {
        if dns_servers.is_empty() {
            return Err(ConnectionError::InvalidAddress(
                "DNS server list cannot be empty if provided".to_string(),
            ));
        }

        for dns in dns_servers {
            validate_ip_address(dns)?;
        }
    }

    validate_mtu(creds.mtu)?;

    Ok(())
}

/// Validates an IP address (IPv4 or IPv6).
///
/// # Errors
/// Returns `ConnectionError::InvalidAddress` if the IP address is invalid.
fn validate_ip_address(ip: &str) -> Result<(), ConnectionError> {
    if ip.is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "IP address cannot be empty".to_string(),
        ));
    }

    if ip.contains(':') {
        if !ip.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
            return Err(ConnectionError::InvalidAddress(format!(
                "Invalid IPv6 address '{}'",
                ip
            )));
        }
    } else {
        let octets: Vec<&str> = ip.split('.').collect();
        if octets.len() != 4 {
            return Err(ConnectionError::InvalidAddress(format!(
                "Invalid IPv4 address '{}' (must have 4 octets)",
                ip
            )));
        }
        for octet in octets {
            let num = octet.parse::<u16>().map_err(|_| {
                ConnectionError::InvalidAddress(format!(
                    "Invalid IPv4 octet '{}' in address '{}'",
                    octet, ip
                ))
            })?;
            if num > 255 {
                return Err(ConnectionError::InvalidAddress(format!(
                    "IPv4 octet {} is too large (max 255) in address '{}'",
                    num, ip
                )));
            }
        }
    }

    Ok(())
}

/// Validates an OpenVPN configuration.
///
/// # Rules
/// - Connection name must be valid (via [`validate_connection_name`])
/// - Remote server must not be empty
/// - Port is validated at the type level (`u16`), no extra check needed
/// - Auth-type-specific required fields:
///   - `Password`: username must be set
///   - `Tls`: CA cert, client cert, and client key must be set
///   - `PasswordTls`: username plus all TLS cert paths must be set
///   - `StaticKey`: no additional fields required
/// - Cert paths (if set) must be non-empty strings
/// - DNS servers (if provided) must be valid IP addresses
/// - MTU (if provided) must be in 576–9000
/// - Proxy server (if provided) must not be empty
///
/// # Errors
/// Returns appropriate `ConnectionError` if the configuration is invalid.
pub fn validate_openvpn_config(config: &OpenVpnConfig) -> Result<(), ConnectionError> {
    validate_connection_name(&config.name)?;

    if config.remote.trim().is_empty() {
        return Err(ConnectionError::InvalidGateway(
            "OpenVPN remote server cannot be empty".to_string(),
        ));
    }

    if let Some(ref auth_type) = config.auth_type {
        match auth_type {
            OpenVpnAuthType::Password => {
                if config.username.as_deref().unwrap_or("").is_empty() {
                    return Err(ConnectionError::InvalidAddress(
                        "Username is required for password authentication".to_string(),
                    ));
                }
            }
            OpenVpnAuthType::Tls => {
                validate_openvpn_cert_paths(config)?;
            }
            OpenVpnAuthType::PasswordTls => {
                if config.username.as_deref().unwrap_or("").is_empty() {
                    return Err(ConnectionError::InvalidAddress(
                        "Username is required for password+TLS authentication".to_string(),
                    ));
                }
                validate_openvpn_cert_paths(config)?;
            }
            OpenVpnAuthType::StaticKey => {}
        }
    }

    validate_optional_cert_path(&config.ca_cert, "CA certificate")?;
    validate_optional_cert_path(&config.client_cert, "Client certificate")?;
    validate_optional_cert_path(&config.client_key, "Client key")?;

    if let Some(ref dns_servers) = config.dns {
        if dns_servers.is_empty() {
            return Err(ConnectionError::InvalidAddress(
                "DNS server list cannot be empty if provided".to_string(),
            ));
        }
        for dns in dns_servers {
            validate_ip_address(dns)?;
        }
    }

    validate_mtu(config.mtu)?;

    if let Some(ref proxy) = config.proxy {
        match proxy {
            OpenVpnProxy::Http { server, .. } | OpenVpnProxy::Socks { server, .. } => {
                if server.trim().is_empty() {
                    return Err(ConnectionError::InvalidAddress(
                        "Proxy server address cannot be empty".to_string(),
                    ));
                }
            }
        }
    }

    for route in &config.routes {
        if route.dest.trim().is_empty() {
            return Err(ConnectionError::InvalidAddress(
                "OpenVPN route destination cannot be empty".to_string(),
            ));
        }
        if route.prefix > 32 {
            return Err(ConnectionError::InvalidAddress(format!(
                "OpenVPN route prefix must be at most 32, got {}",
                route.prefix
            )));
        }
        if let Some(ref nh) = route.next_hop {
            validate_ip_address(nh)?;
        }
    }

    for (label, val) in [
        ("ping", config.ping),
        ("ping-exit", config.ping_exit),
        ("ping-restart", config.ping_restart),
        ("reneg-sec", config.reneg_seconds),
        ("connect-timeout", config.connect_timeout),
    ] {
        if let Some(v) = val
            && v == 0
        {
            return Err(ConnectionError::InvalidAddress(format!(
                "{label} must be greater than 0 if set"
            )));
        }
    }

    Ok(())
}

/// Validates that TLS cert paths required for certificate authentication are present.
fn validate_openvpn_cert_paths(config: &OpenVpnConfig) -> Result<(), ConnectionError> {
    if config.ca_cert.as_deref().unwrap_or("").is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "CA certificate path is required for TLS authentication".to_string(),
        ));
    }
    if config.client_cert.as_deref().unwrap_or("").is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "Client certificate path is required for TLS authentication".to_string(),
        ));
    }
    if config.client_key.as_deref().unwrap_or("").is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "Client key path is required for TLS authentication".to_string(),
        ));
    }
    Ok(())
}

/// Validates that an optional certificate path, if provided, is non-empty.
fn validate_optional_cert_path(path: &Option<String>, label: &str) -> Result<(), ConnectionError> {
    if let Some(p) = path
        && p.trim().is_empty()
    {
        return Err(ConnectionError::InvalidAddress(format!(
            "{label} path cannot be empty if provided"
        )));
    }
    Ok(())
}

/// Validates an MTU value (576–9000).
fn validate_mtu(mtu: Option<u32>) -> Result<(), ConnectionError> {
    if let Some(mtu) = mtu {
        if mtu < 576 {
            return Err(ConnectionError::InvalidAddress(format!(
                "MTU too small: {mtu} (minimum 576)"
            )));
        }
        if mtu > 9000 {
            return Err(ConnectionError::InvalidAddress(format!(
                "MTU too large: {mtu} (maximum 9000)"
            )));
        }
    }
    Ok(())
}

/// Validates a Bluetooth address against the EUI-48 format (using colons).
///
/// # Errors
/// Returns `ConnectionError::InvalidAddress` if the Bluetooth address is invalid.
pub fn validate_bluetooth_address(bdaddr: &str) -> Result<(), ConnectionError> {
    let parts: Vec<&str> = bdaddr.split(':').collect();

    if parts.len() != 6 {
        return Err(ConnectionError::InvalidAddress(format!(
            "Invalid Bluetooth Address '{}' (must have 6 segments)",
            bdaddr,
        )));
    }

    for part in parts {
        if part.len() != 2 {
            return Err(ConnectionError::InvalidAddress(format!(
                "Invalid segment '{}' in Bluetooth Address '{}' (must be 2 characters)",
                part, bdaddr
            )));
        }

        if !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ConnectionError::InvalidAddress(format!(
                "Invalid segment '{}' in Bluetooth Address '{}' (must be hex digits)",
                part, bdaddr
            )));
        }
    }

    Ok(())
}

/// Validates a BSSID (MAC address) in `XX:XX:XX:XX:XX:XX` format.
///
/// Both uppercase and lowercase hex digits are accepted.
///
/// # Errors
///
/// Returns [`ConnectionError::InvalidBssid`] if the format is invalid.
pub fn validate_bssid(bssid: &str) -> Result<(), ConnectionError> {
    let parts: Vec<&str> = bssid.split(':').collect();

    if parts.len() != 6 {
        return Err(ConnectionError::InvalidBssid(bssid.to_string()));
    }

    for part in parts {
        if part.len() != 2 || !part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ConnectionError::InvalidBssid(bssid.to_string()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::{EapMethod, EapOptions, Phase2};

    #[test]
    fn test_validate_ssid_valid() {
        assert!(validate_ssid("MyNetwork").is_ok());
        assert!(validate_ssid("Test-Network_123").is_ok());
        assert!(validate_ssid("A").is_ok());
        assert!(validate_ssid("12345678901234567890123456789012").is_ok()); // 32 bytes
    }

    #[test]
    fn test_validate_ssid_empty() {
        assert!(validate_ssid("").is_err());
        assert!(validate_ssid("   ").is_err());
    }

    #[test]
    fn test_validate_ssid_too_long() {
        let long_ssid = "123456789012345678901234567890123"; // 33 bytes
        assert!(validate_ssid(long_ssid).is_err());
    }

    #[test]
    fn test_validate_connection_name_valid() {
        assert!(validate_connection_name("MyVPN").is_ok());
        assert!(validate_connection_name("Test-VPN_123").is_ok());
        assert!(validate_connection_name("A").is_ok());
        // Connection names can be longer than SSIDs
        assert!(validate_connection_name(&"a".repeat(255)).is_ok());
    }

    #[test]
    fn test_validate_connection_name_too_long() {
        let long_name = "a".repeat(256);
        assert!(validate_connection_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_wifi_security_open() {
        assert!(validate_wifi_security(&WifiSecurity::Open).is_ok());
    }

    #[test]
    fn test_validate_wifi_security_psk_valid() {
        let psk = WifiSecurity::WpaPsk {
            psk: "password123".to_string(),
        };
        assert!(validate_wifi_security(&psk).is_ok());
    }

    #[test]
    fn test_validate_wifi_security_psk_empty() {
        let psk = WifiSecurity::WpaPsk {
            psk: "".to_string(),
        };
        // Empty PSK is allowed (for saved credentials)
        assert!(validate_wifi_security(&psk).is_ok());
    }

    #[test]
    fn test_validate_wifi_security_psk_too_short() {
        let psk = WifiSecurity::WpaPsk {
            psk: "short".to_string(),
        };
        assert!(validate_wifi_security(&psk).is_err());
    }

    #[test]
    fn test_validate_wifi_security_psk_too_long() {
        let psk = WifiSecurity::WpaPsk {
            psk: "a".repeat(64),
        };
        assert!(validate_wifi_security(&psk).is_err());
    }

    #[test]
    fn test_validate_wifi_security_eap_valid() {
        let eap = WifiSecurity::WpaEap {
            opts: EapOptions {
                identity: "user@example.com".to_string(),
                password: "password".to_string(),
                anonymous_identity: None,
                domain_suffix_match: Some("example.com".to_string()),
                ca_cert_path: Some("file:///etc/ssl/cert.pem".to_string()),
                system_ca_certs: false,
                method: EapMethod::Peap,
                phase2: Phase2::Mschapv2,
            },
        };
        assert!(validate_wifi_security(&eap).is_ok());
    }

    #[test]
    fn test_validate_wifi_security_eap_empty_identity() {
        let eap = WifiSecurity::WpaEap {
            opts: EapOptions {
                identity: "".to_string(),
                password: "password".to_string(),
                anonymous_identity: None,
                domain_suffix_match: None,
                ca_cert_path: None,
                system_ca_certs: true,
                method: EapMethod::Peap,
                phase2: Phase2::Mschapv2,
            },
        };
        assert!(validate_wifi_security(&eap).is_err());
    }

    #[test]
    fn test_validate_wifi_security_eap_invalid_ca_cert() {
        let eap = WifiSecurity::WpaEap {
            opts: EapOptions {
                identity: "user@example.com".to_string(),
                password: "password".to_string(),
                anonymous_identity: None,
                domain_suffix_match: None,
                ca_cert_path: Some("/etc/ssl/cert.pem".to_string()), // Missing file://
                system_ca_certs: false,
                method: EapMethod::Peap,
                phase2: Phase2::Mschapv2,
            },
        };
        assert!(validate_wifi_security(&eap).is_err());
    }

    #[test]
    fn test_validate_cidr_ipv4_valid() {
        assert!(validate_cidr("10.0.0.0/24").is_ok());
        assert!(validate_cidr("192.168.1.0/16").is_ok());
        assert!(validate_cidr("0.0.0.0/0").is_ok());
    }

    #[test]
    fn test_validate_cidr_ipv6_valid() {
        assert!(validate_cidr("2001:db8::/32").is_ok());
        assert!(validate_cidr("::/0").is_ok());
    }

    #[test]
    fn test_validate_cidr_invalid() {
        assert!(validate_cidr("10.0.0.0").is_err()); // Missing prefix
        assert!(validate_cidr("10.0.0.0/33").is_err()); // Invalid prefix
        assert!(validate_cidr("256.0.0.0/24").is_err()); // Invalid octet
        assert!(validate_cidr("10.0.0/24").is_err()); // Wrong number of octets
    }

    #[test]
    fn test_validate_ip_address_ipv4_valid() {
        assert!(validate_ip_address("192.168.1.1").is_ok());
        assert!(validate_ip_address("8.8.8.8").is_ok());
        assert!(validate_ip_address("0.0.0.0").is_ok());
    }

    #[test]
    fn test_validate_ip_address_ipv4_invalid() {
        assert!(validate_ip_address("256.1.1.1").is_err());
        assert!(validate_ip_address("192.168.1").is_err());
        assert!(validate_ip_address("192.168.1.1.1").is_err());
    }

    #[test]
    fn test_validate_wireguard_key_valid() {
        // Valid 32-byte base64 key
        let key = "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=";
        assert!(validate_wireguard_key(key, "Test key").is_ok());
    }

    #[test]
    fn test_validate_wireguard_key_invalid_length() {
        let key = "tooshort";
        assert!(validate_wireguard_key(key, "Test key").is_err());
    }

    #[test]
    fn test_validate_wireguard_key_invalid_base64() {
        let key = "!!!invalid-base64-characters-here!!!";
        assert!(validate_wireguard_key(key, "Test key").is_err());
    }

    #[test]
    fn test_validate_bluetooth_address_valid() {
        assert!(validate_bluetooth_address("00:1A:7D:DA:71:13").is_ok());
        assert!(validate_bluetooth_address("00:1a:7d:da:71:13").is_ok());
        assert!(validate_bluetooth_address("aA:bB:cC:dD:eE:fF").is_ok());
    }

    #[test]
    fn test_validate_bluetooth_address_invalid_format() {
        assert!(validate_bluetooth_address("00-1A-7D-DA-71-13").is_err());
        assert!(validate_bluetooth_address("001A7DDA7113").is_err());
        assert!(validate_bluetooth_address("00:1A:7D:DA:711:3").is_err());
    }

    #[test]
    fn test_validate_bluetooth_address_invalid_char() {
        assert!(validate_bluetooth_address("00:1A:7D:DA:71:GG").is_err());
        assert!(validate_bluetooth_address("00:1A:7D:DA:71:!!").is_err());
    }

    #[test]
    fn test_validate_bluetooth_address_invalid_length() {
        assert!(validate_bluetooth_address("00:1A:7D").is_err());
        assert!(validate_bluetooth_address("00:1A:7D:DA:71:13:FF").is_err());
        assert!(validate_bluetooth_address("").is_err());
    }

    fn base_openvpn_config() -> OpenVpnConfig {
        OpenVpnConfig::new("MyVPN", "vpn.example.com", 1194, false)
    }

    #[test]
    fn test_validate_openvpn_valid_minimal() {
        assert!(validate_openvpn_config(&base_openvpn_config()).is_ok());
    }

    #[test]
    fn test_validate_openvpn_empty_name() {
        let config = OpenVpnConfig::new("", "vpn.example.com", 1194, false);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_whitespace_name() {
        let config = OpenVpnConfig::new("   ", "vpn.example.com", 1194, false);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_empty_remote() {
        let config = OpenVpnConfig::new("MyVPN", "", 1194, false);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_whitespace_remote() {
        let config = OpenVpnConfig::new("MyVPN", "   ", 1194, false);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_password_auth_missing_username() {
        let config = base_openvpn_config().with_auth_type(OpenVpnAuthType::Password);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_password_auth_with_username() {
        let config = base_openvpn_config()
            .with_auth_type(OpenVpnAuthType::Password)
            .with_username("user");
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_tls_auth_missing_certs() {
        let config = base_openvpn_config().with_auth_type(OpenVpnAuthType::Tls);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_tls_auth_partial_certs() {
        let config = base_openvpn_config()
            .with_auth_type(OpenVpnAuthType::Tls)
            .with_ca_cert("/path/to/ca.crt");
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_tls_auth_with_all_certs() {
        let config = base_openvpn_config()
            .with_auth_type(OpenVpnAuthType::Tls)
            .with_ca_cert("/path/to/ca.crt")
            .with_client_cert("/path/to/client.crt")
            .with_client_key("/path/to/client.key");
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_password_tls_missing_username() {
        let config = base_openvpn_config()
            .with_auth_type(OpenVpnAuthType::PasswordTls)
            .with_ca_cert("/path/to/ca.crt")
            .with_client_cert("/path/to/client.crt")
            .with_client_key("/path/to/client.key");
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_password_tls_missing_certs() {
        let config = base_openvpn_config()
            .with_auth_type(OpenVpnAuthType::PasswordTls)
            .with_username("user");
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_password_tls_complete() {
        let config = base_openvpn_config()
            .with_auth_type(OpenVpnAuthType::PasswordTls)
            .with_username("user")
            .with_ca_cert("/path/to/ca.crt")
            .with_client_cert("/path/to/client.crt")
            .with_client_key("/path/to/client.key");
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_static_key_minimal() {
        let config = base_openvpn_config().with_auth_type(OpenVpnAuthType::StaticKey);
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_empty_cert_path_provided() {
        let config = base_openvpn_config().with_ca_cert("");
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_whitespace_cert_path() {
        let config = base_openvpn_config().with_client_cert("   ");
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_valid_dns() {
        let config = base_openvpn_config().with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()]);
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_empty_dns_list() {
        let config = base_openvpn_config().with_dns(vec![]);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_invalid_dns() {
        let config = base_openvpn_config().with_dns(vec!["not-an-ip".into()]);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_mtu_too_small() {
        let config = base_openvpn_config().with_mtu(100);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_mtu_too_large() {
        let config = base_openvpn_config().with_mtu(10000);
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_mtu_valid() {
        let config = base_openvpn_config().with_mtu(1500);
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_mtu_boundary_min() {
        let config = base_openvpn_config().with_mtu(576);
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_mtu_boundary_max() {
        let config = base_openvpn_config().with_mtu(9000);
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_empty_proxy_server() {
        let config = base_openvpn_config().with_proxy(OpenVpnProxy::Http {
            server: "".into(),
            port: 8080,
            username: None,
            password: None,
            retry: false,
        });
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_valid_http_proxy() {
        let config = base_openvpn_config().with_proxy(OpenVpnProxy::Http {
            server: "proxy.example.com".into(),
            port: 8080,
            username: None,
            password: None,
            retry: false,
        });
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_empty_socks_proxy_server() {
        let config = base_openvpn_config().with_proxy(OpenVpnProxy::Socks {
            server: "  ".into(),
            port: 1080,
            retry: false,
        });
        assert!(validate_openvpn_config(&config).is_err());
    }

    #[test]
    fn test_validate_openvpn_valid_socks_proxy() {
        let config = base_openvpn_config().with_proxy(OpenVpnProxy::Socks {
            server: "socks.example.com".into(),
            port: 1080,
            retry: false,
        });
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_openvpn_no_auth_type_is_valid() {
        let config = base_openvpn_config();
        assert!(config.auth_type.is_none());
        assert!(validate_openvpn_config(&config).is_ok());
    }

    #[test]
    fn test_validate_bssid_valid_uppercase() {
        assert!(validate_bssid("AA:BB:CC:DD:EE:FF").is_ok());
    }

    #[test]
    fn test_validate_bssid_valid_lowercase() {
        assert!(validate_bssid("aa:bb:cc:dd:ee:ff").is_ok());
    }

    #[test]
    fn test_validate_bssid_valid_mixed() {
        assert!(validate_bssid("aA:Bb:cC:Dd:eE:fF").is_ok());
    }

    #[test]
    fn test_validate_bssid_too_short() {
        assert!(validate_bssid("AA:BB:CC:DD:EE").is_err());
    }

    #[test]
    fn test_validate_bssid_empty() {
        assert!(validate_bssid("").is_err());
    }

    #[test]
    fn test_validate_bssid_unicode() {
        assert!(validate_bssid("AA:BB:CC:DD:EE:ÀÀ").is_err());
    }

    #[test]
    fn test_validate_bssid_invalid_segment() {
        assert!(validate_bssid("GG:BB:CC:DD:EE:FF").is_err());
    }
}
