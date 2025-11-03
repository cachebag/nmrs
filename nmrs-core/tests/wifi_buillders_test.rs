use nmrs_core::models::WifiSecurity;
use nmrs_core::wifi_builders::build_wifi_connection;
use zvariant::Value;

#[test]
fn builds_open_wifi_connection() {
    let conn = build_wifi_connection("testnet", &WifiSecurity::Open);
    assert!(conn.contains_key("connection"));
    assert!(conn.contains_key("802-11-wireless"));
    assert!(conn.contains_key("ipv4"));
    assert!(conn.contains_key("ipv6"));
}

#[test]
fn builds_psk_wifi_connection_with_security_section() {
    let conn = build_wifi_connection(
        "secure",
        &WifiSecurity::WpaPsk {
            psk: "pw123".into(),
        },
    );
    let has_sec = conn.contains_key("802-11-wireless-security");
    assert!(has_sec, "security section missing");
    let sec = conn.get("802-11-wireless-security").unwrap();
    assert_eq!(sec.get("psk"), Some(&Value::from("pw123".to_string())));
}
