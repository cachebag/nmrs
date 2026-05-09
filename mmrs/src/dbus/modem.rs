//! ModemManager Modem proxy.

use std::collections::HashMap;

use zbus::proxy;
use zvariant::OwnedObjectPath;

/// Proxy for the ModemManager1 Modem interface.
///
/// Provides methods for enabling/disabling the modem, managing bearers,
/// resetting, and querying modem properties like signal quality and state.
///
/// # Signals
///
/// The `StateChanged` signal is emitted whenever the modem state changes.
/// Use `receive_state_changed()` to get a stream of state change events:
///
/// ```ignore
/// let mut stream = modem_proxy.receive_modem_state_changed().await?;
/// while let Some(signal) = stream.next().await {
///     let args = signal.args()?;
///     println!("Old: {}, New: {}, Reason: {}", args.old, args.new, args.reason);
/// }
/// ```
#[proxy(
    interface = "org.freedesktop.ModemManager1.Modem",
    default_service = "org.freedesktop.ModemManager1"
)]
pub trait MMModem {
    /// Enable or disable the modem.
    fn enable(&self, enable: bool) -> zbus::Result<()>;

    /// List paths to all bearer objects owned by this modem.
    fn list_bearers(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    /// Create a new packet data bearer from the given properties.
    fn create_bearer(
        &self,
        properties: HashMap<&str, zvariant::Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    /// Delete an existing bearer.
    fn delete_bearer(&self, bearer: OwnedObjectPath) -> zbus::Result<()>;

    /// Reset the modem to its factory state.
    fn reset(&self) -> zbus::Result<()>;

    /// Set the power state of the modem.
    fn set_power_state(&self, state: u32) -> zbus::Result<()>;

    /// Send an AT command to the modem (direct AT channel access).
    fn command(&self, cmd: &str, timeout: u32) -> zbus::Result<String>;

    /// Path to the primary SIM object.
    #[zbus(property)]
    fn sim(&self) -> zbus::Result<OwnedObjectPath>;

    /// Current modem state (see `MMModemState` enum values).
    #[zbus(property)]
    fn state(&self) -> zbus::Result<i32>;

    /// Current power state of the modem.
    #[zbus(property)]
    fn power_state(&self) -> zbus::Result<u32>;

    /// Bitmask of current access technologies in use.
    #[zbus(property)]
    fn access_technologies(&self) -> zbus::Result<u32>;

    /// Signal quality (percentage, recently-updated flag).
    #[zbus(property)]
    fn signal_quality(&self) -> zbus::Result<(u32, bool)>;

    /// Modem manufacturer name.
    #[zbus(property)]
    fn manufacturer(&self) -> zbus::Result<String>;

    /// Modem model name.
    #[zbus(property)]
    fn model(&self) -> zbus::Result<String>;

    /// Equipment identifier (IMEI for GSM, ESN/MEID for CDMA).
    #[zbus(property)]
    fn equipment_identifier(&self) -> zbus::Result<String>;

    /// Maximum number of bearers this modem supports.
    #[zbus(property)]
    fn max_bearers(&self) -> zbus::Result<u32>;

    /// Maximum number of simultaneously active bearers.
    #[zbus(property)]
    fn max_active_bearers(&self) -> zbus::Result<u32>;

    /// Paths to all bearer objects owned by this modem.
    #[zbus(property)]
    fn bearers(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    /// Signal emitted when the modem state changes.
    ///
    /// Named `modem_state_changed` to avoid conflict with the `state`
    /// property's change stream. Use `receive_modem_state_changed()` to
    /// subscribe to this signal.
    ///
    /// Arguments:
    /// - `old`: The previous modem state
    /// - `new`: The new modem state
    /// - `reason`: The reason code for the state transition
    #[zbus(signal, name = "StateChanged")]
    fn modem_state_changed(&self, old: i32, new: i32, reason: u32);
}
