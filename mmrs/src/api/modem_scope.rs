//! Per-modem scoped high-level API.

use crate::api::models::{
    AccessTechnology, Bearer, BearerConfig, ConnectionStatus, Modem, Result, Sim,
};
use crate::api::modem_manager::ModemManager;

/// Operations scoped to a single ModemManager modem object path.
///
/// Create this with [`ModemManager::modem`] when a system has multiple modems
/// and the default primary-modem behavior is not specific enough.
#[derive(Debug)]
pub struct ModemScope<'a> {
    pub(crate) mm: &'a ModemManager,
    pub(crate) path: String,
}

impl<'a> ModemScope<'a> {
    pub(crate) fn new(mm: &'a ModemManager, path: &str) -> Self {
        Self {
            mm,
            path: path.to_string(),
        }
    }

    /// Returns the scoped modem object path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns a snapshot of this modem.
    pub async fn info(&self) -> Result<Modem> {
        self.mm.modem_info_for_path(&self.path).await
    }

    /// Enables this modem.
    pub async fn enable(&self) -> Result<()> {
        self.mm.enable_for_path(&self.path).await
    }

    /// Disables this modem.
    pub async fn disable(&self) -> Result<()> {
        self.mm.disable_for_path(&self.path).await
    }

    /// Connects this modem using only an APN.
    pub async fn connect_simple(&self, apn: &str) -> Result<Bearer> {
        self.connect(&BearerConfig::new(apn)).await
    }

    /// Connects this modem using a full bearer configuration.
    pub async fn connect(&self, config: &BearerConfig) -> Result<Bearer> {
        self.mm.connect_for_path(&self.path, config).await
    }

    /// Disconnects all bearers on this modem.
    pub async fn disconnect(&self) -> Result<()> {
        self.mm.disconnect_for_path(&self.path).await
    }

    /// Returns this modem's current connection status.
    pub async fn status(&self) -> Result<ConnectionStatus> {
        self.mm.status_for_path(&self.path).await
    }

    /// Returns this modem's active SIM, if one is reported.
    pub async fn sim(&self) -> Result<Option<Sim>> {
        self.mm.sim_for_path(&self.path).await
    }

    /// Sends a PIN to unlock this modem's SIM.
    pub async fn unlock_pin(&self, pin: &str) -> Result<()> {
        self.mm.unlock_pin_for_path(&self.path, pin).await
    }

    /// Sends a PUK and new PIN to unlock this modem's SIM.
    pub async fn unlock_puk(&self, puk: &str, new_pin: &str) -> Result<()> {
        self.mm.unlock_puk_for_path(&self.path, puk, new_pin).await
    }

    /// Enables or disables SIM PIN checking on this modem.
    pub async fn set_pin_enabled(&self, pin: &str, enabled: bool) -> Result<()> {
        self.mm
            .set_pin_enabled_for_path(&self.path, pin, enabled)
            .await
    }

    /// Changes this modem SIM's PIN.
    pub async fn change_pin(&self, old: &str, new: &str) -> Result<()> {
        self.mm.change_pin_for_path(&self.path, old, new).await
    }

    /// Returns this modem's current signal quality percentage.
    pub async fn signal_quality(&self) -> Result<u32> {
        self.mm.signal_quality_for_path(&self.path).await
    }

    /// Returns this modem's current access technology bitmask.
    pub async fn access_technology(&self) -> Result<AccessTechnology> {
        self.mm.access_technology_for_path(&self.path).await
    }
}
