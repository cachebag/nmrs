//! NetworkManager connection settings builder.
//!
//! Constructs the D-Bus settings dictionaries required by NetworkManager's
//! `AddAndActivateConnection` method. These settings define the connection
//! type, security parameters, and IP configuration.
//!
//! # NetworkManager Settings Structure
//!
//! A connection is represented as a nested dictionary:
//! - `connection`: General settings (type, id, uuid, autoconnect)
//! - `802-11-wireless`: Wi-Fi specific settings (ssid, mode, security reference)
//! - `802-11-wireless-security`: Security settings (key-mgmt, psk, auth-alg)
//! - `802-1x`: Enterprise authentication settings (for WPA-EAP)
//! - `ipv4` / `ipv6`: IP configuration (usually "auto" for DHCP)

use models::ConnectionOptions;
use std::collections::HashMap;
use zvariant::Value;

use crate::models::{self, EapMethod};

/// Converts a string to bytes for SSID encoding.
fn bytes(val: &str) -> Vec<u8> {
    val.as_bytes().to_vec()
}

/// Creates a D-Bus string array value.
fn string_array(xs: &[&str]) -> Value<'static> {
    let vals: Vec<String> = xs.iter().map(|s| s.to_string()).collect();
    Value::from(vals)
}

/// Builds the `802-11-wireless` section with SSID and mode.
fn base_wifi_section(ssid: &str) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();
    s.insert("ssid", Value::from(bytes(ssid)));
    s.insert("mode", Value::from("infrastructure"));
    s
}

/// Builds the `connection` section with type, id, uuid, and autoconnect settings.
fn base_connection_section(
    ssid: &str,
    opts: &ConnectionOptions,
) -> HashMap<&'static str, Value<'static>> {
    let mut s = HashMap::new();
    s.insert("type", Value::from("802-11-wireless"));
    s.insert("id", Value::from(ssid.to_string()));
    s.insert("uuid", Value::from(uuid::Uuid::new_v4().to_string()));
    s.insert("autoconnect", Value::from(opts.autoconnect));

    if let Some(p) = opts.autoconnect_priority {
        s.insert("autoconnect-priority", Value::from(p));
    }

    if let Some(r) = opts.autoconnect_retries {
        s.insert("autoconnect-retries", Value::from(r));
    }

    s
}

/// Builds the `802-11-wireless-security` section for WPA-PSK networks.
///
/// Uses WPA2 (RSN) with CCMP encryption. The `psk-flags` of 0 means the
/// password is stored in the connection (agent-owned).
fn build_psk_security(psk: &str) -> HashMap<&'static str, Value<'static>> {
    let mut sec = HashMap::new();

    sec.insert("key-mgmt", Value::from("wpa-psk"));
    sec.insert("psk", Value::from(psk.to_string()));
    sec.insert("psk-flags", Value::from(0u32));
    sec.insert("auth-alg", Value::from("open"));

    // Enforce WPA2 with AES
    sec.insert("proto", string_array(&["rsn"]));
    sec.insert("pairwise", string_array(&["ccmp"]));
    sec.insert("group", string_array(&["ccmp"]));

    sec
}

/// Builds security sections for WPA-EAP (802.1X) networks.
///
/// Returns both the `802-11-wireless-security` section and the `802-1x` section.
/// Supports PEAP and TTLS methods with MSCHAPv2 or PAP inner authentication.
fn build_eap_security(
    opts: &models::EapOptions,
) -> (
    HashMap<&'static str, Value<'static>>,
    HashMap<&'static str, Value<'static>>,
) {
    let mut sec = HashMap::new();
    sec.insert("key-mgmt", Value::from("wpa-eap"));
    sec.insert("auth-alg", Value::from("open"));

    let mut e1x = HashMap::new();

    // EAP method (outer authentication)
    let eap_str = match opts.method {
        EapMethod::Peap => "peap",
        EapMethod::Ttls => "ttls",
    };
    e1x.insert("eap", string_array(&[eap_str]));
    e1x.insert("identity", Value::from(opts.identity.clone()));
    e1x.insert("password", Value::from(opts.password.clone()));

    if let Some(ai) = &opts.anonymous_identity {
        e1x.insert("anonymous-identity", Value::from(ai.clone()));
    }

    // Phase 2 (inner authentication)
    let p2 = match opts.phase2 {
        models::Phase2::Mschapv2 => "mschapv2",
        models::Phase2::Pap => "pap",
    };
    e1x.insert("phase2-auth", Value::from(p2));

    // CA certificate handling for server verification
    if opts.system_ca_certs {
        e1x.insert("system-ca-certs", Value::from(true));
    }
    if let Some(cert) = &opts.ca_cert_path {
        e1x.insert("ca-cert", Value::from(cert.clone()));
    }
    if let Some(dom) = &opts.domain_suffix_match {
        e1x.insert("domain-suffix-match", Value::from(dom.clone()));
    }

    (sec, e1x)
}

/// Builds a complete Wi-Fi connection settings dictionary.
///
/// Constructs all required sections for NetworkManager based on the
/// security type. The returned dictionary can be passed directly to
/// `AddAndActivateConnection`.
///
/// # Sections Created
///
/// - `connection`: Always present
/// - `802-11-wireless`: Always present
/// - `ipv4` / `ipv6`: Always present (set to "auto" for DHCP)
/// - `802-11-wireless-security`: Present for PSK and EAP networks
/// - `802-1x`: Present only for EAP networks
pub fn build_wifi_connection(
    ssid: &str,
    security: &models::WifiSecurity,
    opts: &ConnectionOptions,
) -> HashMap<&'static str, HashMap<&'static str, Value<'static>>> {
    let mut conn: HashMap<&'static str, HashMap<&'static str, Value<'static>>> = HashMap::new();

    // base connections
    conn.insert("connection", base_connection_section(ssid, opts));
    conn.insert("802-11-wireless", base_wifi_section(ssid));

    // Add IPv4 and IPv6 configuration to prevent state 60 stall
    // TODO: Expand upon auto/manual configuration options
    let mut ipv4 = HashMap::new();
    ipv4.insert("method", Value::from("auto"));
    conn.insert("ipv4", ipv4);

    let mut ipv6 = HashMap::new();
    ipv6.insert("method", Value::from("auto"));
    conn.insert("ipv6", ipv6);

    match security {
        models::WifiSecurity::Open => {}

        models::WifiSecurity::WpaPsk { psk } => {
            // point wireless at security section
            if let Some(w) = conn.get_mut("802-11-wireless") {
                w.insert("security", Value::from("802-11-wireless-security"));
            }

            let sec = build_psk_security(psk);
            conn.insert("802-11-wireless-security", sec);
        }

        models::WifiSecurity::WpaEap { opts } => {
            if let Some(w) = conn.get_mut("802-11-wireless") {
                w.insert("security", Value::from("802-11-wireless-security"));
            }

            let (mut sec, e1x) = build_eap_security(opts);
            sec.insert("auth-alg", Value::from("open"));
            conn.insert("802-11-wireless-security", sec);
            conn.insert("802-1x", e1x);
        }
    }

    conn
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ConnectionOptions, EapOptions, Phase2, WifiSecurity};
    use zvariant::Value;

    fn default_opts() -> ConnectionOptions {
        ConnectionOptions {
            autoconnect: true,
            autoconnect_priority: None,
            autoconnect_retries: None,
        }
    }

    fn opts_with_priority() -> ConnectionOptions {
        ConnectionOptions {
            autoconnect: false,
            autoconnect_priority: Some(10),
            autoconnect_retries: Some(3),
        }
    }

    #[test]
    fn builds_open_wifi_connection() {
        let conn = build_wifi_connection("testnet", &WifiSecurity::Open, &default_opts());
        assert!(conn.contains_key("connection"));
        assert!(conn.contains_key("802-11-wireless"));
        assert!(conn.contains_key("ipv4"));
        assert!(conn.contains_key("ipv6"));
        // Open networks should NOT have security section
        assert!(!conn.contains_key("802-11-wireless-security"));
    }

    #[test]
    fn open_connection_has_correct_type() {
        let conn = build_wifi_connection("open_net", &WifiSecurity::Open, &default_opts());
        let connection_section = conn.get("connection").unwrap();
        assert_eq!(
            connection_section.get("type"),
            Some(&Value::from("802-11-wireless"))
        );
    }

    #[test]
    fn builds_psk_wifi_connection_with_security_section() {
        let conn = build_wifi_connection(
            "secure",
            &WifiSecurity::WpaPsk {
                psk: "pw123".into(),
            },
            &default_opts(),
        );
        assert!(
            conn.contains_key("802-11-wireless-security"),
            "security section missing"
        );
        let sec = conn.get("802-11-wireless-security").unwrap();
        assert_eq!(sec.get("psk"), Some(&Value::from("pw123".to_string())));
        assert_eq!(sec.get("key-mgmt"), Some(&Value::from("wpa-psk")));
    }

    #[test]
    fn psk_connection_links_wireless_to_security() {
        let conn = build_wifi_connection(
            "secure",
            &WifiSecurity::WpaPsk { psk: "test".into() },
            &default_opts(),
        );
        let wireless = conn.get("802-11-wireless").unwrap();
        assert_eq!(
            wireless.get("security"),
            Some(&Value::from("802-11-wireless-security"))
        );
    }

    #[test]
    fn builds_eap_peap_connection() {
        let eap_opts = EapOptions {
            identity: "user@example.com".into(),
            password: "secret123".into(),
            anonymous_identity: Some("anonymous@example.com".into()),
            domain_suffix_match: Some("example.com".into()),
            ca_cert_path: None,
            system_ca_certs: true,
            method: EapMethod::Peap,
            phase2: Phase2::Mschapv2,
        };
        let conn = build_wifi_connection(
            "enterprise",
            &WifiSecurity::WpaEap { opts: eap_opts },
            &default_opts(),
        );

        assert!(conn.contains_key("802-11-wireless-security"));
        assert!(conn.contains_key("802-1x"));

        let sec = conn.get("802-11-wireless-security").unwrap();
        assert_eq!(sec.get("key-mgmt"), Some(&Value::from("wpa-eap")));

        let e1x = conn.get("802-1x").unwrap();
        assert_eq!(
            e1x.get("identity"),
            Some(&Value::from("user@example.com".to_string()))
        );
        assert_eq!(
            e1x.get("password"),
            Some(&Value::from("secret123".to_string()))
        );
        assert_eq!(e1x.get("phase2-auth"), Some(&Value::from("mschapv2")));
        assert_eq!(e1x.get("system-ca-certs"), Some(&Value::from(true)));
    }

    #[test]
    fn builds_eap_ttls_connection() {
        let eap_opts = EapOptions {
            identity: "student@uni.edu".into(),
            password: "campus123".into(),
            anonymous_identity: None,
            domain_suffix_match: None,
            ca_cert_path: Some("file:///etc/ssl/certs/ca.pem".into()),
            system_ca_certs: false,
            method: EapMethod::Ttls,
            phase2: Phase2::Pap,
        };
        let conn = build_wifi_connection(
            "eduroam",
            &WifiSecurity::WpaEap { opts: eap_opts },
            &default_opts(),
        );

        let e1x = conn.get("802-1x").unwrap();
        assert_eq!(e1x.get("phase2-auth"), Some(&Value::from("pap")));
        assert_eq!(
            e1x.get("ca-cert"),
            Some(&Value::from("file:///etc/ssl/certs/ca.pem".to_string()))
        );
        // system-ca-certs should NOT be present when false
        assert!(e1x.get("system-ca-certs").is_none());
    }

    #[test]
    fn connection_with_priority_and_retries() {
        let conn =
            build_wifi_connection("priority_net", &WifiSecurity::Open, &opts_with_priority());
        let connection_section = conn.get("connection").unwrap();

        assert_eq!(
            connection_section.get("autoconnect"),
            Some(&Value::from(false))
        );
        assert_eq!(
            connection_section.get("autoconnect-priority"),
            Some(&Value::from(10i32))
        );
        assert_eq!(
            connection_section.get("autoconnect-retries"),
            Some(&Value::from(3i32))
        );
    }

    #[test]
    fn connection_without_optional_fields() {
        let conn = build_wifi_connection("simple", &WifiSecurity::Open, &default_opts());
        let connection_section = conn.get("connection").unwrap();

        assert_eq!(
            connection_section.get("autoconnect"),
            Some(&Value::from(true))
        );
        // Optional fields should not be present
        assert!(connection_section.get("autoconnect-priority").is_none());
        assert!(connection_section.get("autoconnect-retries").is_none());
    }

    #[test]
    fn ssid_is_stored_as_bytes() {
        let conn = build_wifi_connection("MyNetwork", &WifiSecurity::Open, &default_opts());
        let wireless = conn.get("802-11-wireless").unwrap();
        let ssid = wireless.get("ssid").unwrap();
        assert_eq!(ssid, &Value::from(b"MyNetwork".to_vec()));
    }

    #[test]
    fn ssid_with_special_characters() {
        let conn = build_wifi_connection("Café-Wïfì_123", &WifiSecurity::Open, &default_opts());
        let wireless = conn.get("802-11-wireless").unwrap();
        let ssid = wireless.get("ssid").unwrap();
        assert_eq!(ssid, &Value::from("Café-Wïfì_123".as_bytes().to_vec()));
    }
}
