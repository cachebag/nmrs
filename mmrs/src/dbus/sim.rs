//! ModemManager SIM proxy.

use zbus::proxy;

/// Proxy for the ModemManager1 SIM interface.
///
/// Provides methods for PIN/PUK management and access
/// to SIM identification properties.
#[proxy(
    interface = "org.freedesktop.ModemManager1.Sim",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait MMSim {
    /// Send the SIM PIN to unlock the modem.
    fn send_pin(&self, pin: &str) -> zbus::Result<()>;

    /// Send the PUK and set a new PIN.
    fn send_puk(&self, puk: &str, pin: &str) -> zbus::Result<()>;

    /// Enable or disable PIN checking on the SIM.
    fn enable_pin(&self, pin: &str, enabled: bool) -> zbus::Result<()>;

    /// Change the SIM PIN.
    fn change_pin(&self, old_pin: &str, new_pin: &str) -> zbus::Result<()>;

    /// Whether this SIM slot is currently active.
    #[zbus(property)]
    fn active(&self) -> zbus::Result<bool>;

    /// SIM identifier (ICCID).
    #[zbus(property)]
    fn sim_identifier(&self) -> zbus::Result<String>;

    /// International Mobile Subscriber Identity.
    #[zbus(property)]
    fn imsi(&self) -> zbus::Result<String>;

    /// Name of the operator this SIM is registered with.
    #[zbus(property)]
    fn operator_name(&self) -> zbus::Result<String>;
}
