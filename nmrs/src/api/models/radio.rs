//! Radio and airplane-mode state types.
//!
//! NetworkManager tracks both a software-enabled flag (controlled via D-Bus)
//! and a hardware-enabled flag (reflecting the kernel rfkill state) for each
//! radio. [`RadioState`] captures both, and [`AirplaneModeState`] aggregates
//! Wi-Fi, WWAN, and Bluetooth into a single snapshot.

/// Software and hardware enabled state for a single radio.
///
/// `enabled` reflects the user-facing toggle (can be written via D-Bus).
/// `hardware_enabled` reflects the kernel rfkill state and cannot be changed
/// from userspace — if `false`, setting `enabled = true` is accepted by NM
/// but the radio remains off until hardware is unkilled.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RadioState {
    /// Software-enabled: can the user turn this radio on via NM?
    pub enabled: bool,
    /// Hardware-enabled: is rfkill allowing this radio?
    /// If `false`, `enabled = true` is a no-op until hardware is unkilled.
    pub hardware_enabled: bool,
}

impl RadioState {
    /// Creates a new `RadioState`.
    #[must_use]
    pub fn new(enabled: bool, hardware_enabled: bool) -> Self {
        Self {
            enabled,
            hardware_enabled,
        }
    }
}

/// Aggregated radio state for all radios that `nmrs` can control.
///
/// Returned by [`NetworkManager::airplane_mode_state`](crate::NetworkManager::airplane_mode_state).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct AirplaneModeState {
    /// Wi-Fi radio state.
    pub wifi: RadioState,
    /// WWAN (mobile broadband) radio state.
    pub wwan: RadioState,
    /// Bluetooth radio state (sourced from BlueZ + rfkill).
    pub bluetooth: RadioState,
}

impl AirplaneModeState {
    /// Creates a new `AirplaneModeState`.
    #[must_use]
    pub fn new(wifi: RadioState, wwan: RadioState, bluetooth: RadioState) -> Self {
        Self {
            wifi,
            wwan,
            bluetooth,
        }
    }

    /// Returns `true` if every radio `nmrs` can control is software-disabled.
    ///
    /// This is the "airplane mode is on" state — all radios off.
    #[must_use]
    pub fn is_airplane_mode(&self) -> bool {
        !self.wifi.enabled && !self.wwan.enabled && !self.bluetooth.enabled
    }

    /// Returns `true` if any radio has its hardware kill switch active.
    #[must_use]
    pub fn any_hardware_killed(&self) -> bool {
        !self.wifi.hardware_enabled
            || !self.wwan.hardware_enabled
            || !self.bluetooth.hardware_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_off_is_airplane_mode() {
        let state = AirplaneModeState::new(
            RadioState::new(false, true),
            RadioState::new(false, true),
            RadioState::new(false, true),
        );
        assert!(state.is_airplane_mode());
        assert!(!state.any_hardware_killed());
    }

    #[test]
    fn any_on_is_not_airplane_mode() {
        let state = AirplaneModeState::new(
            RadioState::new(true, true),
            RadioState::new(false, true),
            RadioState::new(false, true),
        );
        assert!(!state.is_airplane_mode());
    }

    #[test]
    fn hardware_killed_detected() {
        let state = AirplaneModeState::new(
            RadioState::new(true, true),
            RadioState::new(true, false),
            RadioState::new(true, true),
        );
        assert!(state.any_hardware_killed());
    }

    #[test]
    fn no_hardware_kill() {
        let state = AirplaneModeState::new(
            RadioState::new(false, true),
            RadioState::new(false, true),
            RadioState::new(false, true),
        );
        assert!(!state.any_hardware_killed());
    }

    #[test]
    fn radio_state_new() {
        let rs = RadioState::new(true, false);
        assert!(rs.enabled);
        assert!(!rs.hardware_enabled);
    }
}
