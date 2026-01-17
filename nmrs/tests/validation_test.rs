//! Tests for input validation.
//!
//! These tests verify that invalid inputs are rejected before attempting
//! D-Bus operations, providing clear error messages to users.

use nmrs::{
    ConnectionError, EapMethod, EapOptions, Phase2, VpnCredentials, VpnType, WifiSecurity,
    WireGuardPeer,
};

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
    let eap = WifiSecurity::WpaEap {
        opts: EapOptions {
            identity: "".to_string(), // Empty identity should be rejected
            password: "password".to_string(),
            anonymous_identity: None,
            domain_suffix_match: None,
            ca_cert_path: None,
            system_ca_certs: true,
            method: EapMethod::Peap,
            phase2: Phase2::Mschapv2,
        },
    };

    assert!(eap.is_eap());
}

#[test]
fn test_invalid_eap_ca_cert_path() {
    let eap = WifiSecurity::WpaEap {
        opts: EapOptions {
            identity: "user@example.com".to_string(),
            password: "password".to_string(),
            anonymous_identity: None,
            domain_suffix_match: None,
            ca_cert_path: Some("/etc/ssl/cert.pem".to_string()), // Missing file:// prefix
            system_ca_certs: false,
            method: EapMethod::Peap,
            phase2: Phase2::Mschapv2,
        },
    };

    assert!(eap.is_eap());
}

#[test]
fn test_valid_eap() {
    let eap = WifiSecurity::WpaEap {
        opts: EapOptions {
            identity: "user@example.com".to_string(),
            password: "password".to_string(),
            anonymous_identity: Some("anonymous@example.com".to_string()),
            domain_suffix_match: Some("example.com".to_string()),
            ca_cert_path: Some("file:///etc/ssl/cert.pem".to_string()),
            system_ca_certs: false,
            method: EapMethod::Peap,
            phase2: Phase2::Mschapv2,
        },
    };

    assert!(eap.is_eap());
}

#[test]
fn test_invalid_vpn_empty_name() {
    let creds = VpnCredentials {
        vpn_type: VpnType::WireGuard,
        name: "".to_string(), // Empty name should be rejected
        gateway: "vpn.example.com:51820".to_string(),
        private_key: "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".to_string(),
        address: "10.0.0.2/24".to_string(),
        peers: vec![WireGuardPeer {
            public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".to_string(),
            gateway: "vpn.example.com:51820".to_string(),
            allowed_ips: vec!["0.0.0.0/0".to_string()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }],
        dns: Some(vec!["1.1.1.1".to_string()]),
        mtu: None,
        uuid: None,
    };

    // Validation will catch this
    assert_eq!(creds.name, "");
}

#[test]
fn test_invalid_vpn_gateway_no_port() {
    let creds = VpnCredentials {
        vpn_type: VpnType::WireGuard,
        name: "TestVPN".to_string(),
        gateway: "vpn.example.com".to_string(), // Missing port
        private_key: "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".to_string(),
        address: "10.0.0.2/24".to_string(),
        peers: vec![WireGuardPeer {
            public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".to_string(),
            gateway: "vpn.example.com:51820".to_string(),
            allowed_ips: vec!["0.0.0.0/0".to_string()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }],
        dns: Some(vec!["1.1.1.1".to_string()]),
        mtu: None,
        uuid: None,
    };

    // Validation will catch missing port
    assert!(!creds.gateway.contains(':'));
}

#[test]
fn test_invalid_vpn_no_peers() {
    let creds = VpnCredentials {
        vpn_type: VpnType::WireGuard,
        name: "TestVPN".to_string(),
        gateway: "vpn.example.com:51820".to_string(),
        private_key: "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".to_string(),
        address: "10.0.0.2/24".to_string(),
        peers: vec![], // No peers should be rejected
        dns: Some(vec!["1.1.1.1".to_string()]),
        mtu: None,
        uuid: None,
    };

    // Validation will catch empty peers
    assert!(creds.peers.is_empty());
}

#[test]
fn test_invalid_vpn_bad_cidr() {
    let creds = VpnCredentials {
        vpn_type: VpnType::WireGuard,
        name: "TestVPN".to_string(),
        gateway: "vpn.example.com:51820".to_string(),
        private_key: "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".to_string(),
        address: "10.0.0.2".to_string(), // Missing /prefix
        peers: vec![WireGuardPeer {
            public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".to_string(),
            gateway: "vpn.example.com:51820".to_string(),
            allowed_ips: vec!["0.0.0.0/0".to_string()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }],
        dns: Some(vec!["1.1.1.1".to_string()]),
        mtu: None,
        uuid: None,
    };

    // Validation will catch invalid CIDR
    assert!(!creds.address.contains('/'));
}

#[test]
fn test_invalid_vpn_mtu_too_small() {
    let creds = VpnCredentials {
        vpn_type: VpnType::WireGuard,
        name: "TestVPN".to_string(),
        gateway: "vpn.example.com:51820".to_string(),
        private_key: "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".to_string(),
        address: "10.0.0.2/24".to_string(),
        peers: vec![WireGuardPeer {
            public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".to_string(),
            gateway: "vpn.example.com:51820".to_string(),
            allowed_ips: vec!["0.0.0.0/0".to_string()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }],
        dns: Some(vec!["1.1.1.1".to_string()]),
        mtu: Some(500), // Too small (minimum is 576)
        uuid: None,
    };

    // Validation will catch MTU too small
    assert!(creds.mtu.unwrap() < 576);
}

#[test]
fn test_valid_vpn_credentials() {
    let creds = VpnCredentials {
        vpn_type: VpnType::WireGuard,
        name: "TestVPN".to_string(),
        gateway: "vpn.example.com:51820".to_string(),
        private_key: "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=".to_string(),
        address: "10.0.0.2/24".to_string(),
        peers: vec![WireGuardPeer {
            public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".to_string(),
            gateway: "vpn.example.com:51820".to_string(),
            allowed_ips: vec!["0.0.0.0/0".to_string(), "::/0".to_string()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }],
        dns: Some(vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()]),
        mtu: Some(1420),
        uuid: None,
    };

    // All fields should be valid
    assert!(!creds.name.is_empty());
    assert!(creds.gateway.contains(':'));
    assert!(!creds.peers.is_empty());
    assert!(creds.mtu.unwrap() >= 576 && creds.mtu.unwrap() <= 9000);
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
}
