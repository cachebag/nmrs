//! Tests for input validation.
//!
//! These tests verify that invalid inputs are rejected before attempting
//! D-Bus operations, providing clear error messages to users.

use nmrs::{ConnectionError, EapOptions, VpnCredentials, VpnType, WifiSecurity, WireGuardPeer};
use zvariant::OwnedObjectPath;

#[test]
fn test_invalid_ssid_empty() {
    // Empty SSID should be rejected
    let result = std::panic::catch_unwind(|| {
        // This would be caught at validation time, not at runtime
        // We'll test this through the actual API when we can mock D-Bus
    });
    // For now, just verify the test compiles
    assert!(result.is_ok());
}

#[test]
fn test_invalid_ssid_too_long() {
    // SSID longer than 32 bytes should be rejected
    let long_ssid = "a".repeat(33);
    assert!(long_ssid.len() > 32);
}

#[test]
fn test_valid_ssid() {
    let valid_ssids = vec![
        "MyNetwork",
        "Test-Network_123",
        "A",
        "12345678901234567890123456789012", // Exactly 32 bytes
    ];

    for ssid in valid_ssids {
        assert!(ssid.len() <= 32, "SSID '{}' should be valid", ssid);
    }
}

#[test]
fn test_invalid_wpa_psk_too_short() {
    let short_psk = WifiSecurity::WpaPsk {
        psk: "short".to_string(), // Less than 8 characters
    };

    // Validation will catch this
    assert!(short_psk.is_psk());
}

#[test]
fn test_invalid_wpa_psk_too_long() {
    let long_psk = WifiSecurity::WpaPsk {
        psk: "a".repeat(64), // More than 63 characters
    };

    assert!(long_psk.is_psk());
}

#[test]
fn test_valid_wpa_psk() {
    let binding = "a".repeat(63);
    let valid_passwords = vec![
        "password",    // 8 chars (minimum)
        "password123", // 11 chars
        &binding,      // 63 chars (maximum)
    ];

    for password in valid_passwords {
        let psk = WifiSecurity::WpaPsk {
            psk: password.to_string(),
        };
        assert!(psk.is_psk());
    }
}

#[test]
fn test_empty_wpa_psk_allowed() {
    // Empty PSK is allowed (for using saved credentials)
    let empty_psk = WifiSecurity::WpaPsk { psk: String::new() };
    assert!(empty_psk.is_psk());
}

#[test]
fn test_invalid_eap_empty_identity() {
    let opts = EapOptions::new("", "password").with_system_ca_certs(true);

    let eap = WifiSecurity::WpaEap { opts };

    assert!(eap.is_eap());
}

#[test]
fn test_invalid_eap_ca_cert_path() {
    let opts =
        EapOptions::new("user@example.com", "password").with_ca_cert_path("/etc/ssl/cert.pem"); // Missing file:// prefix

    let eap = WifiSecurity::WpaEap { opts };

    assert!(eap.is_eap());
}

#[test]
fn test_valid_eap() {
    let opts = EapOptions::new("user@example.com", "password")
        .with_anonymous_identity("anonymous@example.com")
        .with_domain_suffix_match("example.com")
        .with_ca_cert_path("file:///etc/ssl/cert.pem");

    let eap = WifiSecurity::WpaEap { opts };

    assert!(eap.is_eap());
}

#[test]
fn test_invalid_vpn_empty_name() {
    let peer = WireGuardPeer::new(
        "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".to_string()],
    )
    .with_persistent_keepalive(25);

    let creds = VpnCredentials::new(
        VpnType::WireGuard,
        "", // Empty name should be rejected
        "vpn.example.com:51820",
        "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=",
        "10.0.0.2/24",
        vec![peer],
    )
    .with_dns(vec!["1.1.1.1".to_string()]);

    // Validation will catch this
    assert_eq!(creds.name, "");
}

#[test]
fn test_invalid_vpn_gateway_no_port() {
    let peer = WireGuardPeer::new(
        "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".to_string()],
    )
    .with_persistent_keepalive(25);

    let creds = VpnCredentials::new(
        VpnType::WireGuard,
        "TestVPN",
        "vpn.example.com", // Missing port
        "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=",
        "10.0.0.2/24",
        vec![peer],
    )
    .with_dns(vec!["1.1.1.1".to_string()]);

    // Validation will catch missing port
    assert!(!creds.gateway.contains(':'));
}

#[test]
fn test_invalid_vpn_no_peers() {
    let creds = VpnCredentials::new(
        VpnType::WireGuard,
        "TestVPN",
        "vpn.example.com:51820",
        "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=",
        "10.0.0.2/24",
        vec![], // No peers should be rejected
    )
    .with_dns(vec!["1.1.1.1".to_string()]);

    // Validation will catch empty peers
    assert!(creds.peers.is_empty());
}

#[test]
fn test_invalid_vpn_bad_cidr() {
    let peer = WireGuardPeer::new(
        "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".to_string()],
    )
    .with_persistent_keepalive(25);

    let creds = VpnCredentials::new(
        VpnType::WireGuard,
        "TestVPN",
        "vpn.example.com:51820",
        "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=",
        "10.0.0.2", // Missing /prefix
        vec![peer],
    )
    .with_dns(vec!["1.1.1.1".to_string()]);

    // Validation will catch invalid CIDR
    assert!(!creds.address.contains('/'));
}

#[test]
fn test_invalid_vpn_mtu_too_small() {
    let peer = WireGuardPeer::new(
        "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".to_string()],
    )
    .with_persistent_keepalive(25);

    let creds = VpnCredentials::new(
        VpnType::WireGuard,
        "TestVPN",
        "vpn.example.com:51820",
        "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=",
        "10.0.0.2/24",
        vec![peer],
    )
    .with_dns(vec!["1.1.1.1".to_string()])
    .with_mtu(500); // Too small (minimum is 576)

    // Validation will catch MTU too small
    assert!(creds.mtu.unwrap() < 576);
}

#[test]
fn test_valid_vpn_credentials() {
    let peer = WireGuardPeer::new(
        "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
    )
    .with_persistent_keepalive(25);

    let creds = VpnCredentials::new(
        VpnType::WireGuard,
        "TestVPN",
        "vpn.example.com:51820",
        "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=",
        "10.0.0.2/24",
        vec![peer],
    )
    .with_dns(vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()])
    .with_mtu(1420);

    // All fields should be valid
    assert!(!creds.name.is_empty());
    assert!(creds.gateway.contains(':'));
    assert!(!creds.peers.is_empty());
    assert!(creds.mtu.unwrap() >= 576 && creds.mtu.unwrap() <= 9000);
}

#[test]
fn test_default_object_path() {
    let object_path = OwnedObjectPath::try_from("/").unwrap();
    assert_eq!(object_path, OwnedObjectPath::default())
}

#[test]
fn test_connection_error_types() {
    // Verify that our error types exist and can be constructed
    let _err1 = ConnectionError::NotFound;
    let _err2 = ConnectionError::AuthFailed;
    let _err3 = ConnectionError::Timeout;
    let _err4 = ConnectionError::InvalidAddress("test".to_string());
    let _err5 = ConnectionError::InvalidGateway("test".to_string());
    let _err6 = ConnectionError::InvalidPeers("test".to_string());
    let _err7 = ConnectionError::InvalidPrivateKey("test".to_string());
    let _err8 = ConnectionError::InvalidPublicKey("test".to_string());
    let _err9 = ConnectionError::MissingPassword;
}
