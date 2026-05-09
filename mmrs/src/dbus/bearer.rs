//! ModemManager Bearer proxy.

use std::collections::HashMap;

use zbus::proxy;
use zvariant::OwnedValue;

/// Proxy for the ModemManager1 Bearer interface.
///
/// Represents a packet data connection. Provides methods to connect
/// and disconnect, plus properties for IP configuration and statistics.
#[proxy(
    interface = "org.freedesktop.ModemManager1.Bearer",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait MMBearer {
    /// Activate the bearer and bring up the data connection.
    fn connect(&self) -> zbus::Result<()>;

    /// Deactivate the bearer and tear down the data connection.
    fn disconnect(&self) -> zbus::Result<()>;

    /// Network interface name for this bearer (e.g., "wwan0").
    #[zbus(property)]
    fn interface(&self) -> zbus::Result<String>;

    /// Whether the bearer is currently connected.
    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;

    /// Whether the bearer connection is suspended.
    #[zbus(property)]
    fn suspended(&self) -> zbus::Result<bool>;

    /// IPv4 configuration dictionary.
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<HashMap<String, OwnedValue>>;

    /// IPv6 configuration dictionary.
    #[zbus(property)]
    fn ip6_config(&self) -> zbus::Result<HashMap<String, OwnedValue>>;

    /// Connection statistics (bytes tx/rx, duration, etc.).
    #[zbus(property)]
    fn stats(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
}
