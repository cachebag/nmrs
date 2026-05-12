//! Bearer-level public types.
//!
//! [`Bearer`] is the runtime snapshot of an active or pending data
//! connection, while [`BearerConfig`] is the input passed when asking the
//! modem to create or activate a bearer.

use std::fmt;
use std::net::Ipv4Addr;

/// IP family preference for a bearer connection.
///
/// Maps from `MM_BEARER_IP_FAMILY_*` bits used when creating a bearer.
///
/// | Raw value | Constant                          | Variant   |
/// |-----------|-----------------------------------|-----------|
/// | 0         | `MM_BEARER_IP_FAMILY_NONE`        | `None`    |
/// | 1         | `MM_BEARER_IP_FAMILY_IPV4`        | `Ipv4`    |
/// | 2         | `MM_BEARER_IP_FAMILY_IPV6`        | `Ipv6`    |
/// | 4         | `MM_BEARER_IP_FAMILY_IPV4V6`      | `Ipv4v6`  |
/// | `0xFFFF_FFFF` | `MM_BEARER_IP_FAMILY_ANY`     | `Any`     |
///
/// # Example
///
/// ```rust
/// use mmrs::IpType;
///
/// assert_eq!(IpType::from_raw(1), IpType::Ipv4);
/// assert!(IpType::Ipv4v6.is_dual_stack());
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum IpType {
    /// No IP family is requested.
    None,
    /// IPv4 only.
    #[default]
    Ipv4,
    /// IPv6 only.
    Ipv6,
    /// Dual-stack IPv4 + IPv6.
    Ipv4v6,
    /// Any family — let the network decide.
    Any,
}

impl IpType {
    /// Decode from the raw `MM_BEARER_IP_FAMILY_*` value.
    ///
    /// Returns [`IpType::None`] for `0` and unrecognised values.
    #[must_use]
    pub const fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Ipv4,
            2 => Self::Ipv6,
            4 => Self::Ipv4v6,
            u32::MAX => Self::Any,
            _ => Self::None,
        }
    }

    /// Returns the raw `MM_BEARER_IP_FAMILY_*` constant.
    #[must_use]
    pub const fn as_raw(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Ipv4 => 1,
            Self::Ipv6 => 2,
            Self::Ipv4v6 => 4,
            Self::Any => u32::MAX,
        }
    }

    /// Returns `true` if this family requests both IPv4 and IPv6.
    #[must_use]
    pub const fn is_dual_stack(self) -> bool {
        matches!(self, Self::Ipv4v6)
    }
}

impl From<u32> for IpType {
    fn from(value: u32) -> Self {
        Self::from_raw(value)
    }
}

impl fmt::Display for IpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::None => "none",
            Self::Ipv4 => "ipv4",
            Self::Ipv6 => "ipv6",
            Self::Ipv4v6 => "ipv4v6",
            Self::Any => "any",
        };
        f.write_str(s)
    }
}

/// IPv4 configuration reported on an active bearer.
///
/// Decoded from the `Bearer.Ip4Config` dictionary. All fields are
/// optional except the prefix, which defaults to `0` when not reported.
/// External callers receive `Ip4Config` from the higher-level API; the
/// example below shows how to inspect such an instance.
///
/// # Example
///
/// ```rust
/// use mmrs::Ip4Config;
///
/// fn print_address(cfg: &Ip4Config) {
///     if let Some(addr) = cfg.address {
///         println!("address: {}/{}", addr, cfg.prefix);
///     }
///     for dns in &cfg.dns {
///         println!("dns: {dns}");
///     }
/// }
/// # let _ = print_address;
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ip4Config {
    /// Configuration method reported by ModemManager
    /// (typically `"ppp"`, `"static"`, or `"dhcp"`).
    pub method: String,
    /// Bearer interface IPv4 address.
    pub address: Option<Ipv4Addr>,
    /// CIDR prefix length (e.g. `24` for `/24`).
    pub prefix: u32,
    /// Default gateway, if any.
    pub gateway: Option<Ipv4Addr>,
    /// DNS servers in priority order (typically up to three).
    pub dns: Vec<Ipv4Addr>,
    /// Path MTU when reported.
    pub mtu: Option<u32>,
}

/// Connection statistics for a bearer.
///
/// Decoded from the `Bearer.Stats` dictionary. Counters covering only the
/// current session live in the unqualified fields (`rx_bytes`,
/// `tx_bytes`, `duration_seconds`); cumulative counters across reconnects
/// (where the modem reports them) live in the `total_*` fields.
///
/// # Example
///
/// ```rust
/// use mmrs::BearerStats;
///
/// fn throughput_kib_per_s(stats: &BearerStats) -> f64 {
///     if stats.duration_seconds == 0 {
///         return 0.0;
///     }
///     let bytes = stats.rx_bytes + stats.tx_bytes;
///     (bytes as f64 / 1024.0) / stats.duration_seconds as f64
/// }
/// # let _ = throughput_kib_per_s;
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BearerStats {
    /// Bytes received during the current connection.
    pub rx_bytes: u64,
    /// Bytes sent during the current connection.
    pub tx_bytes: u64,
    /// Duration of the current connection, in seconds.
    pub duration_seconds: u32,
    /// Number of connection attempts (current session).
    pub attempts: u32,
    /// Number of failed connection attempts (current session).
    pub failed_attempts: u32,
    /// Cumulative connected duration across sessions, in seconds.
    pub total_duration_seconds: u32,
    /// Cumulative bytes received across sessions.
    pub total_rx_bytes: u64,
    /// Cumulative bytes sent across sessions.
    pub total_tx_bytes: u64,
}

/// Snapshot of a single packet-data bearer owned by a modem.
///
/// Mirrors the `org.freedesktop.ModemManager1.Bearer` D-Bus interface.
/// Instances are produced by the higher-level `mmrs` APIs; the example
/// below shows the inspection-only side of the type.
///
/// # Example
///
/// ```rust
/// use mmrs::Bearer;
///
/// fn summary(bearer: &Bearer) -> String {
///     let state = if bearer.connected { "up" } else { "down" };
///     format!("{} ({}): {}", bearer.interface, state, bearer.path)
/// }
/// # let _ = summary;
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bearer {
    /// D-Bus object path of the bearer
    /// (e.g. `/org/freedesktop/ModemManager1/Bearer/0`).
    pub path: String,
    /// Network interface name once the bearer is connected
    /// (`Interface` property, e.g. `"wwan0"`).
    pub interface: String,
    /// Whether the bearer is currently connected (`Connected` property).
    pub connected: bool,
    /// IPv4 configuration when the bearer is active and IPv4 is enabled.
    pub ip4_config: Option<Ip4Config>,
    /// Connection statistics reported by ModemManager.
    pub stats: BearerStats,
}

/// Configuration passed to `Modem.CreateBearer` (or `Simple.Connect`).
///
/// Use [`BearerConfig::new`] to start from a required APN and chain
/// [`with_user`](Self::with_user) / [`with_password`](Self::with_password)
/// / [`with_ip_type`](Self::with_ip_type) /
/// [`with_allow_roaming`](Self::with_allow_roaming) to fill optional fields.
///
/// # Example
///
/// ```rust
/// use mmrs::{BearerConfig, IpType};
///
/// let cfg = BearerConfig::new("internet")
///     .with_user("user")
///     .with_password("hunter2")
///     .with_ip_type(IpType::Ipv4v6)
///     .with_allow_roaming(true);
///
/// assert_eq!(cfg.apn, "internet");
/// assert_eq!(cfg.ip_type, IpType::Ipv4v6);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BearerConfig {
    /// Access Point Name (`apn` key) — required by virtually every carrier.
    pub apn: String,
    /// Requested IP family (`ip-type` key).
    pub ip_type: IpType,
    /// Optional username for PAP/CHAP auth (`user` key).
    pub user: Option<String>,
    /// Optional password for PAP/CHAP auth (`password` key).
    pub password: Option<String>,
    /// Allow data while roaming (`allow-roaming` key). Defaults to `false`.
    pub allow_roaming: bool,
}

impl BearerConfig {
    /// Creates a new [`BearerConfig`] with the given APN and sensible defaults
    /// (`Ipv4`, no credentials, roaming disabled).
    ///
    /// # Example
    ///
    /// ```rust
    /// use mmrs::BearerConfig;
    ///
    /// let cfg = BearerConfig::new("hologram");
    /// assert_eq!(cfg.apn, "hologram");
    /// assert!(!cfg.allow_roaming);
    /// ```
    pub fn new(apn: impl Into<String>) -> Self {
        Self {
            apn: apn.into(),
            ip_type: IpType::Ipv4,
            user: None,
            password: None,
            allow_roaming: false,
        }
    }

    /// Sets the requested IP family.
    #[must_use]
    pub fn with_ip_type(mut self, ip_type: IpType) -> Self {
        self.ip_type = ip_type;
        self
    }

    /// Sets the username for PAP/CHAP authentication.
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets the password for PAP/CHAP authentication.
    #[must_use]
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Allows or disallows data while roaming.
    #[must_use]
    pub fn with_allow_roaming(mut self, allow: bool) -> Self {
        self.allow_roaming = allow;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ip_type_round_trip() {
        for variant in [
            IpType::None,
            IpType::Ipv4,
            IpType::Ipv6,
            IpType::Ipv4v6,
            IpType::Any,
        ] {
            assert_eq!(IpType::from_raw(variant.as_raw()), variant);
        }
    }

    #[test]
    fn ip_type_default_is_ipv4() {
        assert_eq!(IpType::default(), IpType::Ipv4);
    }

    #[test]
    fn ip_type_unknown_maps_to_none() {
        assert_eq!(IpType::from_raw(99), IpType::None);
    }

    #[test]
    fn ip_type_dual_stack_predicate() {
        assert!(IpType::Ipv4v6.is_dual_stack());
        assert!(!IpType::Ipv4.is_dual_stack());
        assert!(!IpType::Any.is_dual_stack());
    }

    #[test]
    fn bearer_config_new_has_defaults() {
        let cfg = BearerConfig::new("hologram");
        assert_eq!(cfg.apn, "hologram");
        assert_eq!(cfg.ip_type, IpType::Ipv4);
        assert!(cfg.user.is_none());
        assert!(cfg.password.is_none());
        assert!(!cfg.allow_roaming);
    }

    #[test]
    fn bearer_config_builders_set_fields() {
        let cfg = BearerConfig::new("internet")
            .with_user("u")
            .with_password("p")
            .with_ip_type(IpType::Ipv4v6)
            .with_allow_roaming(true);

        assert_eq!(cfg.user.as_deref(), Some("u"));
        assert_eq!(cfg.password.as_deref(), Some("p"));
        assert_eq!(cfg.ip_type, IpType::Ipv4v6);
        assert!(cfg.allow_roaming);
    }

    #[test]
    fn bearer_stats_default_is_zeroed() {
        let stats = BearerStats::default();
        assert_eq!(stats.rx_bytes, 0);
        assert_eq!(stats.tx_bytes, 0);
        assert_eq!(stats.duration_seconds, 0);
        assert_eq!(stats.total_rx_bytes, 0);
    }

    #[test]
    fn ip4_config_default_is_empty() {
        let cfg = Ip4Config::default();
        assert!(cfg.method.is_empty());
        assert!(cfg.address.is_none());
        assert_eq!(cfg.prefix, 0);
        assert!(cfg.dns.is_empty());
    }
}
