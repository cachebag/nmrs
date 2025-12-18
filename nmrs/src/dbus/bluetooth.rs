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
    /// Bluetooth name of device.
    #[zbus(property)]
    fn name(&self) -> Result<String>;

    /// Bluetooth capabilities of the device (either DUN or NAP).
    #[zbus(property)]
    fn bt_capabilities(&self) -> Result<u32>;
}
