//! Modem-level public types.
//!
//! Mirrors the ModemManager `org.freedesktop.ModemManager1.Modem` interface:
//! [`ModemState`] decodes the `State` property, [`AccessTechnology`]
//! decodes the `AccessTechnologies` bitmask, and [`Modem`] is the
//! high-level snapshot returned by enumeration APIs.

use std::fmt;

/// Lifecycle state of a managed modem.
///
/// Maps from the `MM_MODEM_STATE_*` constants on the ModemManager
/// [`org.freedesktop.ModemManager1.Modem`] interface. Use
/// [`ModemState::from_raw`] (or the [`From`]/[`TryFrom`] impls) to convert
/// the raw `i32` returned over D-Bus.
///
/// | Raw value | Constant                          | Variant         |
/// |-----------|-----------------------------------|-----------------|
/// | -1        | `MM_MODEM_STATE_FAILED`           | `Failed`        |
/// | 0         | `MM_MODEM_STATE_UNKNOWN`          | `Unknown`       |
/// | 1         | `MM_MODEM_STATE_INITIALIZING`     | `Initializing`  |
/// | 2         | `MM_MODEM_STATE_LOCKED`           | `Locked`        |
/// | 3         | `MM_MODEM_STATE_DISABLED`         | `Disabled`      |
/// | 4         | `MM_MODEM_STATE_DISABLING`        | `Disabling`     |
/// | 5         | `MM_MODEM_STATE_ENABLING`         | `Enabling`      |
/// | 6         | `MM_MODEM_STATE_ENABLED`          | `Enabled`       |
/// | 7         | `MM_MODEM_STATE_SEARCHING`        | `Searching`     |
/// | 8         | `MM_MODEM_STATE_REGISTERED`       | `Registered`    |
/// | 9         | `MM_MODEM_STATE_DISCONNECTING`    | `Disconnecting` |
/// | 10        | `MM_MODEM_STATE_CONNECTING`       | `Connecting`    |
/// | 11        | `MM_MODEM_STATE_CONNECTED`        | `Connected`     |
///
/// # Example
///
/// ```rust
/// use mmrs::ModemState;
///
/// assert_eq!(ModemState::from_raw(8), ModemState::Registered);
/// assert!(ModemState::from_raw(11).is_connected());
/// assert!(ModemState::from_raw(7).is_searching());
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModemState {
    /// The modem is in a failed state and cannot be used.
    Failed,
    /// State is not yet known.
    Unknown,
    /// The modem is performing initialization checks.
    Initializing,
    /// The SIM is locked and requires unlocking.
    Locked,
    /// The modem is administratively disabled.
    Disabled,
    /// The modem is transitioning from enabled to disabled.
    Disabling,
    /// The modem is transitioning from disabled to enabled.
    Enabling,
    /// The modem is enabled but not yet registered.
    Enabled,
    /// The modem is actively searching for a network.
    Searching,
    /// The modem is registered with a network but not connected.
    Registered,
    /// A bearer disconnection is in progress.
    Disconnecting,
    /// A bearer connection is in progress.
    Connecting,
    /// A bearer is up and the modem is connected.
    Connected,
}

impl ModemState {
    /// Decode the raw `i32` value returned by ModemManager's `State` property.
    ///
    /// Unknown values map to [`ModemState::Unknown`] so the conversion is
    /// total.
    #[must_use]
    pub const fn from_raw(value: i32) -> Self {
        match value {
            -1 => Self::Failed,
            1 => Self::Initializing,
            2 => Self::Locked,
            3 => Self::Disabled,
            4 => Self::Disabling,
            5 => Self::Enabling,
            6 => Self::Enabled,
            7 => Self::Searching,
            8 => Self::Registered,
            9 => Self::Disconnecting,
            10 => Self::Connecting,
            11 => Self::Connected,
            _ => Self::Unknown,
        }
    }

    /// Returns the raw ModemManager constant for this state.
    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Failed => -1,
            Self::Unknown => 0,
            Self::Initializing => 1,
            Self::Locked => 2,
            Self::Disabled => 3,
            Self::Disabling => 4,
            Self::Enabling => 5,
            Self::Enabled => 6,
            Self::Searching => 7,
            Self::Registered => 8,
            Self::Disconnecting => 9,
            Self::Connecting => 10,
            Self::Connected => 11,
        }
    }

    /// Returns `true` when the modem has an active data bearer.
    #[must_use]
    pub const fn is_connected(self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Returns `true` when the modem is registered on a network.
    ///
    /// This includes the higher [`Connecting`](Self::Connecting) and
    /// [`Connected`](Self::Connected) states.
    #[must_use]
    pub const fn is_registered(self) -> bool {
        matches!(
            self,
            Self::Registered | Self::Disconnecting | Self::Connecting | Self::Connected
        )
    }

    /// Returns `true` while the modem is searching for a network.
    #[must_use]
    pub const fn is_searching(self) -> bool {
        matches!(self, Self::Searching)
    }

    /// Returns `true` for terminal failure states.
    #[must_use]
    pub const fn is_failed(self) -> bool {
        matches!(self, Self::Failed)
    }
}

impl From<i32> for ModemState {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl fmt::Display for ModemState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Failed => "failed",
            Self::Unknown => "unknown",
            Self::Initializing => "initializing",
            Self::Locked => "locked",
            Self::Disabled => "disabled",
            Self::Disabling => "disabling",
            Self::Enabling => "enabling",
            Self::Enabled => "enabled",
            Self::Searching => "searching",
            Self::Registered => "registered",
            Self::Disconnecting => "disconnecting",
            Self::Connecting => "connecting",
            Self::Connected => "connected",
        };
        f.write_str(s)
    }
}

// Raw bit positions from `MM_MODEM_ACCESS_TECHNOLOGY_*` in ModemManager's
// `mm-enums.h`. Kept as private constants so the public surface stays
// driven by named methods rather than magic numbers.
const AT_POTS: u32 = 1 << 0;
const AT_GSM: u32 = 1 << 1;
const AT_GSM_COMPACT: u32 = 1 << 2;
const AT_GPRS: u32 = 1 << 3;
const AT_EDGE: u32 = 1 << 4;
const AT_UMTS: u32 = 1 << 5;
const AT_HSDPA: u32 = 1 << 6;
const AT_HSUPA: u32 = 1 << 7;
const AT_HSPA: u32 = 1 << 8;
const AT_HSPA_PLUS: u32 = 1 << 9;
const AT_1XRTT: u32 = 1 << 10;
const AT_EVDO0: u32 = 1 << 11;
const AT_EVDOA: u32 = 1 << 12;
const AT_EVDOB: u32 = 1 << 13;
const AT_LTE: u32 = 1 << 14;
const AT_5GNR: u32 = 1 << 15;
const AT_LTE_CAT_M: u32 = 1 << 16;
const AT_LTE_NB_IOT: u32 = 1 << 17;

// Convenience masks
const AT_2G: u32 = AT_GSM | AT_GSM_COMPACT | AT_GPRS | AT_EDGE;
const AT_3G: u32 = AT_UMTS | AT_HSDPA | AT_HSUPA | AT_HSPA | AT_HSPA_PLUS;
const AT_4G: u32 = AT_LTE | AT_LTE_CAT_M | AT_LTE_NB_IOT;
const AT_5G: u32 = AT_5GNR;
const AT_CDMA: u32 = AT_1XRTT | AT_EVDO0 | AT_EVDOA | AT_EVDOB;
const AT_3GPP: u32 = AT_2G | AT_3G | AT_4G | AT_5G;

/// Bitmask of radio access technologies currently in use by a modem.
///
/// Constructed from the raw `u32` returned by the
/// `org.freedesktop.ModemManager1.Modem.AccessTechnologies` property
/// (which combines `MM_MODEM_ACCESS_TECHNOLOGY_*` bits).
///
/// The named bit constants are exposed as [`AccessTechnology::LTE`],
/// [`AccessTechnology::FIVE_G_NR`], etc., and predicate methods such as
/// [`AccessTechnology::has_lte`] / [`AccessTechnology::has_5g`] decode
/// common categories.
///
/// # Example
///
/// ```rust
/// use mmrs::AccessTechnology;
///
/// let tech = AccessTechnology::from(0x4000); // MM_MODEM_ACCESS_TECHNOLOGY_LTE
/// assert!(tech.has_lte());
/// assert!(tech.is_4g());
/// assert!(tech.is_3gpp());
/// assert!(!tech.has_5g());
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct AccessTechnology(u32);

impl AccessTechnology {
    /// Empty bitmask — the modem reports no known access technology.
    pub const NONE: Self = Self(0);

    /// Plain Old Telephone Service.
    pub const POTS: Self = Self(AT_POTS);
    /// GSM (2G).
    pub const GSM: Self = Self(AT_GSM);
    /// GSM Compact (2G).
    pub const GSM_COMPACT: Self = Self(AT_GSM_COMPACT);
    /// GPRS (2.5G).
    pub const GPRS: Self = Self(AT_GPRS);
    /// EDGE (2.75G).
    pub const EDGE: Self = Self(AT_EDGE);
    /// UMTS (3G).
    pub const UMTS: Self = Self(AT_UMTS);
    /// HSDPA (3.5G).
    pub const HSDPA: Self = Self(AT_HSDPA);
    /// HSUPA (3.5G).
    pub const HSUPA: Self = Self(AT_HSUPA);
    /// HSPA (3.5G).
    pub const HSPA: Self = Self(AT_HSPA);
    /// HSPA+ (3.75G).
    pub const HSPA_PLUS: Self = Self(AT_HSPA_PLUS);
    /// CDMA 1xRTT.
    pub const ONE_X_RTT: Self = Self(AT_1XRTT);
    /// CDMA EV-DO release 0.
    pub const EVDO0: Self = Self(AT_EVDO0);
    /// CDMA EV-DO revision A.
    pub const EVDOA: Self = Self(AT_EVDOA);
    /// CDMA EV-DO revision B.
    pub const EVDOB: Self = Self(AT_EVDOB);
    /// LTE (4G).
    pub const LTE: Self = Self(AT_LTE);
    /// 5G New Radio.
    pub const FIVE_G_NR: Self = Self(AT_5GNR);
    /// LTE Category M (LTE-M).
    pub const LTE_CAT_M: Self = Self(AT_LTE_CAT_M);
    /// LTE Narrowband IoT (NB-IoT).
    pub const LTE_NB_IOT: Self = Self(AT_LTE_NB_IOT);

    /// Construct from the raw `u32` bitmask reported by ModemManager.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns the raw bitmask, suitable for round-tripping over D-Bus.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns `true` when no bits are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if every bit in `other` is also set in `self`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns `true` if any bit in `other` overlaps `self`.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Returns the union of two access-technology masks.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns `true` if any 2G technology (GSM/GPRS/EDGE) is reported.
    #[must_use]
    pub const fn is_2g(self) -> bool {
        (self.0 & AT_2G) != 0
    }

    /// Returns `true` if any 3G technology (UMTS/HSPA family) is reported.
    #[must_use]
    pub const fn is_3g(self) -> bool {
        (self.0 & AT_3G) != 0
    }

    /// Returns `true` if any 4G LTE variant is reported.
    #[must_use]
    pub const fn is_4g(self) -> bool {
        (self.0 & AT_4G) != 0
    }

    /// Returns `true` if 5G NR is reported.
    #[must_use]
    pub const fn is_5g(self) -> bool {
        (self.0 & AT_5G) != 0
    }

    /// Returns `true` if the bitmask contains an LTE variant.
    ///
    /// Equivalent to [`is_4g`](Self::is_4g).
    #[must_use]
    pub const fn has_lte(self) -> bool {
        self.is_4g()
    }

    /// Returns `true` if the bitmask contains 5G NR.
    ///
    /// Equivalent to [`is_5g`](Self::is_5g).
    #[must_use]
    pub const fn has_5g(self) -> bool {
        self.is_5g()
    }

    /// Returns `true` if the mask reports any 3GPP technology
    /// (2G / 3G / 4G / 5G), as opposed to 3GPP2/CDMA.
    #[must_use]
    pub const fn is_3gpp(self) -> bool {
        (self.0 & AT_3GPP) != 0
    }

    /// Returns `true` if the mask reports a CDMA/EV-DO technology.
    #[must_use]
    pub const fn is_cdma(self) -> bool {
        (self.0 & AT_CDMA) != 0
    }
}

impl From<u32> for AccessTechnology {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<AccessTechnology> for u32 {
    fn from(value: AccessTechnology) -> Self {
        value.0
    }
}

impl std::ops::BitOr for AccessTechnology {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for AccessTechnology {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl fmt::Display for AccessTechnology {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("none");
        }

        let mut first = true;
        let mut write = |label: &str| -> fmt::Result {
            if !first {
                f.write_str("|")?;
            }
            first = false;
            f.write_str(label)
        };

        let pairs: &[(u32, &str)] = &[
            (AT_POTS, "POTS"),
            (AT_GSM, "GSM"),
            (AT_GSM_COMPACT, "GSM-Compact"),
            (AT_GPRS, "GPRS"),
            (AT_EDGE, "EDGE"),
            (AT_UMTS, "UMTS"),
            (AT_HSDPA, "HSDPA"),
            (AT_HSUPA, "HSUPA"),
            (AT_HSPA, "HSPA"),
            (AT_HSPA_PLUS, "HSPA+"),
            (AT_1XRTT, "1xRTT"),
            (AT_EVDO0, "EVDO0"),
            (AT_EVDOA, "EVDOA"),
            (AT_EVDOB, "EVDOB"),
            (AT_LTE, "LTE"),
            (AT_5GNR, "5G-NR"),
            (AT_LTE_CAT_M, "LTE-M"),
            (AT_LTE_NB_IOT, "NB-IoT"),
        ];

        for (bit, label) in pairs {
            if self.0 & bit != 0 {
                write(label)?;
            }
        }

        Ok(())
    }
}

/// High-level snapshot of a managed modem.
///
/// Mirrors the most commonly used properties on the
/// `org.freedesktop.ModemManager1.Modem` D-Bus interface. The values are
/// captured at a single point in time; use the live D-Bus proxy or the
/// monitoring API for change notifications. Construction is intentionally
/// controlled — instances are produced by the higher-level `mmrs` APIs
/// and consumed by callers via field access.
///
/// # Example
///
/// ```rust
/// use mmrs::Modem;
///
/// fn signal_bar(modem: &Modem) -> &'static str {
///     match modem.signal_quality {
///         0..=24 => "weak",
///         25..=49 => "ok",
///         50..=74 => "good",
///         _ => "strong",
///     }
/// }
///
/// fn describe(modem: &Modem) -> String {
///     format!(
///         "{} {} on {} ({}%, {})",
///         modem.manufacturer,
///         modem.model,
///         modem.access_technologies,
///         modem.signal_quality,
///         modem.state,
///     )
/// }
/// # let _ = describe;
/// # let _ = signal_bar;
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Modem {
    /// D-Bus object path of the modem
    /// (e.g. `/org/freedesktop/ModemManager1/Modem/0`).
    pub path: String,
    /// Current modem state.
    pub state: ModemState,
    /// Modem manufacturer (`Manufacturer` property).
    pub manufacturer: String,
    /// Modem model (`Model` property).
    pub model: String,
    /// Equipment identifier — IMEI for 3GPP modems,
    /// ESN/MEID for CDMA (`EquipmentIdentifier` property).
    pub equipment_identifier: String,
    /// Bitmask of access technologies currently in use
    /// (`AccessTechnologies` property).
    pub access_technologies: AccessTechnology,
    /// Signal quality as a percentage in `0..=100`
    /// (`SignalQuality` property, only the first tuple member).
    pub signal_quality: u32,
    /// D-Bus object path of the active SIM, if any
    /// (`Sim` property; `/` is reported as `None`).
    pub primary_sim_path: Option<String>,
    /// D-Bus object paths of bearers owned by this modem
    /// (`Bearers` property).
    pub bearer_paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modem_state_round_trips_through_raw() {
        for raw in [-1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] {
            let state = ModemState::from_raw(raw);
            assert_eq!(state.as_raw(), raw, "round-trip broken for raw {raw}");
        }
    }

    #[test]
    fn modem_state_unknown_for_garbage_raw() {
        assert_eq!(ModemState::from_raw(99), ModemState::Unknown);
        assert_eq!(ModemState::from_raw(-42), ModemState::Unknown);
    }

    #[test]
    fn modem_state_predicates() {
        assert!(ModemState::Connected.is_connected());
        assert!(!ModemState::Registered.is_connected());

        assert!(ModemState::Registered.is_registered());
        assert!(ModemState::Connected.is_registered());
        assert!(!ModemState::Searching.is_registered());

        assert!(ModemState::Searching.is_searching());
        assert!(!ModemState::Connected.is_searching());

        assert!(ModemState::Failed.is_failed());
        assert!(!ModemState::Unknown.is_failed());
    }

    #[test]
    fn modem_state_display_is_lowercase() {
        assert_eq!(ModemState::Connected.to_string(), "connected");
        assert_eq!(ModemState::Disabling.to_string(), "disabling");
    }

    #[test]
    fn access_technology_lte_and_5g() {
        let lte = AccessTechnology::LTE;
        assert!(lte.has_lte());
        assert!(lte.is_4g());
        assert!(lte.is_3gpp());
        assert!(!lte.has_5g());
        assert!(!lte.is_cdma());

        let nr = AccessTechnology::FIVE_G_NR;
        assert!(nr.has_5g());
        assert!(nr.is_5g());
        assert!(nr.is_3gpp());
        assert!(!nr.has_lte());
    }

    #[test]
    fn access_technology_cdma_is_not_3gpp() {
        let cdma = AccessTechnology::EVDOA;
        assert!(cdma.is_cdma());
        assert!(!cdma.is_3gpp());
    }

    #[test]
    fn access_technology_bit_operations() {
        let lte_plus_nr = AccessTechnology::LTE | AccessTechnology::FIVE_G_NR;
        assert!(lte_plus_nr.contains(AccessTechnology::LTE));
        assert!(lte_plus_nr.contains(AccessTechnology::FIVE_G_NR));
        assert!(lte_plus_nr.intersects(AccessTechnology::LTE));

        let just_lte = lte_plus_nr & AccessTechnology::LTE;
        assert_eq!(just_lte, AccessTechnology::LTE);
    }

    #[test]
    fn access_technology_bits_round_trip() {
        let raw = AccessTechnology::LTE.bits() | AccessTechnology::HSPA.bits();
        let tech = AccessTechnology::from(raw);
        assert!(tech.is_4g());
        assert!(tech.is_3g());
        assert_eq!(u32::from(tech), raw);
    }

    #[test]
    fn access_technology_default_and_empty() {
        let empty = AccessTechnology::default();
        assert!(empty.is_empty());
        assert_eq!(empty.to_string(), "none");
    }

    #[test]
    fn access_technology_display_joins_bits() {
        let mixed = AccessTechnology::LTE | AccessTechnology::HSPA;
        let rendered = mixed.to_string();
        assert!(rendered.contains("HSPA"));
        assert!(rendered.contains("LTE"));
        assert!(rendered.contains('|'));
    }
}
