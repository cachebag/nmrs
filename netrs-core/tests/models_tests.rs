use netrs_core::models::{ConnectionError, Device, DeviceState, DeviceType, Network};
use serde_json;

#[cfg(test)]
mod network_tests {
    use super::*;

    #[test]
    fn test_network_creation_with_all_fields() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("TestNetwork"),
            bssid: Some(String::from("00:11:22:33:44:55")),
            strength: Some(85),
        };

        assert_eq\!(network.device, "wlan0");
        assert_eq\!(network.ssid, "TestNetwork");
        assert_eq\!(network.bssid, Some(String::from("00:11:22:33:44:55")));
        assert_eq\!(network.strength, Some(85));
    }

    #[test]
    fn test_network_creation_with_none_optional_fields() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("TestNetwork"),
            bssid: None,
            strength: None,
        };

        assert_eq\!(network.device, "wlan0");
        assert_eq\!(network.ssid, "TestNetwork");
        assert\!(network.bssid.is_none());
        assert\!(network.strength.is_none());
    }

    #[test]
    fn test_network_with_empty_ssid() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from(""),
            bssid: None,
            strength: None,
        };

        assert_eq\!(network.ssid, "");
    }

    #[test]
    fn test_network_with_max_strength() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("StrongNetwork"),
            bssid: Some(String::from("AA:BB:CC:DD:EE:FF")),
            strength: Some(100),
        };

        assert_eq\!(network.strength, Some(100));
    }

    #[test]
    fn test_network_with_zero_strength() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("WeakNetwork"),
            bssid: Some(String::from("11:22:33:44:55:66")),
            strength: Some(0),
        };

        assert_eq\!(network.strength, Some(0));
    }

    #[test]
    fn test_network_clone() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("TestNetwork"),
            bssid: Some(String::from("00:11:22:33:44:55")),
            strength: Some(75),
        };

        let cloned = network.clone();
        assert_eq\!(cloned.device, network.device);
        assert_eq\!(cloned.ssid, network.ssid);
        assert_eq\!(cloned.bssid, network.bssid);
        assert_eq\!(cloned.strength, network.strength);
    }

    #[test]
    fn test_network_serialize() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("TestNetwork"),
            bssid: Some(String::from("00:11:22:33:44:55")),
            strength: Some(85),
        };

        let serialized = serde_json::to_string(&network).unwrap();
        assert\!(serialized.contains("wlan0"));
        assert\!(serialized.contains("TestNetwork"));
        assert\!(serialized.contains("00:11:22:33:44:55"));
        assert\!(serialized.contains("85"));
    }

    #[test]
    fn test_network_deserialize() {
        let json = r#"{"device":"wlan0","ssid":"TestNetwork","bssid":"00:11:22:33:44:55","strength":85}"#;
        let network: Network = serde_json::from_str(json).unwrap();

        assert_eq\!(network.device, "wlan0");
        assert_eq\!(network.ssid, "TestNetwork");
        assert_eq\!(network.bssid, Some(String::from("00:11:22:33:44:55")));
        assert_eq\!(network.strength, Some(85));
    }

    #[test]
    fn test_network_deserialize_with_null_optional_fields() {
        let json = r#"{"device":"wlan0","ssid":"TestNetwork","bssid":null,"strength":null}"#;
        let network: Network = serde_json::from_str(json).unwrap();

        assert_eq\!(network.device, "wlan0");
        assert_eq\!(network.ssid, "TestNetwork");
        assert\!(network.bssid.is_none());
        assert\!(network.strength.is_none());
    }

    #[test]
    fn test_network_serialize_deserialize_roundtrip() {
        let original = Network {
            device: String::from("wlan0"),
            ssid: String::from("TestNetwork"),
            bssid: Some(String::from("00:11:22:33:44:55")),
            strength: Some(85),
        };

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Network = serde_json::from_str(&serialized).unwrap();

        assert_eq\!(deserialized.device, original.device);
        assert_eq\!(deserialized.ssid, original.ssid);
        assert_eq\!(deserialized.bssid, original.bssid);
        assert_eq\!(deserialized.strength, original.strength);
    }

    #[test]
    fn test_network_with_unicode_ssid() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("测试网络"),
            bssid: None,
            strength: Some(50),
        };

        assert_eq\!(network.ssid, "测试网络");
    }

    #[test]
    fn test_network_with_special_characters_in_ssid() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("Test-Network_123\!@#"),
            bssid: None,
            strength: Some(60),
        };

        assert_eq\!(network.ssid, "Test-Network_123\!@#");
    }
}

#[cfg(test)]
mod device_tests {
    use super::*;

    #[test]
    fn test_device_creation_with_all_fields() {
        let device = Device {
            path: String::from("/org/freedesktop/NetworkManager/Devices/1"),
            interface: String::from("wlan0"),
            device_type: DeviceType::Wifi,
            state: DeviceState::Activated,
            managed: Some(true),
            driver: Some(String::from("iwlwifi")),
        };

        assert_eq\!(device.path, "/org/freedesktop/NetworkManager/Devices/1");
        assert_eq\!(device.interface, "wlan0");
        assert_eq\!(device.device_type, DeviceType::Wifi);
        assert_eq\!(device.state, DeviceState::Activated);
        assert_eq\!(device.managed, Some(true));
        assert_eq\!(device.driver, Some(String::from("iwlwifi")));
    }

    #[test]
    fn test_device_creation_with_none_optional_fields() {
        let device = Device {
            path: String::from("/org/freedesktop/NetworkManager/Devices/2"),
            interface: String::from("eth0"),
            device_type: DeviceType::Ethernet,
            state: DeviceState::Disconnected,
            managed: None,
            driver: None,
        };

        assert\!(device.managed.is_none());
        assert\!(device.driver.is_none());
    }

    #[test]
    fn test_device_with_ethernet_type() {
        let device = Device {
            path: String::from("/org/freedesktop/NetworkManager/Devices/3"),
            interface: String::from("eth0"),
            device_type: DeviceType::Ethernet,
            state: DeviceState::Activated,
            managed: Some(true),
            driver: Some(String::from("e1000e")),
        };

        assert_eq\!(device.device_type, DeviceType::Ethernet);
    }

    #[test]
    fn test_device_with_loopback_type() {
        let device = Device {
            path: String::from("/org/freedesktop/NetworkManager/Devices/4"),
            interface: String::from("lo"),
            device_type: DeviceType::Loopback,
            state: DeviceState::Unmanaged,
            managed: Some(false),
            driver: None,
        };

        assert_eq\!(device.device_type, DeviceType::Loopback);
    }

    #[test]
    fn test_device_with_wifi_p2p_type() {
        let device = Device {
            path: String::from("/org/freedesktop/NetworkManager/Devices/5"),
            interface: String::from("p2p-dev-wlan0"),
            device_type: DeviceType::WifiP2P,
            state: DeviceState::Unavailable,
            managed: Some(false),
            driver: Some(String::from("iwlwifi")),
        };

        assert_eq\!(device.device_type, DeviceType::WifiP2P);
    }

    #[test]
    fn test_device_clone() {
        let device = Device {
            path: String::from("/org/freedesktop/NetworkManager/Devices/1"),
            interface: String::from("wlan0"),
            device_type: DeviceType::Wifi,
            state: DeviceState::Activated,
            managed: Some(true),
            driver: Some(String::from("iwlwifi")),
        };

        let cloned = device.clone();
        assert_eq\!(cloned.path, device.path);
        assert_eq\!(cloned.interface, device.interface);
        assert_eq\!(cloned.device_type, device.device_type);
        assert_eq\!(cloned.state, device.state);
        assert_eq\!(cloned.managed, device.managed);
        assert_eq\!(cloned.driver, device.driver);
    }
}

#[cfg(test)]
mod device_type_tests {
    use super::*;

    #[test]
    fn test_device_type_from_u32_ethernet() {
        let device_type = DeviceType::from(1);
        assert_eq\!(device_type, DeviceType::Ethernet);
    }

    #[test]
    fn test_device_type_from_u32_wifi() {
        let device_type = DeviceType::from(2);
        assert_eq\!(device_type, DeviceType::Wifi);
    }

    #[test]
    fn test_device_type_from_u32_wifi_p2p() {
        let device_type = DeviceType::from(30);
        assert_eq\!(device_type, DeviceType::WifiP2P);
    }

    #[test]
    fn test_device_type_from_u32_loopback() {
        let device_type = DeviceType::from(32);
        assert_eq\!(device_type, DeviceType::Loopback);
    }

    #[test]
    fn test_device_type_from_u32_other() {
        let device_type = DeviceType::from(999);
        assert_eq\!(device_type, DeviceType::Other(999));
    }

    #[test]
    fn test_device_type_from_u32_zero() {
        let device_type = DeviceType::from(0);
        assert_eq\!(device_type, DeviceType::Other(0));
    }

    #[test]
    fn test_device_type_from_u32_max() {
        let device_type = DeviceType::from(u32::MAX);
        assert_eq\!(device_type, DeviceType::Other(u32::MAX));
    }

    #[test]
    fn test_device_type_from_u32_boundary_before_wifi_p2p() {
        let device_type = DeviceType::from(29);
        assert_eq\!(device_type, DeviceType::Other(29));
    }

    #[test]
    fn test_device_type_from_u32_boundary_after_wifi_p2p() {
        let device_type = DeviceType::from(31);
        assert_eq\!(device_type, DeviceType::Other(31));
    }

    #[test]
    fn test_device_type_display_ethernet() {
        let device_type = DeviceType::Ethernet;
        assert_eq\!(format\!("{}", device_type), "Ethernet");
    }

    #[test]
    fn test_device_type_display_wifi() {
        let device_type = DeviceType::Wifi;
        assert_eq\!(format\!("{}", device_type), "Wi-Fi");
    }

    #[test]
    fn test_device_type_display_wifi_p2p() {
        let device_type = DeviceType::WifiP2P;
        assert_eq\!(format\!("{}", device_type), "Wi-Fi P2P");
    }

    #[test]
    fn test_device_type_display_loopback() {
        let device_type = DeviceType::Loopback;
        assert_eq\!(format\!("{}", device_type), "Loopback");
    }

    #[test]
    fn test_device_type_display_other() {
        let device_type = DeviceType::Other(42);
        assert_eq\!(format\!("{}", device_type), "Other(42)");
    }

    #[test]
    fn test_device_type_display_other_zero() {
        let device_type = DeviceType::Other(0);
        assert_eq\!(format\!("{}", device_type), "Other(0)");
    }

    #[test]
    fn test_device_type_equality() {
        assert_eq\!(DeviceType::Ethernet, DeviceType::Ethernet);
        assert_eq\!(DeviceType::Wifi, DeviceType::Wifi);
        assert_eq\!(DeviceType::Other(42), DeviceType::Other(42));
    }

    #[test]
    fn test_device_type_inequality() {
        assert_ne\!(DeviceType::Ethernet, DeviceType::Wifi);
        assert_ne\!(DeviceType::Other(42), DeviceType::Other(43));
        assert_ne\!(DeviceType::Wifi, DeviceType::WifiP2P);
    }

    #[test]
    fn test_device_type_clone() {
        let original = DeviceType::Wifi;
        let cloned = original.clone();
        assert_eq\!(original, cloned);

        let original_other = DeviceType::Other(123);
        let cloned_other = original_other.clone();
        assert_eq\!(original_other, cloned_other);
    }
}

#[cfg(test)]
mod device_state_tests {
    use super::*;

    #[test]
    fn test_device_state_from_u32_unmanaged() {
        let state = DeviceState::from(10);
        assert_eq\!(state, DeviceState::Unmanaged);
    }

    #[test]
    fn test_device_state_from_u32_unavailable() {
        let state = DeviceState::from(20);
        assert_eq\!(state, DeviceState::Unavailable);
    }

    #[test]
    fn test_device_state_from_u32_disconnected() {
        let state = DeviceState::from(30);
        assert_eq\!(state, DeviceState::Disconnected);
    }

    #[test]
    fn test_device_state_from_u32_prepare() {
        let state = DeviceState::from(40);
        assert_eq\!(state, DeviceState::Prepare);
    }

    #[test]
    fn test_device_state_from_u32_config() {
        let state = DeviceState::from(50);
        assert_eq\!(state, DeviceState::Config);
    }

    #[test]
    fn test_device_state_from_u32_activated() {
        let state = DeviceState::from(100);
        assert_eq\!(state, DeviceState::Activated);
    }

    #[test]
    fn test_device_state_from_u32_other() {
        let state = DeviceState::from(999);
        assert_eq\!(state, DeviceState::Other(999));
    }

    #[test]
    fn test_device_state_from_u32_zero() {
        let state = DeviceState::from(0);
        assert_eq\!(state, DeviceState::Other(0));
    }

    #[test]
    fn test_device_state_from_u32_max() {
        let state = DeviceState::from(u32::MAX);
        assert_eq\!(state, DeviceState::Other(u32::MAX));
    }

    #[test]
    fn test_device_state_from_u32_boundary_values() {
        assert_eq\!(DeviceState::from(9), DeviceState::Other(9));
        assert_eq\!(DeviceState::from(11), DeviceState::Other(11));
        assert_eq\!(DeviceState::from(19), DeviceState::Other(19));
        assert_eq\!(DeviceState::from(21), DeviceState::Other(21));
        assert_eq\!(DeviceState::from(99), DeviceState::Other(99));
        assert_eq\!(DeviceState::from(101), DeviceState::Other(101));
    }

    #[test]
    fn test_device_state_display_unmanaged() {
        let state = DeviceState::Unmanaged;
        assert_eq\!(format\!("{}", state), "Unmanaged");
    }

    #[test]
    fn test_device_state_display_unavailable() {
        let state = DeviceState::Unavailable;
        assert_eq\!(format\!("{}", state), "Unavailable");
    }

    #[test]
    fn test_device_state_display_disconnected() {
        let state = DeviceState::Disconnected;
        assert_eq\!(format\!("{}", state), "Disconnected");
    }

    #[test]
    fn test_device_state_display_prepare() {
        let state = DeviceState::Prepare;
        assert_eq\!(format\!("{}", state), "Preparing");
    }

    #[test]
    fn test_device_state_display_config() {
        let state = DeviceState::Config;
        assert_eq\!(format\!("{}", state), "Configuring");
    }

    #[test]
    fn test_device_state_display_activated() {
        let state = DeviceState::Activated;
        assert_eq\!(format\!("{}", state), "Activated");
    }

    #[test]
    fn test_device_state_display_other() {
        let state = DeviceState::Other(75);
        assert_eq\!(format\!("{}", state), "Other(75)");
    }

    #[test]
    fn test_device_state_display_other_zero() {
        let state = DeviceState::Other(0);
        assert_eq\!(format\!("{}", state), "Other(0)");
    }

    #[test]
    fn test_device_state_equality() {
        assert_eq\!(DeviceState::Unmanaged, DeviceState::Unmanaged);
        assert_eq\!(DeviceState::Activated, DeviceState::Activated);
        assert_eq\!(DeviceState::Other(42), DeviceState::Other(42));
    }

    #[test]
    fn test_device_state_inequality() {
        assert_ne\!(DeviceState::Unmanaged, DeviceState::Unavailable);
        assert_ne\!(DeviceState::Activated, DeviceState::Disconnected);
        assert_ne\!(DeviceState::Other(42), DeviceState::Other(43));
    }

    #[test]
    fn test_device_state_clone() {
        let original = DeviceState::Activated;
        let cloned = original.clone();
        assert_eq\!(original, cloned);

        let original_other = DeviceState::Other(456);
        let cloned_other = original_other.clone();
        assert_eq\!(original_other, cloned_other);
    }
}

#[cfg(test)]
mod connection_error_tests {
    use super::*;

    #[test]
    fn test_connection_error_not_found_display() {
        let error = ConnectionError::NotFound;
        assert_eq\!(format\!("{}", error), "Network not found");
    }

    #[test]
    fn test_connection_error_auth_failed_display() {
        let error = ConnectionError::AuthFailed;
        assert_eq\!(format\!("{}", error), "Authentication failed");
    }

    #[test]
    fn test_connection_error_debug_format() {
        let error = ConnectionError::NotFound;
        let debug_str = format\!("{:?}", error);
        assert\!(debug_str.contains("NotFound"));
    }

    #[test]
    fn test_connection_error_is_error_trait() {
        let error = ConnectionError::NotFound;
        let _error_trait: &dyn std::error::Error = &error;
    }

    #[test]
    fn test_connection_error_not_found_source_is_none() {
        use std::error::Error;
        let error = ConnectionError::NotFound;
        assert\!(error.source().is_none());
    }

    #[test]
    fn test_connection_error_auth_failed_source_is_none() {
        use std::error::Error;
        let error = ConnectionError::AuthFailed;
        assert\!(error.source().is_none());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_device_workflow() {
        // Create a device from u32 values
        let device_type = DeviceType::from(2);
        let device_state = DeviceState::from(100);

        let device = Device {
            path: String::from("/org/freedesktop/NetworkManager/Devices/1"),
            interface: String::from("wlan0"),
            device_type,
            state: device_state,
            managed: Some(true),
            driver: Some(String::from("iwlwifi")),
        };

        // Verify device is properly configured
        assert_eq\!(device.device_type, DeviceType::Wifi);
        assert_eq\!(device.state, DeviceState::Activated);
        assert_eq\!(format\!("{}", device.device_type), "Wi-Fi");
        assert_eq\!(format\!("{}", device.state), "Activated");
    }

    #[test]
    fn test_network_with_associated_device_type() {
        let network = Network {
            device: String::from("wlan0"),
            ssid: String::from("TestNetwork"),
            bssid: Some(String::from("00:11:22:33:44:55")),
            strength: Some(85),
        };

        let device_type = DeviceType::from(2); // Wifi
        assert_eq\!(device_type, DeviceType::Wifi);
        assert_eq\!(network.device, "wlan0");
    }

    #[test]
    fn test_device_state_transitions() {
        let states = vec\![
            (10, DeviceState::Unmanaged, "Unmanaged"),
            (20, DeviceState::Unavailable, "Unavailable"),
            (30, DeviceState::Disconnected, "Disconnected"),
            (40, DeviceState::Prepare, "Preparing"),
            (50, DeviceState::Config, "Configuring"),
            (100, DeviceState::Activated, "Activated"),
        ];

        for (value, expected_state, expected_display) in states {
            let state = DeviceState::from(value);
            assert_eq\!(state, expected_state);
            assert_eq\!(format\!("{}", state), expected_display);
        }
    }

    #[test]
    fn test_device_type_mapping() {
        let types = vec\![
            (1, DeviceType::Ethernet, "Ethernet"),
            (2, DeviceType::Wifi, "Wi-Fi"),
            (30, DeviceType::WifiP2P, "Wi-Fi P2P"),
            (32, DeviceType::Loopback, "Loopback"),
        ];

        for (value, expected_type, expected_display) in types {
            let device_type = DeviceType::from(value);
            assert_eq\!(device_type, expected_type);
            assert_eq\!(format\!("{}", device_type), expected_display);
        }
    }

    #[test]
    fn test_network_serialization_with_all_variants() {
        let networks = vec\![
            Network {
                device: String::from("wlan0"),
                ssid: String::from("Network1"),
                bssid: Some(String::from("AA:BB:CC:DD:EE:FF")),
                strength: Some(90),
            },
            Network {
                device: String::from("eth0"),
                ssid: String::from("Network2"),
                bssid: None,
                strength: None,
            },
            Network {
                device: String::from("wlan1"),
                ssid: String::from(""),
                bssid: Some(String::from("11:22:33:44:55:66")),
                strength: Some(0),
            },
        ];

        for network in networks {
            let serialized = serde_json::to_string(&network).unwrap();
            let deserialized: Network = serde_json::from_str(&serialized).unwrap();
            assert_eq\!(network.device, deserialized.device);
            assert_eq\!(network.ssid, deserialized.ssid);
            assert_eq\!(network.bssid, deserialized.bssid);
            assert_eq\!(network.strength, deserialized.strength);
        }
    }

    #[test]
    fn test_edge_case_strength_values() {
        let edge_cases = vec\![0u8, 1, 50, 99, 100, u8::MAX];

        for strength_value in edge_cases {
            let network = Network {
                device: String::from("wlan0"),
                ssid: String::from("EdgeCaseNetwork"),
                bssid: None,
                strength: Some(strength_value),
            };
            assert_eq\!(network.strength, Some(strength_value));
        }
    }

    #[test]
    fn test_error_types_are_distinct() {
        let not_found = ConnectionError::NotFound;
        let auth_failed = ConnectionError::AuthFailed;

        assert_ne\!(format\!("{}", not_found), format\!("{}", auth_failed));
    }
}