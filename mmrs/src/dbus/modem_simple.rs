//! ModemManager Modem.Simple proxy.

use std::collections::HashMap;

use zbus::proxy;
use zvariant::{OwnedObjectPath, OwnedValue};

/// Proxy for the ModemManager1 Modem.Simple interface.
///
/// Provides a simplified API for connecting, disconnecting,
/// and querying modem status in a single call.
#[proxy(
    interface = "org.freedesktop.ModemManager1.Modem.Simple",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait MMModemSimple {
    /// Simple connect with the given connection properties.
    ///
    /// Returns the path to the connected bearer.
    fn connect(
        &self,
        properties: HashMap<&str, zvariant::Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    /// Disconnect a specific bearer, or all if `"/"` is passed.
    fn disconnect(&self, bearer: OwnedObjectPath) -> zbus::Result<()>;

    /// Get the overall modem status as a property dictionary.
    fn get_status(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
}
