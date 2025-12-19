//! Bluetooth Device Proxy

use zbus::proxy;
use zbus::Result;

/// Proxy for Bluetooth devices
///
/// Provides access to Bluetooth-specific properties and methods.
#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Bluetooth",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NMBluetooth {
    /// Bluetooth MAC address of the device.
    #[zbus(property)]
    fn bd_address(&self) -> Result<String>;

    /// Bluetooth capabilities of the device (either DUN or NAP).
    #[zbus(property)]
    fn bt_capabilities(&self) -> Result<u32>;
}

/// Extension trait for Bluetooth device information via BlueZ.
/// Provides convenient methods to access Bluetooth-specific properties otherwise
/// not exposed by NetworkManager.
#[proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
pub trait BluezDeviceExt {
    /// Returns the name of the Bluetooth device.
    #[zbus(property)]
    fn name(&self) -> Result<String>;

    /// Returns the alias of the Bluetooth device.
    #[zbus(property)]
    fn alias(&self) -> Result<String>;
}
