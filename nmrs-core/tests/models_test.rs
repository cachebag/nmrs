use nmrs_core::models::*;

#[test]
fn device_type_from_u32_matches_expected() {
    assert_eq!(DeviceType::from(1), DeviceType::Ethernet);
    assert_eq!(DeviceType::from(2), DeviceType::Wifi);
    assert_eq!(DeviceType::from(999), DeviceType::Other(999));
}

#[test]
fn device_state_from_u32_matches_expected() {
    assert_eq!(DeviceState::from(100), DeviceState::Activated);
    assert_eq!(DeviceState::from(120), DeviceState::Failed);
    assert_eq!(DeviceState::from(7), DeviceState::Other(7));
}

#[test]
fn wifi_security_flags_are_correct() {
    let open = WifiSecurity::Open;
    let psk = WifiSecurity::WpaPsk { psk: "abc".into() };
    assert!(!open.secured());
    assert!(psk.secured());
    assert!(psk.is_psk());
    assert!(!psk.is_eap());
}
