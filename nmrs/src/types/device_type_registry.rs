//! Device type registry for extensible device type support.
//!
//! This module provides a trait-based system for registering and working with
//! different network device types. It enables adding new device types without
//! breaking the public API.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Trait for device type-specific behavior.
///
/// Implement this trait to add support for a new device type.
/// The trait provides metadata about the device type and type-specific
/// operations that may be needed.
pub trait DeviceTypeInfo: Send + Sync {
    /// Returns the NetworkManager D-Bus constant for this device type.
    fn nm_type_code(&self) -> u32;

    /// Returns the human-readable name of this device type.
    fn display_name(&self) -> &'static str;

    /// Returns the NetworkManager connection type string.
    ///
    /// This is used when creating connections for this device type.
    /// Examples: "802-11-wireless", "802-3-ethernet", "wireguard", "bluetooth"
    fn connection_type(&self) -> &'static str;

    /// Returns whether this device type supports scanning for networks.
    fn supports_scanning(&self) -> bool {
        false
    }

    /// Returns whether this device type requires an access point or similar target.
    fn requires_specific_object(&self) -> bool {
        false
    }

    /// Returns whether this device type can be globally enabled/disabled.
    fn has_global_enabled_state(&self) -> bool {
        false
    }
}

/// WiFi device type implementation.
struct WifiDeviceType;

impl DeviceTypeInfo for WifiDeviceType {
    fn nm_type_code(&self) -> u32 {
        2
    }

    fn display_name(&self) -> &'static str {
        "Wi-Fi"
    }

    fn connection_type(&self) -> &'static str {
        "802-11-wireless"
    }

    fn supports_scanning(&self) -> bool {
        true
    }

    fn requires_specific_object(&self) -> bool {
        true
    }

    fn has_global_enabled_state(&self) -> bool {
        true
    }
}

/// Ethernet device type implementation.
struct EthernetDeviceType;

impl DeviceTypeInfo for EthernetDeviceType {
    fn nm_type_code(&self) -> u32 {
        1
    }

    fn display_name(&self) -> &'static str {
        "Ethernet"
    }

    fn connection_type(&self) -> &'static str {
        "802-3-ethernet"
    }
}

/// WiFi P2P device type implementation.
struct WifiP2PDeviceType;

impl DeviceTypeInfo for WifiP2PDeviceType {
    fn nm_type_code(&self) -> u32 {
        30
    }

    fn display_name(&self) -> &'static str {
        "Wi-Fi P2P"
    }

    fn connection_type(&self) -> &'static str {
        "wifi-p2p"
    }

    fn supports_scanning(&self) -> bool {
        true
    }
}

/// Loopback device type implementation.
struct LoopbackDeviceType;

impl DeviceTypeInfo for LoopbackDeviceType {
    fn nm_type_code(&self) -> u32 {
        32
    }

    fn display_name(&self) -> &'static str {
        "Loopback"
    }

    fn connection_type(&self) -> &'static str {
        "loopback"
    }
}

/// Bridge device type implementation.
struct BridgeDeviceType;

impl DeviceTypeInfo for BridgeDeviceType {
    fn nm_type_code(&self) -> u32 {
        13
    }

    fn display_name(&self) -> &'static str {
        "Bridge"
    }

    fn connection_type(&self) -> &'static str {
        "bridge"
    }
}

/// Bond device type implementation.
struct BondDeviceType;

impl DeviceTypeInfo for BondDeviceType {
    fn nm_type_code(&self) -> u32 {
        12
    }

    fn display_name(&self) -> &'static str {
        "Bond"
    }

    fn connection_type(&self) -> &'static str {
        "bond"
    }
}

/// VLAN device type implementation.
struct VlanDeviceType;

impl DeviceTypeInfo for VlanDeviceType {
    fn nm_type_code(&self) -> u32 {
        11
    }

    fn display_name(&self) -> &'static str {
        "VLAN"
    }

    fn connection_type(&self) -> &'static str {
        "vlan"
    }
}

/// TUN/TAP device type implementation.
struct TunDeviceType;

impl DeviceTypeInfo for TunDeviceType {
    fn nm_type_code(&self) -> u32 {
        16
    }

    fn display_name(&self) -> &'static str {
        "TUN"
    }

    fn connection_type(&self) -> &'static str {
        "tun"
    }
}

/// WireGuard device type implementation.
struct WireGuardDeviceType;

impl DeviceTypeInfo for WireGuardDeviceType {
    fn nm_type_code(&self) -> u32 {
        29
    }

    fn display_name(&self) -> &'static str {
        "WireGuard"
    }

    fn connection_type(&self) -> &'static str {
        "wireguard"
    }
}

/// Global registry of device types.
///
/// This registry maps NetworkManager type codes to device type information.
/// It's populated once at first access and remains immutable thereafter.
static DEVICE_TYPE_REGISTRY: OnceLock<HashMap<u32, Box<dyn DeviceTypeInfo>>> = OnceLock::new();

/// Initializes and returns the device type registry.
fn registry() -> &'static HashMap<u32, Box<dyn DeviceTypeInfo>> {
    DEVICE_TYPE_REGISTRY.get_or_init(|| {
        let mut map: HashMap<u32, Box<dyn DeviceTypeInfo>> = HashMap::new();

        let types: Vec<Box<dyn DeviceTypeInfo>> = vec![
            Box::new(EthernetDeviceType),
            Box::new(WifiDeviceType),
            Box::new(WifiP2PDeviceType),
            Box::new(LoopbackDeviceType),
            Box::new(BridgeDeviceType),
            Box::new(BondDeviceType),
            Box::new(VlanDeviceType),
            Box::new(TunDeviceType),
            Box::new(WireGuardDeviceType),
        ];

        for type_info in types {
            map.insert(type_info.nm_type_code(), type_info);
        }

        map
    })
}

/// Looks up device type information by NetworkManager type code.
///
/// Returns `None` if the device type is not recognized.
pub fn get_device_type_info(code: u32) -> Option<&'static dyn DeviceTypeInfo> {
    registry().get(&code).map(|b| &**b)
}

/// Returns the display name for a device type code.
///
/// If the code is not recognized, returns a generic "Other(N)" string.
pub fn display_name_for_code(code: u32) -> String {
    get_device_type_info(code)
        .map(|info| info.display_name().to_string())
        .unwrap_or_else(|| format!("Other({})", code))
}

/// Returns the connection type string for a device type code.
///
/// Returns `None` if the device type is not recognized or doesn't
/// have an associated connection type.
pub fn connection_type_for_code(code: u32) -> Option<&'static str> {
    get_device_type_info(code).map(|info| info.connection_type())
}

/// Returns whether a device type supports scanning.
pub fn supports_scanning(code: u32) -> bool {
    get_device_type_info(code)
        .map(|info| info.supports_scanning())
        .unwrap_or(false)
}

/// Returns whether a device type requires a specific object (like an AP).
pub fn requires_specific_object(code: u32) -> bool {
    get_device_type_info(code)
        .map(|info| info.requires_specific_object())
        .unwrap_or(false)
}

/// Returns whether a device type has a global enabled state.
pub fn has_global_enabled_state(code: u32) -> bool {
    get_device_type_info(code)
        .map(|info| info.has_global_enabled_state())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wifi_type_info() {
        let info = get_device_type_info(2).expect("WiFi should be registered");
        assert_eq!(info.nm_type_code(), 2);
        assert_eq!(info.display_name(), "Wi-Fi");
        assert_eq!(info.connection_type(), "802-11-wireless");
        assert!(info.supports_scanning());
        assert!(info.requires_specific_object());
        assert!(info.has_global_enabled_state());
    }

    #[test]
    fn ethernet_type_info() {
        let info = get_device_type_info(1).expect("Ethernet should be registered");
        assert_eq!(info.nm_type_code(), 1);
        assert_eq!(info.display_name(), "Ethernet");
        assert_eq!(info.connection_type(), "802-3-ethernet");
        assert!(!info.supports_scanning());
        assert!(!info.requires_specific_object());
    }

    #[test]
    fn wireguard_type_info() {
        let info = get_device_type_info(29).expect("WireGuard should be registered");
        assert_eq!(info.nm_type_code(), 29);
        assert_eq!(info.display_name(), "WireGuard");
        assert_eq!(info.connection_type(), "wireguard");
    }

    #[test]
    fn unknown_device_type() {
        let info = get_device_type_info(999);
        assert!(info.is_none());
    }

    #[test]
    fn display_name_for_unknown() {
        let name = display_name_for_code(999);
        assert_eq!(name, "Other(999)");
    }

    #[test]
    fn wifi_supports_scanning() {
        assert!(supports_scanning(2));
        assert!(!supports_scanning(1));
    }

    #[test]
    fn wifi_requires_specific_object() {
        assert!(requires_specific_object(2));
        assert!(!requires_specific_object(1));
    }

    #[test]
    fn wifi_has_global_enabled_state() {
        assert!(has_global_enabled_state(2));
        assert!(!has_global_enabled_state(1));
    }

    #[test]
    fn all_registered_types_have_connection_type() {
        for code in [1u32, 2, 11, 12, 13, 16, 29, 30, 32] {
            let conn_type = connection_type_for_code(code);
            assert!(
                conn_type.is_some(),
                "Device type {} should have a connection type",
                code
            );
        }
    }

    #[test]
    fn registry_is_consistent() {
        let reg = registry();
        for (code, type_info) in reg.iter() {
            assert_eq!(
                *code,
                type_info.nm_type_code(),
                "Registry key must match type code"
            );
        }
    }
}
