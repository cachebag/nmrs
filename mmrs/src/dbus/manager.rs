//! ModemManager manager proxy.

use zbus::proxy;

/// Proxy for the main ModemManager1 interface.
///
/// Provides methods for scanning devices, configuring logging,
/// and inhibiting device management.
#[proxy(
    interface = "org.freedesktop.ModemManager1",
    default_service = "org.freedesktop.ModemManager1",
    default_path = "/org/freedesktop/ModemManager1"
)]
pub trait MMManager {
    /// Re-scan for available modem devices.
    fn scan_devices(&self) -> zbus::Result<()>;

    /// Set the logging verbosity level.
    fn set_logging(&self, level: &str) -> zbus::Result<()>;

    /// Inhibit or release a modem device by its UID.
    fn inhibit_device(&self, uid: &str, inhibit: bool) -> zbus::Result<()>;

    /// ModemManager version string.
    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}
