//! SIM-level public types.
//!
//! Mirrors the ModemManager `org.freedesktop.ModemManager1.Sim` interface.

use std::fmt;

/// Snapshot of a SIM slot on a managed modem.
///
/// Captures the most commonly used properties exposed by the
/// `org.freedesktop.ModemManager1.Sim` interface at a single point in time.
/// Construction is intentionally controlled — instances are produced by
/// the higher-level `mmrs` APIs and consumed by callers via field access.
///
/// # Example
///
/// ```rust
/// use mmrs::Sim;
///
/// fn describe(sim: &Sim) -> String {
///     format!("{} (active={}, iccid={})", sim.operator_name, sim.active, sim.iccid)
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sim {
    /// D-Bus object path of the SIM
    /// (e.g. `/org/freedesktop/ModemManager1/SIM/0`).
    pub path: String,
    /// Whether this SIM slot is currently active on the modem
    /// (`Active` property).
    pub active: bool,
    /// Integrated Circuit Card Identifier (`SimIdentifier` property).
    pub iccid: String,
    /// International Mobile Subscriber Identity (`Imsi` property).
    pub imsi: String,
    /// Operator name reported by the SIM (`OperatorName` property).
    ///
    /// May be empty when the SIM does not advertise a name.
    pub operator_name: String,
}

/// SIM lock state, mapping ModemManager's `MM_MODEM_LOCK_*` constants.
///
/// Reported via the `Modem.UnlockRequired` property; pass to
/// [`crate::ModemError::SimLocked`] when surfacing a lock to callers.
///
/// | Raw value | Constant                          | Variant         |
/// |-----------|-----------------------------------|-----------------|
/// | 0         | `MM_MODEM_LOCK_UNKNOWN`            | `Unknown`       |
/// | 1         | `MM_MODEM_LOCK_NONE`               | `None`          |
/// | 2         | `MM_MODEM_LOCK_SIM_PIN`            | `SimPin`        |
/// | 3         | `MM_MODEM_LOCK_SIM_PIN2`           | `SimPin2`       |
/// | 4         | `MM_MODEM_LOCK_SIM_PUK`            | `SimPuk`        |
/// | 5         | `MM_MODEM_LOCK_SIM_PUK2`           | `SimPuk2`       |
/// | 6         | `MM_MODEM_LOCK_PH_SP_PIN`          | `PhoneSpPin`    |
/// | 7         | `MM_MODEM_LOCK_PH_SP_PUK`          | `PhoneSpPuk`    |
/// | 8         | `MM_MODEM_LOCK_PH_NET_PIN`         | `PhoneNetPin`   |
/// | 9         | `MM_MODEM_LOCK_PH_NET_PUK`         | `PhoneNetPuk`   |
/// | 10        | `MM_MODEM_LOCK_PH_SIM_PIN`         | `PhoneSimPin`   |
/// | 11        | `MM_MODEM_LOCK_PH_CORP_PIN`        | `PhoneCorpPin`  |
/// | 12        | `MM_MODEM_LOCK_PH_CORP_PUK`        | `PhoneCorpPuk`  |
/// | 13        | `MM_MODEM_LOCK_PH_FSIM_PIN`        | `PhoneFsimPin`  |
/// | 14        | `MM_MODEM_LOCK_PH_FSIM_PUK`        | `PhoneFsimPuk`  |
/// | 15        | `MM_MODEM_LOCK_PH_NETSUB_PIN`      | `PhoneNetSubPin`|
/// | 16        | `MM_MODEM_LOCK_PH_NETSUB_PUK`      | `PhoneNetSubPuk`|
///
/// # Example
///
/// ```rust
/// use mmrs::SimLockState;
///
/// assert_eq!(SimLockState::from_raw(2), SimLockState::SimPin);
/// assert!(SimLockState::SimPin.requires_pin());
/// assert!(SimLockState::SimPuk.requires_puk());
/// assert!(!SimLockState::None.is_locked());
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimLockState {
    /// Lock state is not yet known.
    Unknown,
    /// No lock is active.
    None,
    /// SIM PIN required.
    SimPin,
    /// SIM PIN2 required.
    SimPin2,
    /// SIM PUK required (PIN retries exhausted).
    SimPuk,
    /// SIM PUK2 required.
    SimPuk2,
    /// Service-provider personalization PIN required.
    PhoneSpPin,
    /// Service-provider personalization PUK required.
    PhoneSpPuk,
    /// Network personalization PIN required.
    PhoneNetPin,
    /// Network personalization PUK required.
    PhoneNetPuk,
    /// Phone-to-SIM PIN required.
    PhoneSimPin,
    /// Corporate personalization PIN required.
    PhoneCorpPin,
    /// Corporate personalization PUK required.
    PhoneCorpPuk,
    /// Phone-to-very-first-SIM PIN required.
    PhoneFsimPin,
    /// Phone-to-very-first-SIM PUK required.
    PhoneFsimPuk,
    /// Network subset personalization PIN required.
    PhoneNetSubPin,
    /// Network subset personalization PUK required.
    PhoneNetSubPuk,
}

impl SimLockState {
    /// Decode the raw `u32` value from `Modem.UnlockRequired`.
    ///
    /// Unrecognised values map to [`SimLockState::Unknown`].
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::None,
            2 => Self::SimPin,
            3 => Self::SimPin2,
            4 => Self::SimPuk,
            5 => Self::SimPuk2,
            6 => Self::PhoneSpPin,
            7 => Self::PhoneSpPuk,
            8 => Self::PhoneNetPin,
            9 => Self::PhoneNetPuk,
            10 => Self::PhoneSimPin,
            11 => Self::PhoneCorpPin,
            12 => Self::PhoneCorpPuk,
            13 => Self::PhoneFsimPin,
            14 => Self::PhoneFsimPuk,
            15 => Self::PhoneNetSubPin,
            16 => Self::PhoneNetSubPuk,
            _ => Self::Unknown,
        }
    }

    /// Returns the raw `MM_MODEM_LOCK_*` constant.
    #[must_use]
    pub const fn as_raw(self) -> u32 {
        match self {
            Self::Unknown => 0,
            Self::None => 1,
            Self::SimPin => 2,
            Self::SimPin2 => 3,
            Self::SimPuk => 4,
            Self::SimPuk2 => 5,
            Self::PhoneSpPin => 6,
            Self::PhoneSpPuk => 7,
            Self::PhoneNetPin => 8,
            Self::PhoneNetPuk => 9,
            Self::PhoneSimPin => 10,
            Self::PhoneCorpPin => 11,
            Self::PhoneCorpPuk => 12,
            Self::PhoneFsimPin => 13,
            Self::PhoneFsimPuk => 14,
            Self::PhoneNetSubPin => 15,
            Self::PhoneNetSubPuk => 16,
        }
    }

    /// Returns `true` if the SIM is in any locked state (i.e. not
    /// [`None`](Self::None)).
    ///
    /// [`Unknown`](Self::Unknown) is treated as locked because the modem
    /// has not yet confirmed an unlocked state.
    #[must_use]
    pub const fn is_locked(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns `true` when a PIN code is required to unlock.
    #[must_use]
    pub const fn requires_pin(self) -> bool {
        matches!(
            self,
            Self::SimPin
                | Self::SimPin2
                | Self::PhoneSpPin
                | Self::PhoneNetPin
                | Self::PhoneSimPin
                | Self::PhoneCorpPin
                | Self::PhoneFsimPin
                | Self::PhoneNetSubPin
        )
    }

    /// Returns `true` when a PUK code is required (PIN retries exhausted).
    #[must_use]
    pub const fn requires_puk(self) -> bool {
        matches!(
            self,
            Self::SimPuk
                | Self::SimPuk2
                | Self::PhoneSpPuk
                | Self::PhoneNetPuk
                | Self::PhoneCorpPuk
                | Self::PhoneFsimPuk
                | Self::PhoneNetSubPuk
        )
    }
}

impl From<u32> for SimLockState {
    fn from(value: u32) -> Self {
        Self::from_raw(value)
    }
}

impl fmt::Display for SimLockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Unknown => "unknown",
            Self::None => "none",
            Self::SimPin => "sim-pin",
            Self::SimPin2 => "sim-pin2",
            Self::SimPuk => "sim-puk",
            Self::SimPuk2 => "sim-puk2",
            Self::PhoneSpPin => "ph-sp-pin",
            Self::PhoneSpPuk => "ph-sp-puk",
            Self::PhoneNetPin => "ph-net-pin",
            Self::PhoneNetPuk => "ph-net-puk",
            Self::PhoneSimPin => "ph-sim-pin",
            Self::PhoneCorpPin => "ph-corp-pin",
            Self::PhoneCorpPuk => "ph-corp-puk",
            Self::PhoneFsimPin => "ph-fsim-pin",
            Self::PhoneFsimPuk => "ph-fsim-puk",
            Self::PhoneNetSubPin => "ph-netsub-pin",
            Self::PhoneNetSubPuk => "ph-netsub-puk",
        };
        f.write_str(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_state_round_trip() {
        for raw in 0u32..=16 {
            let state = SimLockState::from_raw(raw);
            assert_eq!(state.as_raw(), raw, "round-trip broken for raw {raw}");
        }
    }

    #[test]
    fn lock_state_unknown_for_garbage_raw() {
        assert_eq!(SimLockState::from_raw(999), SimLockState::Unknown);
    }

    #[test]
    fn lock_state_predicates() {
        assert!(!SimLockState::None.is_locked());
        assert!(SimLockState::SimPin.is_locked());
        assert!(SimLockState::Unknown.is_locked());

        assert!(SimLockState::SimPin.requires_pin());
        assert!(SimLockState::PhoneSimPin.requires_pin());
        assert!(!SimLockState::SimPuk.requires_pin());

        assert!(SimLockState::SimPuk.requires_puk());
        assert!(SimLockState::PhoneCorpPuk.requires_puk());
        assert!(!SimLockState::SimPin.requires_puk());
    }

    #[test]
    fn lock_state_display() {
        assert_eq!(SimLockState::SimPin.to_string(), "sim-pin");
        assert_eq!(SimLockState::PhoneFsimPuk.to_string(), "ph-fsim-puk");
    }

    #[test]
    fn sim_struct_construction() {
        let sim = Sim {
            path: "/org/freedesktop/ModemManager1/SIM/0".into(),
            active: true,
            iccid: "89014103211118510720".into(),
            imsi: "310410000000000".into(),
            operator_name: "Test Carrier".into(),
        };
        assert!(sim.active);
        assert_eq!(sim.operator_name, "Test Carrier");
    }
}
