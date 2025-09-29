use netrs_core::models::{ConnectionError, Device, DeviceState, DeviceType};

fn wifi_ready_condition(device: &Device) -> bool {
    device.device_type == DeviceType::Wifi
        && (device.state == DeviceState::Disconnected || device.state == DeviceState::Activated)
}

fn decode_ssid(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) if \!s.is_empty() => s.to_string(),
        _ => "<Hidden Network>".to_string(),
    }
}

#[test]
fn device_type_from_known_values() {
    assert_eq\!(DeviceType::from(1), DeviceType::Ethernet);
    assert_eq\!(DeviceType::from(2), DeviceType::Wifi);
    assert_eq\!(DeviceType::from(30), DeviceType::WifiP2P);
    assert_eq\!(DeviceType::from(32), DeviceType::Loopback);
}

#[test]
fn device_type_from_unknown_value() {
    assert_eq\!(DeviceType::from(77), DeviceType::Other(77));
}

#[test]
fn device_state_from_known_values() {
    assert_eq\!(DeviceState::from(10), DeviceState::Unmanaged);
    assert_eq\!(DeviceState::from(20), DeviceState::Unavailable);
    assert_eq\!(DeviceState::from(30), DeviceState::Disconnected);
    assert_eq\!(DeviceState::from(40), DeviceState::Prepare);
    assert_eq\!(DeviceState::from(50), DeviceState::Config);
    assert_eq\!(DeviceState::from(100), DeviceState::Activated);
}

#[test]
fn device_state_from_unknown_value() {
    assert_eq\!(DeviceState::from(99), DeviceState::Other(99));
}

#[test]
fn device_type_display_is_human_friendly() {
    assert_eq\!(DeviceType::Ethernet.to_string(), "Ethernet");
    assert_eq\!(DeviceType::Wifi.to_string(), "Wi-Fi");
    assert_eq\!(DeviceType::WifiP2P.to_string(), "Wi-Fi P2P");
    assert_eq\!(DeviceType::Loopback.to_string(), "Loopback");
    assert_eq\!(DeviceType::Other(42).to_string(), "Other(42)");
}

#[test]
fn device_state_display_is_human_friendly() {
    assert_eq\!(DeviceState::Unmanaged.to_string(), "Unmanaged");
    assert_eq\!(DeviceState::Unavailable.to_string(), "Unavailable");
    assert_eq\!(DeviceState::Disconnected.to_string(), "Disconnected");
    assert_eq\!(DeviceState::Prepare.to_string(), "Preparing");
    assert_eq\!(DeviceState::Config.to_string(), "Configuring");
    assert_eq\!(DeviceState::Activated.to_string(), "Activated");
    assert_eq\!(DeviceState::Other(73).to_string(), "Other(73)");
}

#[test]
fn connection_error_wraps_zbus_error_message() {
    let message = "Wi-Fi device never became ready";
    let failure = zbus::Error::Failure(message.into());
    let err: ConnectionError = failure.into();
    assert\!(matches\!(err, ConnectionError::Dbus(_)));
    assert\!(
        err.to_string().contains(message),
        "Expected error message to contain '{message}', got '{}'",
        err
    );
}

#[test]
fn wifi_ready_condition_accepts_valid_states() {
    let wifi_disconnected = Device {
        path: "/dev/mock-wifi0".into(),
        interface: "wlan0".into(),
        device_type: DeviceType::Wifi,
        state: DeviceState::Disconnected,
        managed: Some(true),
        driver: Some("mock".into()),
    };

    let wifi_activated = Device {
        path: "/dev/mock-wifi1".into(),
        interface: "wlan1".into(),
        device_type: DeviceType::Wifi,
        state: DeviceState::Activated,
        managed: Some(false),
        driver: None,
    };

    assert\!(wifi_ready_condition(&wifi_disconnected));
    assert\!(wifi_ready_condition(&wifi_activated));
}

#[test]
fn wifi_ready_condition_rejects_non_wifi_or_wrong_state() {
    let ethernet_device = Device {
        path: "/dev/mock-eth0".into(),
        interface: "eth0".into(),
        device_type: DeviceType::Ethernet,
        state: DeviceState::Activated,
        managed: Some(true),
        driver: Some("e1000".into()),
    };

    let wifi_preparing = Device {
        path: "/dev/mock-wifi2".into(),
        interface: "wlan2".into(),
        device_type: DeviceType::Wifi,
        state: DeviceState::Prepare,
        managed: None,
        driver: None,
    };

    assert\!(\!wifi_ready_condition(&ethernet_device));
    assert\!(\!wifi_ready_condition(&wifi_preparing));
}

#[test]
fn decode_ssid_returns_visible_name_for_valid_utf8() {
    let ssid = b"MyNetwork";
    assert_eq\!(decode_ssid(ssid), "MyNetwork");
}

#[test]
fn decode_ssid_labels_empty_bytes_as_hidden_network() {
    let empty: &[u8] = &[];
    assert_eq\!(decode_ssid(empty), "<Hidden Network>");
}

#[test]
fn decode_ssid_labels_invalid_utf8_as_hidden_network() {
    let invalid_utf8: &[u8] = &[0xff, 0xfe, 0xfd];
    assert_eq\!(decode_ssid(invalid_utf8), "<Hidden Network>");
}