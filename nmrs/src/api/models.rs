use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use crate::util::validation::validate_bluetooth_address;

/// NetworkManager active connection state.
///
/// These values represent the lifecycle states of an active connection
/// as reported by the NM D-Bus API.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveConnectionState {
    /// Connection state is unknown.
    Unknown,
    /// Connection is activating (connecting).
    Activating,
    /// Connection is fully activated (connected).
    Activated,
    /// Connection is deactivating (disconnecting).
    Deactivating,
    /// Connection is fully deactivated (disconnected).
    Deactivated,
    /// Unknown state code not mapped to a specific variant.
    Other(u32),
}

impl From<u32> for ActiveConnectionState {
    fn from(code: u32) -> Self {
        match code {
            0 => Self::Unknown,
            1 => Self::Activating,
            2 => Self::Activated,
            3 => Self::Deactivating,
            4 => Self::Deactivated,
            v => Self::Other(v),
        }
    }
}

impl Display for ActiveConnectionState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Activating => write!(f, "activating"),
            Self::Activated => write!(f, "activated"),
            Self::Deactivating => write!(f, "deactivating"),
            Self::Deactivated => write!(f, "deactivated"),
            Self::Other(v) => write!(f, "unknown state ({v})"),
        }
    }
}

/// NetworkManager active connection state reason codes.
///
/// These values indicate why an active connection transitioned to its
/// current state. Use `ConnectionStateReason::from(code)` to convert
/// from the raw u32 values returned by NetworkManager signals.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStateReason {
    /// The reason is unknown.
    Unknown,
    /// No specific reason.
    None,
    /// User disconnected.
    UserDisconnected,
    /// Device disconnected.
    DeviceDisconnected,
    /// The NetworkManager service stopped.
    ServiceStopped,
    /// IP configuration was invalid.
    IpConfigInvalid,
    /// Connection timed out while activating.
    ConnectTimeout,
    /// Service start timed out.
    ServiceStartTimeout,
    /// Service failed to start.
    ServiceStartFailed,
    /// No secrets (password) were provided.
    NoSecrets,
    /// Login/authentication failed.
    LoginFailed,
    /// The connection was removed.
    ConnectionRemoved,
    /// A dependency failed.
    DependencyFailed,
    /// Device realization failed.
    DeviceRealizeFailed,
    /// Device was removed.
    DeviceRemoved,
    /// Unknown reason code not mapped to a specific variant.
    Other(u32),
}

impl From<u32> for ConnectionStateReason {
    fn from(code: u32) -> Self {
        match code {
            0 => Self::Unknown,
            1 => Self::None,
            2 => Self::UserDisconnected,
            3 => Self::DeviceDisconnected,
            4 => Self::ServiceStopped,
            5 => Self::IpConfigInvalid,
            6 => Self::ConnectTimeout,
            7 => Self::ServiceStartTimeout,
            8 => Self::ServiceStartFailed,
            9 => Self::NoSecrets,
            10 => Self::LoginFailed,
            11 => Self::ConnectionRemoved,
            12 => Self::DependencyFailed,
            13 => Self::DeviceRealizeFailed,
            14 => Self::DeviceRemoved,
            v => Self::Other(v),
        }
    }
}

impl Display for ConnectionStateReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::None => write!(f, "none"),
            Self::UserDisconnected => write!(f, "user disconnected"),
            Self::DeviceDisconnected => write!(f, "device disconnected"),
            Self::ServiceStopped => write!(f, "service stopped"),
            Self::IpConfigInvalid => write!(f, "IP configuration invalid"),
            Self::ConnectTimeout => write!(f, "connection timed out"),
            Self::ServiceStartTimeout => write!(f, "service start timed out"),
            Self::ServiceStartFailed => write!(f, "service start failed"),
            Self::NoSecrets => write!(f, "no secrets (password) provided"),
            Self::LoginFailed => write!(f, "login/authentication failed"),
            Self::ConnectionRemoved => write!(f, "connection was removed"),
            Self::DependencyFailed => write!(f, "dependency failed"),
            Self::DeviceRealizeFailed => write!(f, "device realization failed"),
            Self::DeviceRemoved => write!(f, "device was removed"),
            Self::Other(v) => write!(f, "unknown reason ({v})"),
        }
    }
}

/// Converts a connection state reason code to a specific `ConnectionError`.
///
/// Maps authentication-related failures to `AuthFailed`, timeout issues to `Timeout`,
/// and other failures to the appropriate variant.
pub fn connection_state_reason_to_error(code: u32) -> ConnectionError {
    let reason = ConnectionStateReason::from(code);
    match reason {
        // Authentication failures
        ConnectionStateReason::NoSecrets | ConnectionStateReason::LoginFailed => {
            ConnectionError::AuthFailed
        }

        // Timeout failures
        ConnectionStateReason::ConnectTimeout | ConnectionStateReason::ServiceStartTimeout => {
            ConnectionError::Timeout
        }

        // IP configuration failures (often DHCP)
        ConnectionStateReason::IpConfigInvalid => ConnectionError::DhcpFailed,

        // All other failures
        _ => ConnectionError::ActivationFailed(reason),
    }
}

/// NetworkManager device state reason codes.
///
/// These values come from the NM D-Bus API and indicate why a device
/// transitioned to its current state. Use `StateReason::from(code)` to
/// convert from the raw u32 values returned by NetworkManager.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateReason {
    /// The reason is unknown.
    Unknown,
    /// No specific reason given.
    None,
    /// The user disconnected the device.
    UserDisconnected,
    /// The device was disconnected by the system.
    DeviceDisconnected,
    /// The carrier/link status changed (e.g., cable unplugged).
    CarrierChanged,
    /// The Wi-Fi supplicant disconnected unexpectedly.
    SupplicantDisconnected,
    /// The Wi-Fi supplicant's configuration failed.
    SupplicantConfigFailed,
    /// The Wi-Fi supplicant failed (authentication issue).
    SupplicantFailed,
    /// The Wi-Fi supplicant timed out during authentication.
    SupplicantTimeout,
    /// PPP connection start failed.
    PppStartFailed,
    /// DHCP client failed to start.
    DhcpStartFailed,
    /// DHCP client encountered an error.
    DhcpError,
    /// DHCP client failed to obtain an IP address.
    DhcpFailed,
    /// Modem connection failed.
    ModemConnectionFailed,
    /// Modem initialization failed.
    ModemInitFailed,
    /// InfiniBand device mode mismatch.
    InfinibandMode,
    /// A dependency connection failed.
    DependencyFailed,
    /// BR2684 bridge setup failed.
    Br2684Failed,
    /// Failed to set the device mode (e.g., AP mode).
    ModeSetFailed,
    /// GSM modem APN selection failed.
    GsmApnSelectFailed,
    /// GSM modem is not searching for networks.
    GsmNotSearching,
    /// GSM network registration was denied.
    GsmRegistrationDenied,
    /// GSM network registration timed out.
    GsmRegistrationTimeout,
    /// GSM network registration failed.
    GsmRegistrationFailed,
    /// GSM SIM PIN check failed.
    GsmPinCheckFailed,
    /// Required firmware is missing for the device.
    FirmwareMissing,
    /// The device was removed from the system.
    DeviceRemoved,
    /// The system is entering sleep mode.
    Sleeping,
    /// The connection profile was removed.
    ConnectionRemoved,
    /// The user requested the operation.
    UserRequested,
    /// Carrier status changed.
    Carrier,
    /// NetworkManager assumed an existing connection.
    ConnectionAssumed,
    /// The Wi-Fi supplicant became available.
    SupplicantAvailable,
    /// The modem device was not found.
    ModemNotFound,
    /// Bluetooth connection failed.
    BluetoothFailed,
    /// GSM SIM card is not inserted.
    GsmSimNotInserted,
    /// GSM SIM PIN is required.
    GsmSimPinRequired,
    /// GSM SIM PUK is required.
    GsmSimPukRequired,
    /// Wrong GSM SIM card inserted.
    GsmSimWrong,
    /// The requested SSID was not found.
    SsidNotFound,
    /// A secondary connection failed.
    SecondaryConnectionFailed,
    /// DCB/FCoE setup failed.
    DcbFcoeFailed,
    /// teamd control interface failed.
    TeamdControlFailed,
    /// Modem operation failed.
    ModemFailed,
    /// Modem became available.
    ModemAvailable,
    /// SIM PIN was incorrect.
    SimPinIncorrect,
    /// A new connection activation was queued.
    NewActivationEnqueued,
    /// Parent device became unreachable.
    ParentUnreachable,
    /// Parent device changed.
    ParentChanged,
    /// Unknown reason code not mapped to a specific variant.
    Other(u32),
}

/// Represents a Wi-Fi network discovered during a scan.
///
/// This struct contains information about a WiFi network that was discovered
/// by NetworkManager during a scan operation.
///
/// # Examples
///
/// ```no_run
/// use nmrs::NetworkManager;
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
///
/// // Scan for networks
/// nm.scan_networks().await?;
/// let networks = nm.list_networks().await?;
///
/// for net in networks {
///     println!("SSID: {}", net.ssid);
///     println!("  Signal: {}%", net.strength.unwrap_or(0));
///     println!("  Secured: {}", net.secured);
///     
///     if let Some(freq) = net.frequency {
///         let band = if freq > 5000 { "5GHz" } else { "2.4GHz" };
///         println!("  Band: {}", band);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    /// Device interface name (e.g., "wlan0")
    pub device: String,
    /// Network SSID (name)
    pub ssid: String,
    /// Access point MAC address (BSSID)
    pub bssid: Option<String>,
    /// Signal strength (0-100)
    pub strength: Option<u8>,
    /// Frequency in MHz (e.g., 2437 for channel 6)
    pub frequency: Option<u32>,
    /// Whether the network requires authentication
    pub secured: bool,
    /// Whether the network uses WPA-PSK authentication
    pub is_psk: bool,
    /// Whether the network uses WPA-EAP (Enterprise) authentication
    pub is_eap: bool,
    /// Assigned IPv4 address with CIDR notation (only present when connected)
    pub ip4_address: Option<String>,
    /// Assigned IPv6 address with CIDR notation (only present when connected)
    pub ip6_address: Option<String>,
}

/// Detailed information about a Wi-Fi network.
///
/// Contains comprehensive information about a WiFi network, including
/// connection status, signal quality, and technical details.
///
/// # Examples
///
/// ```no_run
/// use nmrs::NetworkManager;
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
/// let networks = nm.list_networks().await?;
///
/// if let Some(network) = networks.first() {
///     let info = nm.show_details(network).await?;
///     
///     println!("Network: {}", info.ssid);
///     println!("Signal: {} {}", info.strength, info.bars);
///     println!("Security: {}", info.security);
///     println!("Status: {}", info.status);
///     
///     if let Some(rate) = info.rate_mbps {
///         println!("Speed: {} Mbps", rate);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Network SSID (name)
    pub ssid: String,
    /// Access point MAC address (BSSID)
    pub bssid: String,
    /// Signal strength (0-100)
    pub strength: u8,
    /// Frequency in MHz
    pub freq: Option<u32>,
    /// WiFi channel number
    pub channel: Option<u16>,
    /// Operating mode (e.g., "infrastructure")
    pub mode: String,
    /// Connection speed in Mbps
    pub rate_mbps: Option<u32>,
    /// Visual signal strength representation (e.g., "▂▄▆█")
    pub bars: String,
    /// Security type description
    pub security: String,
    /// Connection status
    pub status: String,
    /// Assigned IPv4 address with CIDR notation (only present when connected)
    pub ip4_address: Option<String>,
    /// Assigned IPv6 address with CIDR notation (only present when connected)
    pub ip6_address: Option<String>,
}

/// Represents a network device managed by NetworkManager.
///
/// A device can be a WiFi adapter, Ethernet interface, or other network hardware.
///
/// # Examples
///
/// ```no_run
/// use nmrs::NetworkManager;
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
/// let devices = nm.list_devices().await?;
///
/// for device in devices {
///     println!("Interface: {}", device.interface);
///     println!("  Type: {}", device.device_type);
///     println!("  State: {}", device.state);
///     println!("  MAC: {}", device.identity.current_mac);
///     
///     if device.is_wireless() {
///         println!("  This is a WiFi device");
///     } else if device.is_wired() {
///         println!("  This is an Ethernet device");
///     } else if device.is_bluetooth() {
///         println!("  This is a Bluetooth device");
///     }
///     
///     if let Some(driver) = &device.driver {
///         println!("  Driver: {}", driver);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Device {
    /// D-Bus object path
    pub path: String,
    /// Interface name (e.g., "wlan0", "eth0")
    pub interface: String,
    /// Device hardware identity (MAC addresses)
    pub identity: DeviceIdentity,
    /// Type of device (WiFi, Ethernet, etc.)
    pub device_type: DeviceType,
    /// Current device state
    pub state: DeviceState,
    /// Whether NetworkManager manages this device
    pub managed: Option<bool>,
    /// Kernel driver name
    pub driver: Option<String>,
    /// Assigned IPv4 address with CIDR notation (only present when connected)
    pub ip4_address: Option<String>,
    /// Assigned IPv6 address with CIDR notation (only present when connected)
    pub ip6_address: Option<String>,
    // Link speed in Mb/s (wired devices)
    // pub speed: Option<u32>,
}

/// Represents the hardware identity of a network device.
///
/// Contains MAC addresses that uniquely identify the device. The permanent
/// MAC is burned into the hardware, while the current MAC may be different
/// if MAC address randomization or spoofing is enabled.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceIdentity {
    /// The permanent (factory-assigned) MAC address.
    pub permanent_mac: String,
    /// The current MAC address in use (may differ if randomized/spoofed).
    pub current_mac: String,
}

impl DeviceIdentity {
    /// Creates a new `DeviceIdentity`.
    ///
    /// # Arguments
    ///
    /// * `permanent_mac` - The permanent (factory-assigned) MAC address
    /// * `current_mac` - The current MAC address in use
    pub fn new(permanent_mac: String, current_mac: String) -> Self {
        Self {
            permanent_mac,
            current_mac,
        }
    }
}

/// EAP (Extensible Authentication Protocol) method for WPA-Enterprise Wi-Fi.
///
/// These are the outer authentication methods used in 802.1X authentication.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EapMethod {
    /// Protected EAP (PEAPv0) - tunnels inner authentication in TLS.
    /// Most commonly used with MSCHAPv2 inner authentication.
    Peap,
    /// Tunneled TLS (EAP-TTLS) - similar to PEAP but more flexible.
    /// Can use various inner authentication methods like PAP or MSCHAPv2.
    Ttls,
}

/// Phase 2 (inner) authentication methods for EAP connections.
///
/// These methods run inside the TLS tunnel established by the outer
/// EAP method (PEAP or TTLS).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase2 {
    /// Microsoft Challenge Handshake Authentication Protocol v2.
    /// More secure than PAP, commonly used with PEAP.
    Mschapv2,
    /// Password Authentication Protocol.
    /// Simple plaintext password (protected by TLS tunnel).
    /// Often used with TTLS.
    Pap,
}

/// EAP options for WPA-EAP (Enterprise) Wi-Fi connections.
///
/// Configuration for 802.1X authentication, commonly used in corporate
/// and educational networks.
///
/// # Examples
///
/// ## PEAP with MSCHAPv2 (Common Corporate Setup)
///
/// ```rust
/// use nmrs::{EapOptions, EapMethod, Phase2};
///
/// let opts = EapOptions::new("employee@company.com", "my_password")
///     .with_anonymous_identity("anonymous@company.com")
///     .with_domain_suffix_match("company.com")
///     .with_system_ca_certs(true)  // Use system certificate store
///     .with_method(EapMethod::Peap)
///     .with_phase2(Phase2::Mschapv2);
/// ```
///
/// ## TTLS with PAP (Alternative Setup)
///
/// ```rust
/// use nmrs::{EapOptions, EapMethod, Phase2};
///
/// let opts = EapOptions::new("student@university.edu", "password")
///     .with_ca_cert_path("file:///etc/ssl/certs/university-ca.pem")
///     .with_method(EapMethod::Ttls)
///     .with_phase2(Phase2::Pap);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EapOptions {
    /// User identity (usually email or username)
    pub identity: String,
    /// Password for authentication
    pub password: String,
    /// Anonymous outer identity (for privacy)
    pub anonymous_identity: Option<String>,
    /// Domain to match against server certificate
    pub domain_suffix_match: Option<String>,
    /// Path to CA certificate file (file:// URL)
    pub ca_cert_path: Option<String>,
    /// Use system CA certificate store
    pub system_ca_certs: bool,
    /// EAP method (PEAP or TTLS)
    pub method: EapMethod,
    /// Phase 2 inner authentication method
    pub phase2: Phase2,
}

impl Default for EapOptions {
    fn default() -> Self {
        Self {
            identity: String::new(),
            password: String::new(),
            anonymous_identity: None,
            domain_suffix_match: None,
            ca_cert_path: None,
            system_ca_certs: false,
            method: EapMethod::Peap,
            phase2: Phase2::Mschapv2,
        }
    }
}

impl EapOptions {
    /// Creates a new `EapOptions` with the minimum required fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{EapOptions, EapMethod, Phase2};
    ///
    /// let opts = EapOptions::new("user@example.com", "password")
    ///     .with_method(EapMethod::Peap)
    ///     .with_phase2(Phase2::Mschapv2);
    /// ```
    pub fn new(identity: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            password: password.into(),
            ..Default::default()
        }
    }

    /// Creates a new `EapOptions` builder.
    ///
    /// This provides an alternative way to construct EAP options with a fluent API,
    /// making it clearer what each configuration option does.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{EapOptions, EapMethod, Phase2};
    ///
    /// let opts = EapOptions::builder()
    ///     .identity("user@company.com")
    ///     .password("my_password")
    ///     .method(EapMethod::Peap)
    ///     .phase2(Phase2::Mschapv2)
    ///     .domain_suffix_match("company.com")
    ///     .system_ca_certs(true)
    ///     .build();
    /// ```
    pub fn builder() -> EapOptionsBuilder {
        EapOptionsBuilder::default()
    }

    /// Sets the anonymous identity for privacy.
    pub fn with_anonymous_identity(mut self, anonymous_identity: impl Into<String>) -> Self {
        self.anonymous_identity = Some(anonymous_identity.into());
        self
    }

    /// Sets the domain suffix to match against the server certificate.
    pub fn with_domain_suffix_match(mut self, domain: impl Into<String>) -> Self {
        self.domain_suffix_match = Some(domain.into());
        self
    }

    /// Sets the path to the CA certificate file (must start with `file://`).
    pub fn with_ca_cert_path(mut self, path: impl Into<String>) -> Self {
        self.ca_cert_path = Some(path.into());
        self
    }

    /// Sets whether to use the system CA certificate store.
    pub fn with_system_ca_certs(mut self, use_system: bool) -> Self {
        self.system_ca_certs = use_system;
        self
    }

    /// Sets the EAP method (PEAP or TTLS).
    pub fn with_method(mut self, method: EapMethod) -> Self {
        self.method = method;
        self
    }

    /// Sets the Phase 2 authentication method.
    pub fn with_phase2(mut self, phase2: Phase2) -> Self {
        self.phase2 = phase2;
        self
    }
}

/// Builder for constructing `EapOptions` with a fluent API.
///
/// This builder provides an ergonomic way to create EAP (Enterprise WiFi)
/// authentication options, making the configuration more explicit and readable.
///
/// # Examples
///
/// ## PEAP with MSCHAPv2 (Common Corporate Setup)
///
/// ```rust
/// use nmrs::{EapOptions, EapMethod, Phase2};
///
/// let opts = EapOptions::builder()
///     .identity("employee@company.com")
///     .password("my_password")
///     .method(EapMethod::Peap)
///     .phase2(Phase2::Mschapv2)
///     .anonymous_identity("anonymous@company.com")
///     .domain_suffix_match("company.com")
///     .system_ca_certs(true)
///     .build();
/// ```
///
/// ## TTLS with PAP
///
/// ```rust
/// use nmrs::{EapOptions, EapMethod, Phase2};
///
/// let opts = EapOptions::builder()
///     .identity("student@university.edu")
///     .password("password")
///     .method(EapMethod::Ttls)
///     .phase2(Phase2::Pap)
///     .ca_cert_path("file:///etc/ssl/certs/university-ca.pem")
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct EapOptionsBuilder {
    identity: Option<String>,
    password: Option<String>,
    anonymous_identity: Option<String>,
    domain_suffix_match: Option<String>,
    ca_cert_path: Option<String>,
    system_ca_certs: bool,
    method: Option<EapMethod>,
    phase2: Option<Phase2>,
}

impl EapOptionsBuilder {
    /// Sets the user identity (usually email or username).
    ///
    /// This is a required field.
    pub fn identity(mut self, identity: impl Into<String>) -> Self {
        self.identity = Some(identity.into());
        self
    }

    /// Sets the password for authentication.
    ///
    /// This is a required field.
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the anonymous outer identity for privacy.
    ///
    /// This identity is sent in the clear during the initial handshake,
    /// while the real identity is protected inside the TLS tunnel.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::EapOptions;
    ///
    /// let builder = EapOptions::builder()
    ///     .anonymous_identity("anonymous@company.com");
    /// ```
    pub fn anonymous_identity(mut self, anonymous_identity: impl Into<String>) -> Self {
        self.anonymous_identity = Some(anonymous_identity.into());
        self
    }

    /// Sets the domain suffix to match against the server certificate.
    ///
    /// This provides additional security by verifying the server's certificate
    /// matches the expected domain.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::EapOptions;
    ///
    /// let builder = EapOptions::builder()
    ///     .domain_suffix_match("company.com");
    /// ```
    pub fn domain_suffix_match(mut self, domain: impl Into<String>) -> Self {
        self.domain_suffix_match = Some(domain.into());
        self
    }

    /// Sets the path to the CA certificate file.
    ///
    /// The path must start with `file://` (e.g., "file:///etc/ssl/certs/ca.pem").
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::EapOptions;
    ///
    /// let builder = EapOptions::builder()
    ///     .ca_cert_path("file:///etc/ssl/certs/company-ca.pem");
    /// ```
    pub fn ca_cert_path(mut self, path: impl Into<String>) -> Self {
        self.ca_cert_path = Some(path.into());
        self
    }

    /// Sets whether to use the system CA certificate store.
    ///
    /// When enabled, the system's trusted CA certificates will be used
    /// to validate the server certificate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::EapOptions;
    ///
    /// let builder = EapOptions::builder()
    ///     .system_ca_certs(true);
    /// ```
    pub fn system_ca_certs(mut self, use_system: bool) -> Self {
        self.system_ca_certs = use_system;
        self
    }

    /// Sets the EAP method (PEAP or TTLS).
    ///
    /// This is a required field. PEAP is more common in corporate environments,
    /// while TTLS offers more flexibility in inner authentication methods.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{EapOptions, EapMethod};
    ///
    /// let builder = EapOptions::builder()
    ///     .method(EapMethod::Peap);
    /// ```
    pub fn method(mut self, method: EapMethod) -> Self {
        self.method = Some(method);
        self
    }

    /// Sets the Phase 2 (inner) authentication method.
    ///
    /// This is a required field. MSCHAPv2 is commonly used with PEAP,
    /// while PAP is often used with TTLS.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{EapOptions, Phase2};
    ///
    /// let builder = EapOptions::builder()
    ///     .phase2(Phase2::Mschapv2);
    /// ```
    pub fn phase2(mut self, phase2: Phase2) -> Self {
        self.phase2 = Some(phase2);
        self
    }

    /// Builds the `EapOptions` from the configured values.
    ///
    /// # Panics
    ///
    /// Panics if any required field is missing:
    /// - `identity` (use [`identity()`](Self::identity))
    /// - `password` (use [`password()`](Self::password))
    /// - `method` (use [`method()`](Self::method))
    /// - `phase2` (use [`phase2()`](Self::phase2))
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{EapOptions, EapMethod, Phase2};
    ///
    /// let opts = EapOptions::builder()
    ///     .identity("user@example.com")
    ///     .password("password")
    ///     .method(EapMethod::Peap)
    ///     .phase2(Phase2::Mschapv2)
    ///     .build();
    /// ```
    pub fn build(self) -> EapOptions {
        EapOptions {
            identity: self
                .identity
                .expect("identity is required (use .identity())"),
            password: self
                .password
                .expect("password is required (use .password())"),
            anonymous_identity: self.anonymous_identity,
            domain_suffix_match: self.domain_suffix_match,
            ca_cert_path: self.ca_cert_path,
            system_ca_certs: self.system_ca_certs,
            method: self.method.expect("method is required (use .method())"),
            phase2: self.phase2.expect("phase2 is required (use .phase2())"),
        }
    }
}

/// Timeout configuration for NetworkManager operations.
///
/// Controls how long NetworkManager will wait for various network operations
/// to complete before timing out. This allows customization for different
/// network environments (slow networks, enterprise auth, etc.).
///
/// # Examples
///
/// ```rust
/// use nmrs::TimeoutConfig;
/// use std::time::Duration;
///
/// // Use default timeouts (30s connect, 10s disconnect)
/// let config = TimeoutConfig::default();
///
/// // Custom timeouts for slow networks
/// let config = TimeoutConfig::new()
///     .with_connection_timeout(Duration::from_secs(60))
///     .with_disconnect_timeout(Duration::from_secs(20));
///
/// // Quick timeouts for fast networks
/// let config = TimeoutConfig::new()
///     .with_connection_timeout(Duration::from_secs(15))
///     .with_disconnect_timeout(Duration::from_secs(5));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct TimeoutConfig {
    /// Timeout for connection activation (default: 30 seconds)
    pub connection_timeout: Duration,
    /// Timeout for device disconnection (default: 10 seconds)
    pub disconnect_timeout: Duration,
}

impl Default for TimeoutConfig {
    /// Returns the default timeout configuration.
    ///
    /// Defaults:
    /// - `connection_timeout`: 30 seconds
    /// - `disconnect_timeout`: 10 seconds
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            disconnect_timeout: Duration::from_secs(10),
        }
    }
}

impl TimeoutConfig {
    /// Creates a new `TimeoutConfig` with default values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::TimeoutConfig;
    ///
    /// let config = TimeoutConfig::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the connection activation timeout.
    ///
    /// This controls how long to wait for a network connection to activate
    /// before giving up. Increase this for slow networks or enterprise
    /// authentication that may take longer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::TimeoutConfig;
    /// use std::time::Duration;
    ///
    /// let config = TimeoutConfig::new()
    ///     .with_connection_timeout(Duration::from_secs(60));
    /// ```
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Sets the disconnection timeout.
    ///
    /// This controls how long to wait for a device to disconnect before
    /// giving up.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::TimeoutConfig;
    /// use std::time::Duration;
    ///
    /// let config = TimeoutConfig::new()
    ///     .with_disconnect_timeout(Duration::from_secs(20));
    /// ```
    pub fn with_disconnect_timeout(mut self, timeout: Duration) -> Self {
        self.disconnect_timeout = timeout;
        self
    }
}

/// Connection options for saved NetworkManager connections.
///
/// Controls how NetworkManager handles saved connection profiles,
/// including automatic connection behavior.
///
/// # Examples
///
/// ```rust
/// use nmrs::ConnectionOptions;
///
/// // Basic auto-connect (using defaults)
/// let opts = ConnectionOptions::default();
///
/// // High-priority connection with retry limit
/// let opts_priority = ConnectionOptions::new(true)
///     .with_priority(10)  // Higher = more preferred
///     .with_retries(3);   // Retry up to 3 times
///
/// // Manual connection only
/// let opts_manual = ConnectionOptions::new(false);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct ConnectionOptions {
    /// Whether to automatically connect when available
    pub autoconnect: bool,
    /// Priority for auto-connection (higher = more preferred)
    pub autoconnect_priority: Option<i32>,
    /// Maximum number of auto-connect retry attempts
    pub autoconnect_retries: Option<i32>,
}

impl Default for ConnectionOptions {
    /// Returns the default connection options.
    ///
    /// Defaults:
    /// - `autoconnect`: `true`
    /// - `autoconnect_priority`: `None` (uses NetworkManager's default of 0)
    /// - `autoconnect_retries`: `None` (unlimited retries)
    fn default() -> Self {
        Self {
            autoconnect: true,
            autoconnect_priority: None,
            autoconnect_retries: None,
        }
    }
}

impl ConnectionOptions {
    /// Creates new `ConnectionOptions` with the specified autoconnect setting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::ConnectionOptions;
    ///
    /// let opts = ConnectionOptions::new(true);
    /// ```
    pub fn new(autoconnect: bool) -> Self {
        Self {
            autoconnect,
            autoconnect_priority: None,
            autoconnect_retries: None,
        }
    }

    /// Sets the auto-connection priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.autoconnect_priority = Some(priority);
        self
    }

    /// Sets the maximum number of auto-connect retry attempts.
    pub fn with_retries(mut self, retries: i32) -> Self {
        self.autoconnect_retries = Some(retries);
        self
    }
}

/// Wi-Fi connection security types.
///
/// Represents the authentication method for connecting to a WiFi network.
///
/// # Variants
///
/// - [`Open`](WifiSecurity::Open) - No authentication required (open network)
/// - [`WpaPsk`](WifiSecurity::WpaPsk) - WPA/WPA2/WPA3 Personal (password-based)
/// - [`WpaEap`](WifiSecurity::WpaEap) - WPA/WPA2 Enterprise (802.1X authentication)
///
/// # Examples
///
/// ## Open Network
///
/// ```rust
/// use nmrs::WifiSecurity;
///
/// let security = WifiSecurity::Open;
/// ```
///
/// ## Password-Protected Network
///
/// ```no_run
/// use nmrs::{NetworkManager, WifiSecurity};
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
///
/// nm.connect("HomeWiFi", WifiSecurity::WpaPsk {
///     psk: "my_secure_password".into()
/// }).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Enterprise Network (WPA-EAP)
///
/// ```no_run
/// use nmrs::{NetworkManager, WifiSecurity, EapOptions, EapMethod, Phase2};
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
///
/// let eap_opts = EapOptions::new("user@company.com", "password")
///     .with_domain_suffix_match("company.com")
///     .with_system_ca_certs(true)
///     .with_method(EapMethod::Peap)
///     .with_phase2(Phase2::Mschapv2);
///
/// nm.connect("CorpWiFi", WifiSecurity::WpaEap {
///     opts: eap_opts
/// }).await?;
/// # Ok(())
/// # }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WifiSecurity {
    /// Open network (no authentication)
    Open,
    /// WPA-PSK (password-based authentication)
    WpaPsk {
        /// Pre-shared key (password)
        psk: String,
    },
    /// WPA-EAP (Enterprise authentication via 802.1X)
    WpaEap {
        /// EAP configuration options
        opts: EapOptions,
    },
}

/// VPN connection type.
///
/// Identifies the VPN protocol/technology used for the connection.
/// Currently only WireGuard is supported.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VpnType {
    /// WireGuard - modern, high-performance VPN protocol.
    WireGuard,
}

/// VPN Credentials for establishing a VPN connection.
///
/// Stores the necessary information to configure and connect to a VPN.
/// Currently supports WireGuard VPN connections.
///
/// # Fields
///
/// - `vpn_type`: The type of VPN (currently only WireGuard)
/// - `name`: Unique identifier for the connection
/// - `gateway`: VPN gateway endpoint (e.g., "vpn.example.com:51820")
/// - `private_key`: Client's WireGuard private key
/// - `address`: Client's IP address with CIDR notation (e.g., "10.0.0.2/24")
/// - `peers`: List of WireGuard peers to connect to
/// - `dns`: Optional DNS servers to use (e.g., ["1.1.1.1", "8.8.8.8"])
/// - `mtu`: Optional Maximum Transmission Unit
/// - `uuid`: Optional UUID for the connection (auto-generated if not provided)
///
/// # Example
///
/// ```rust
/// use nmrs::{VpnCredentials, VpnType, WireGuardPeer};
///
/// let peer = WireGuardPeer::new(
///     "server_public_key",
///     "vpn.home.com:51820",
///     vec!["0.0.0.0/0".into()],
/// ).with_persistent_keepalive(25);
///
/// let creds = VpnCredentials::new(
///     VpnType::WireGuard,
///     "HomeVPN",
///     "vpn.home.com:51820",
///     "aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789=",
///     "10.0.0.2/24",
///     vec![peer],
/// ).with_dns(vec!["1.1.1.1".into()]);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct VpnCredentials {
    /// The type of VPN (currently only WireGuard).
    pub vpn_type: VpnType,
    /// Unique name for the connection profile.
    pub name: String,
    /// VPN gateway endpoint (e.g., "vpn.example.com:51820").
    pub gateway: String,
    /// Client's WireGuard private key (base64 encoded).
    pub private_key: String,
    /// Client's IP address with CIDR notation (e.g., "10.0.0.2/24").
    pub address: String,
    /// List of WireGuard peers to connect to.
    pub peers: Vec<WireGuardPeer>,
    /// Optional DNS servers to use when connected.
    pub dns: Option<Vec<String>>,
    /// Optional Maximum Transmission Unit size.
    pub mtu: Option<u32>,
    /// Optional UUID for the connection (auto-generated if not provided).
    pub uuid: Option<Uuid>,
}

impl VpnCredentials {
    /// Creates new `VpnCredentials` with the required fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{VpnCredentials, VpnType, WireGuardPeer};
    ///
    /// let peer = WireGuardPeer::new(
    ///     "server_public_key",
    ///     "vpn.example.com:51820",
    ///     vec!["0.0.0.0/0".into()],
    /// );
    ///
    /// let creds = VpnCredentials::new(
    ///     VpnType::WireGuard,
    ///     "MyVPN",
    ///     "vpn.example.com:51820",
    ///     "client_private_key",
    ///     "10.0.0.2/24",
    ///     vec![peer],
    /// );
    /// ```
    pub fn new(
        vpn_type: VpnType,
        name: impl Into<String>,
        gateway: impl Into<String>,
        private_key: impl Into<String>,
        address: impl Into<String>,
        peers: Vec<WireGuardPeer>,
    ) -> Self {
        Self {
            vpn_type,
            name: name.into(),
            gateway: gateway.into(),
            private_key: private_key.into(),
            address: address.into(),
            peers,
            dns: None,
            mtu: None,
            uuid: None,
        }
    }

    /// Creates a new `VpnCredentials` builder.
    ///
    /// This provides a more ergonomic way to construct VPN credentials with a fluent API,
    /// making it harder to mix up parameter order and easier to see what each value represents.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{VpnCredentials, VpnType, WireGuardPeer};
    ///
    /// let peer = WireGuardPeer::new(
    ///     "server_public_key",
    ///     "vpn.example.com:51820",
    ///     vec!["0.0.0.0/0".into()],
    /// );
    ///
    /// let creds = VpnCredentials::builder()
    ///     .name("MyVPN")
    ///     .wireguard()
    ///     .gateway("vpn.example.com:51820")
    ///     .private_key("client_private_key")
    ///     .address("10.0.0.2/24")
    ///     .add_peer(peer)
    ///     .with_dns(vec!["1.1.1.1".into()])
    ///     .build();
    /// ```
    pub fn builder() -> VpnCredentialsBuilder {
        VpnCredentialsBuilder::default()
    }

    /// Sets the DNS servers to use when connected.
    pub fn with_dns(mut self, dns: Vec<String>) -> Self {
        self.dns = Some(dns);
        self
    }

    /// Sets the MTU (Maximum Transmission Unit) size.
    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Sets the UUID for the connection.
    pub fn with_uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }
}

/// Builder for constructing `VpnCredentials` with a fluent API.
///
/// This builder provides a more ergonomic way to create VPN credentials,
/// making the code more readable and less error-prone compared to the
/// traditional constructor with many positional parameters.
///
/// # Examples
///
/// ## Basic WireGuard VPN
///
/// ```rust
/// use nmrs::{VpnCredentials, WireGuardPeer};
///
/// let peer = WireGuardPeer::new(
///     "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
///     "vpn.example.com:51820",
///     vec!["0.0.0.0/0".into()],
/// );
///
/// let creds = VpnCredentials::builder()
///     .name("HomeVPN")
///     .wireguard()
///     .gateway("vpn.example.com:51820")
///     .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
///     .address("10.0.0.2/24")
///     .add_peer(peer)
///     .build();
/// ```
///
/// ## With Optional DNS and MTU
///
/// ```rust
/// use nmrs::{VpnCredentials, WireGuardPeer};
///
/// let peer = WireGuardPeer::new(
///     "server_public_key",
///     "vpn.example.com:51820",
///     vec!["0.0.0.0/0".into()],
/// ).with_persistent_keepalive(25);
///
/// let creds = VpnCredentials::builder()
///     .name("CorpVPN")
///     .wireguard()
///     .gateway("vpn.corp.com:51820")
///     .private_key("private_key_here")
///     .address("10.8.0.2/24")
///     .add_peer(peer)
///     .with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()])
///     .with_mtu(1420)
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct VpnCredentialsBuilder {
    vpn_type: Option<VpnType>,
    name: Option<String>,
    gateway: Option<String>,
    private_key: Option<String>,
    address: Option<String>,
    peers: Vec<WireGuardPeer>,
    dns: Option<Vec<String>>,
    mtu: Option<u32>,
    uuid: Option<Uuid>,
}

impl VpnCredentialsBuilder {
    /// Sets the VPN type to WireGuard.
    ///
    /// Currently, WireGuard is the only supported VPN type.
    pub fn wireguard(mut self) -> Self {
        self.vpn_type = Some(VpnType::WireGuard);
        self
    }

    /// Sets the VPN type.
    ///
    /// For most use cases, prefer using [`wireguard()`](Self::wireguard) instead.
    pub fn vpn_type(mut self, vpn_type: VpnType) -> Self {
        self.vpn_type = Some(vpn_type);
        self
    }

    /// Sets the connection name.
    ///
    /// This is the unique identifier for the VPN connection profile.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the VPN gateway endpoint.
    ///
    /// Should be in "host:port" format (e.g., "vpn.example.com:51820").
    pub fn gateway(mut self, gateway: impl Into<String>) -> Self {
        self.gateway = Some(gateway.into());
        self
    }

    /// Sets the client's WireGuard private key.
    ///
    /// The private key should be base64 encoded.
    pub fn private_key(mut self, private_key: impl Into<String>) -> Self {
        self.private_key = Some(private_key.into());
        self
    }

    /// Sets the client's IP address with CIDR notation.
    ///
    /// # Examples
    ///
    /// - "10.0.0.2/24" for a /24 subnet
    /// - "192.168.1.10/32" for a single IP
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Adds a WireGuard peer to the connection.
    ///
    /// Multiple peers can be added by calling this method multiple times.
    pub fn add_peer(mut self, peer: WireGuardPeer) -> Self {
        self.peers.push(peer);
        self
    }

    /// Sets all WireGuard peers at once.
    ///
    /// This replaces any previously added peers.
    pub fn peers(mut self, peers: Vec<WireGuardPeer>) -> Self {
        self.peers = peers;
        self
    }

    /// Sets the DNS servers to use when connected.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::VpnCredentials;
    ///
    /// let builder = VpnCredentials::builder()
    ///     .with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()]);
    /// ```
    pub fn with_dns(mut self, dns: Vec<String>) -> Self {
        self.dns = Some(dns);
        self
    }

    /// Sets the MTU (Maximum Transmission Unit) size.
    ///
    /// Typical values are 1420 for WireGuard over standard networks.
    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Sets a specific UUID for the connection.
    ///
    /// If not set, NetworkManager will generate one automatically.
    pub fn with_uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    /// Builds the `VpnCredentials` from the configured values.
    ///
    /// # Panics
    ///
    /// Panics if any required field is missing:
    /// - `vpn_type` (use [`wireguard()`](Self::wireguard))
    /// - `name` (use [`name()`](Self::name))
    /// - `gateway` (use [`gateway()`](Self::gateway))
    /// - `private_key` (use [`private_key()`](Self::private_key))
    /// - `address` (use [`address()`](Self::address))
    /// - At least one peer must be added (use [`add_peer()`](Self::add_peer))
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{VpnCredentials, WireGuardPeer};
    ///
    /// let peer = WireGuardPeer::new(
    ///     "public_key",
    ///     "vpn.example.com:51820",
    ///     vec!["0.0.0.0/0".into()],
    /// );
    ///
    /// let creds = VpnCredentials::builder()
    ///     .name("MyVPN")
    ///     .wireguard()
    ///     .gateway("vpn.example.com:51820")
    ///     .private_key("private_key")
    ///     .address("10.0.0.2/24")
    ///     .add_peer(peer)
    ///     .build();
    /// ```
    pub fn build(self) -> VpnCredentials {
        VpnCredentials {
            vpn_type: self
                .vpn_type
                .expect("vpn_type is required (use .wireguard())"),
            name: self.name.expect("name is required (use .name())"),
            gateway: self.gateway.expect("gateway is required (use .gateway())"),
            private_key: self
                .private_key
                .expect("private_key is required (use .private_key())"),
            address: self.address.expect("address is required (use .address())"),
            peers: {
                if self.peers.is_empty() {
                    panic!("at least one peer is required (use .add_peer())");
                }
                self.peers
            },
            dns: self.dns,
            mtu: self.mtu,
            uuid: self.uuid,
        }
    }
}

/// WireGuard peer configuration.
///
/// Represents a single WireGuard peer (server) to connect to.
///
/// # Fields
///
/// - `public_key`: The peer's WireGuard public key
/// - `gateway`: Peer endpoint in "host:port" format (e.g., "vpn.example.com:51820")
/// - `allowed_ips`: List of IP ranges allowed through this peer (e.g., ["0.0.0.0/0"])
/// - `preshared_key`: Optional pre-shared key for additional security
/// - `persistent_keepalive`: Optional keepalive interval in seconds (e.g., 25)
///
/// # Example
///
/// ```rust
/// use nmrs::WireGuardPeer;
///
/// let peer = WireGuardPeer::new(
///     "aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789=",
///     "vpn.example.com:51820",
///     vec!["0.0.0.0/0".into(), "::/0".into()],
/// );
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct WireGuardPeer {
    /// The peer's WireGuard public key (base64 encoded).
    pub public_key: String,
    /// Peer endpoint in "host:port" format.
    pub gateway: String,
    /// IP ranges to route through this peer (e.g., ["0.0.0.0/0"]).
    pub allowed_ips: Vec<String>,
    /// Optional pre-shared key for additional security.
    pub preshared_key: Option<String>,
    /// Optional keepalive interval in seconds (e.g., 25).
    pub persistent_keepalive: Option<u32>,
}

impl WireGuardPeer {
    /// Creates a new `WireGuardPeer` with the required fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::WireGuardPeer;
    ///
    /// let peer = WireGuardPeer::new(
    ///     "aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789=",
    ///     "vpn.example.com:51820",
    ///     vec!["0.0.0.0/0".into()],
    /// );
    /// ```
    pub fn new(
        public_key: impl Into<String>,
        gateway: impl Into<String>,
        allowed_ips: Vec<String>,
    ) -> Self {
        Self {
            public_key: public_key.into(),
            gateway: gateway.into(),
            allowed_ips,
            preshared_key: None,
            persistent_keepalive: None,
        }
    }

    /// Sets the pre-shared key for additional security.
    pub fn with_preshared_key(mut self, psk: impl Into<String>) -> Self {
        self.preshared_key = Some(psk.into());
        self
    }

    /// Sets the persistent keepalive interval in seconds.
    pub fn with_persistent_keepalive(mut self, interval: u32) -> Self {
        self.persistent_keepalive = Some(interval);
        self
    }
}

/// VPN Connection information.
///
/// Represents a VPN connection managed by NetworkManager, including both
/// saved and active connections.
///
/// # Fields
///
/// - `name`: The connection name/identifier
/// - `vpn_type`: The type of VPN (WireGuard, etc.)
/// - `state`: Current connection state (for active connections)
/// - `interface`: Network interface name (e.g., "wg0") when active
///
/// # Example
///
/// ```no_run
/// # use nmrs::{VpnConnection, VpnType, DeviceState};
/// # // This struct is returned by the library, not constructed directly
/// # let vpn: VpnConnection = todo!();
/// println!("VPN: {}, State: {:?}", vpn.name, vpn.state);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct VpnConnection {
    /// The connection name/identifier.
    pub name: String,
    /// The type of VPN (WireGuard, etc.).
    pub vpn_type: VpnType,
    /// Current connection state.
    pub state: DeviceState,
    /// Network interface name when active (e.g., "wg0").
    pub interface: Option<String>,
}

/// Detailed VPN connection information and statistics.
///
/// Provides comprehensive information about an active VPN connection,
/// including IP configuration and connection details.
///
/// # Example
///
/// ```no_run
/// # use nmrs::{VpnConnectionInfo, VpnType, DeviceState};
/// # // This struct is returned by the library, not constructed directly
/// # let info: VpnConnectionInfo = todo!();
/// if let Some(ip) = &info.ip4_address {
///     println!("VPN IPv4: {}", ip);
/// }
/// if let Some(ip) = &info.ip6_address {
///     println!("VPN IPv6: {}", ip);
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct VpnConnectionInfo {
    /// The connection name/identifier.
    pub name: String,
    /// The type of VPN (WireGuard, etc.).
    pub vpn_type: VpnType,
    /// Current connection state.
    pub state: DeviceState,
    /// Network interface name when active (e.g., "wg0").
    pub interface: Option<String>,
    /// VPN gateway endpoint address.
    pub gateway: Option<String>,
    /// Assigned IPv4 address with CIDR notation.
    pub ip4_address: Option<String>,
    /// Assigned IPv6 address with CIDR notation.
    pub ip6_address: Option<String>,
    /// DNS servers configured for this VPN.
    pub dns_servers: Vec<String>,
}

/// Bluetooth network role.
///
/// Specifies the role of the Bluetooth device in the network connection.
///
/// # Stability
///
/// This enum is marked as `#[non_exhaustive]` so as to assume that new Bluetooth roles may be
/// added in future versions. When pattern matching, always include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothNetworkRole {
    PanU, // Personal Area Network User
    Dun,  // Dial-Up Networking
}

/// Bluetooth device identity information.
///
/// Relevant info for Bluetooth devices managed by NetworkManager.
///
/// # Example
///```rust
/// use nmrs::models::{BluetoothIdentity, BluetoothNetworkRole};
///
/// let bt_settings = BluetoothIdentity::new(
///    "00:1A:7D:DA:71:13".into(),
///    BluetoothNetworkRole::Dun,
/// ).unwrap();
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct BluetoothIdentity {
    /// MAC address of Bluetooth device
    pub bdaddr: String,
    /// Bluetooth device type (DUN or PANU)
    pub bt_device_type: BluetoothNetworkRole,
}

impl BluetoothIdentity {
    /// Creates a new `BluetoothIdentity`.
    ///
    /// # Arguments
    ///
    /// * `bdaddr` - Bluetooth MAC address (e.g., "00:1A:7D:DA:71:13")
    /// * `bt_device_type` - Bluetooth network role (PanU or Dun)
    ///
    /// # Errors
    ///
    /// Returns a `ConnectionError` if the provided `bdaddr` is not a
    /// valid Bluetooth MAC address format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nmrs::models::{BluetoothIdentity, BluetoothNetworkRole};
    ///
    /// let identity = BluetoothIdentity::new(
    ///     "00:1A:7D:DA:71:13".into(),
    ///     BluetoothNetworkRole::PanU,
    /// ).unwrap();
    /// ```
    pub fn new(
        bdaddr: String,
        bt_device_type: BluetoothNetworkRole,
    ) -> Result<Self, ConnectionError> {
        validate_bluetooth_address(&bdaddr)?;
        Ok(Self {
            bdaddr,
            bt_device_type,
        })
    }
}

/// Bluetooth device with friendly name from BlueZ.
///
/// Contains information about a Bluetooth device managed by NetworkManager,
/// proxying data from BlueZ.
///
/// This is a specialized struct for Bluetooth devices, separate from the
/// general `Device` struct.
///
/// # Example
///
/// # Example
///
/// ```rust
/// use nmrs::models::{BluetoothDevice, BluetoothNetworkRole, DeviceState};
///
/// let role = BluetoothNetworkRole::PanU as u32;
/// let device = BluetoothDevice::new(
///     "00:1A:7D:DA:71:13".into(),
///     Some("My Phone".into()),
///     Some("Phone".into()),
///     role,
///     DeviceState::Activated,
/// );
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    /// Bluetooth MAC address
    pub bdaddr: String,
    /// Friendly device name from BlueZ
    pub name: Option<String>,
    /// Device alias from BlueZ
    pub alias: Option<String>,
    /// Bluetooth device type (DUN or PANU)
    pub bt_caps: u32,
    /// Current device state
    pub state: DeviceState,
}

impl BluetoothDevice {
    /// Creates a new `BluetoothDevice`.
    ///
    /// # Arguments
    ///
    /// * `bdaddr` - Bluetooth MAC address
    /// * `name` - Friendly device name from BlueZ
    /// * `alias` - Device alias from BlueZ
    /// * `bt_caps` - Bluetooth device capabilities/type
    /// * `state` - Current device state
    ///
    /// # Example
    ///
    /// ```rust
    /// use nmrs::models::{BluetoothDevice, BluetoothNetworkRole, DeviceState};
    ///
    /// let role = BluetoothNetworkRole::PanU as u32;
    /// let device = BluetoothDevice::new(
    ///     "00:1A:7D:DA:71:13".into(),
    ///     Some("My Phone".into()),
    ///     Some("Phone".into()),
    ///     role,
    ///     DeviceState::Activated,
    /// );
    /// ```
    pub fn new(
        bdaddr: String,
        name: Option<String>,
        alias: Option<String>,
        bt_caps: u32,
        state: DeviceState,
    ) -> Self {
        Self {
            bdaddr,
            name,
            alias,
            bt_caps,
            state,
        }
    }
}

/// NetworkManager device types.
///
/// Represents the type of network hardware managed by NetworkManager.
/// This enum uses a registry-based system to support adding new device
/// types without breaking the API.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    /// Wired Ethernet device.
    Ethernet,
    /// Wi-Fi (802.11) wireless device.
    Wifi,
    /// Wi-Fi P2P (peer-to-peer) device.
    WifiP2P,
    /// Loopback device (localhost).
    Loopback,
    /// Bluetooth
    Bluetooth,
    /// Unknown or unsupported device type with raw code.
    ///
    /// Use the methods on `DeviceType` to query capabilities of unknown device types,
    /// which will consult the internal device type registry.
    Other(u32),
}

impl DeviceType {
    /// Returns whether this device type supports network scanning.
    ///
    /// Currently only WiFi and WiFi P2P devices support scanning.
    /// For unknown device types, consults the internal device type registry.
    pub fn supports_scanning(&self) -> bool {
        match self {
            Self::Wifi | Self::WifiP2P => true,
            Self::Other(code) => crate::types::device_type_registry::supports_scanning(*code),
            _ => false,
        }
    }

    /// Returns whether this device type requires a specific object (like an access point).
    ///
    /// WiFi devices require an access point to connect to, while Ethernet can connect
    /// without a specific target.
    /// For unknown device types, consults the internal device type registry.
    pub fn requires_specific_object(&self) -> bool {
        match self {
            Self::Wifi | Self::WifiP2P => true,
            Self::Other(code) => {
                crate::types::device_type_registry::requires_specific_object(*code)
            }
            _ => false,
        }
    }

    /// Returns whether this device type has a global enabled/disabled state.
    ///
    /// WiFi has a global radio killswitch that can enable/disable all WiFi devices.
    /// For unknown device types, consults the internal device type registry.
    pub fn has_global_enabled_state(&self) -> bool {
        match self {
            Self::Wifi => true,
            Self::Other(code) => {
                crate::types::device_type_registry::has_global_enabled_state(*code)
            }
            _ => false,
        }
    }

    /// Returns the NetworkManager connection type string for this device.
    ///
    /// This is used when creating connection profiles for this device type.
    /// For unknown device types, consults the internal device type registry.
    pub fn connection_type_str(&self) -> &'static str {
        match self {
            Self::Ethernet => "802-3-ethernet",
            Self::Wifi => "802-11-wireless",
            Self::WifiP2P => "wifi-p2p",
            Self::Loopback => "loopback",
            Self::Bluetooth => "bluetooth",
            Self::Other(code) => {
                crate::types::device_type_registry::connection_type_for_code(*code)
                    .unwrap_or("generic")
            }
        }
    }

    /// Returns the raw NetworkManager type code for this device.
    pub fn to_code(&self) -> u32 {
        match self {
            Self::Ethernet => 1,
            Self::Wifi => 2,
            Self::WifiP2P => 30,
            Self::Loopback => 32,
            Self::Bluetooth => 6,
            Self::Other(code) => *code,
        }
    }
}

/// NetworkManager device states.
///
/// Represents the current operational state of a network device.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceState {
    /// Device is not managed by NetworkManager.
    Unmanaged,
    /// Device is managed but not yet available (e.g., Wi-Fi disabled).
    Unavailable,
    /// Device is available but not connected.
    Disconnected,
    /// Device is preparing to connect.
    Prepare,
    /// Device is being configured (IP, etc.).
    Config,
    /// Device is fully connected and operational.
    Activated,
    /// Device is disconnecting.
    Deactivating,
    /// Device connection failed.
    Failed,
    /// Unknown or unsupported state with raw code.
    Other(u32),
}

impl Device {
    /// Returns `true` if this is a wired (Ethernet) device.
    pub fn is_wired(&self) -> bool {
        matches!(self.device_type, DeviceType::Ethernet)
    }

    /// Returns `true` if this is a wireless (Wi-Fi) device.
    pub fn is_wireless(&self) -> bool {
        matches!(self.device_type, DeviceType::Wifi)
    }

    /// Returns 'true' if this is a Bluetooth (DUN or PANU) device.
    pub fn is_bluetooth(&self) -> bool {
        matches!(self.device_type, DeviceType::Bluetooth)
    }
}

/// Display implementation for Device struct.
///
/// Formats the device information as "interface (device_type) [state]".
impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) [{}]",
            self.interface, self.device_type, self.state
        )
    }
}

/// Display implementation for BluetoothDevice struct.
///
/// Formats the device information as "alias (device_type) [bdaddr]".
impl Display for BluetoothDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let role = BluetoothNetworkRole::from(self.bt_caps);
        write!(
            f,
            "{} ({}) [{}]",
            self.alias.as_deref().unwrap_or("unknown"),
            role,
            self.bdaddr
        )
    }
}

/// Display implementation for Device struct.
///
/// Formats the device information as "interface (device_type) [state]".
impl Display for BluetoothNetworkRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BluetoothNetworkRole::Dun => write!(f, "DUN"),
            BluetoothNetworkRole::PanU => write!(f, "PANU"),
        }
    }
}

/// Errors that can occur during network operations.
///
/// This enum provides specific error types for different failure modes,
/// making it easy to handle errors appropriately in your application.
///
/// # Examples
///
/// ## Basic Error Handling
///
/// ```no_run
/// use nmrs::{NetworkManager, WifiSecurity, ConnectionError};
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
///
/// match nm.connect("MyNetwork", WifiSecurity::WpaPsk {
///     psk: "password".into()
/// }).await {
///     Ok(_) => println!("Connected!"),
///     Err(ConnectionError::AuthFailed) => {
///         eprintln!("Wrong password");
///     }
///     Err(ConnectionError::NotFound) => {
///         eprintln!("Network not in range");
///     }
///     Err(ConnectionError::Timeout) => {
///         eprintln!("Connection timed out");
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Retry Logic
///
/// ```no_run
/// use nmrs::{NetworkManager, WifiSecurity, ConnectionError};
///
/// # async fn example() -> nmrs::Result<()> {
/// let nm = NetworkManager::new().await?;
///
/// for attempt in 1..=3 {
///     match nm.connect("MyNetwork", WifiSecurity::Open).await {
///         Ok(_) => {
///             println!("Connected on attempt {}", attempt);
///             break;
///         }
///         Err(ConnectionError::Timeout) if attempt < 3 => {
///             println!("Timeout, retrying...");
///             continue;
///         }
///         Err(e) => return Err(e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ConnectionError {
    /// A D-Bus communication error occurred.
    #[error("D-Bus error: {0}")]
    Dbus(#[from] zbus::Error),

    /// The requested network was not found during scan.
    #[error("network not found")]
    NotFound,

    /// Authentication with the access point failed (wrong password, rejected credentials).
    #[error("authentication failed")]
    AuthFailed,

    /// The supplicant (wpa_supplicant) encountered a configuration error.
    #[error("supplicant configuration failed")]
    SupplicantConfigFailed,

    /// The supplicant timed out during authentication.
    #[error("supplicant timeout")]
    SupplicantTimeout,

    /// DHCP failed to obtain an IP address.
    #[error("DHCP failed")]
    DhcpFailed,

    /// The connection timed out waiting for activation.
    #[error("connection timeout")]
    Timeout,

    /// The connection is stuck in an unexpected state.
    #[error("connection stuck in state: {0}")]
    Stuck(String),

    /// No Wi-Fi device was found on the system.
    #[error("no Wi-Fi device found")]
    NoWifiDevice,

    /// No wired (ethernet) device was found on the system.
    #[error("no wired device was found")]
    NoWiredDevice,

    /// Wi-Fi device did not become ready in time.
    #[error("Wi-Fi device not ready")]
    WifiNotReady,

    /// No saved connection exists for the requested network.
    #[error("no saved connection for network")]
    NoSavedConnection,

    /// An empty password was provided for the requested network.
    #[error("no password was provided")]
    MissingPassword,

    /// A general connection failure with a device state reason code.
    #[error("connection failed: {0}")]
    DeviceFailed(StateReason),

    /// A connection activation failure with a connection state reason.
    #[error("connection activation failed: {0}")]
    ActivationFailed(ConnectionStateReason),

    /// Invalid UTF-8 encountered in SSID.
    #[error("invalid UTF-8 in SSID: {0}")]
    InvalidUtf8(#[from] std::str::Utf8Error),

    /// No VPN connection found
    #[error("no VPN connection found")]
    NoVpnConnection,

    /// Invalid IP address or CIDR notation
    #[error("invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid VPN peer configuration
    #[error("invalid peer configuration: {0}")]
    InvalidPeers(String),

    /// Invalid WireGuard private key format
    #[error("invalid WireGuard private key: {0}")]
    InvalidPrivateKey(String),

    /// Invalid WireGuard public key format
    #[error("invalid WireGuard public key: {0}")]
    InvalidPublicKey(String),

    /// Invalid VPN gateway format (should be host:port)
    #[error("invalid VPN gateway: {0}")]
    InvalidGateway(String),

    /// VPN connection failed
    #[error("VPN connection failed: {0}")]
    VpnFailed(String),

    /// Bluetooth device not found
    #[error("Bluetooth device not found")]
    NoBluetoothDevice,

    /// A D-Bus operation failed with context about what was being attempted
    #[error("{context}: {source}")]
    DbusOperation {
        context: String,
        #[source]
        source: zbus::Error,
    },
}

/// NetworkManager device state reason codes.
impl From<u32> for StateReason {
    fn from(code: u32) -> Self {
        match code {
            0 => Self::Unknown,
            1 => Self::None,
            2 => Self::UserDisconnected,
            3 => Self::DeviceDisconnected,
            4 => Self::CarrierChanged,
            7 => Self::SupplicantDisconnected,
            8 => Self::SupplicantConfigFailed,
            9 => Self::SupplicantFailed,
            10 => Self::SupplicantTimeout,
            11 => Self::PppStartFailed,
            15 => Self::DhcpStartFailed,
            16 => Self::DhcpError,
            17 => Self::DhcpFailed,
            24 => Self::ModemConnectionFailed,
            25 => Self::ModemInitFailed,
            42 => Self::InfinibandMode,
            43 => Self::DependencyFailed,
            44 => Self::Br2684Failed,
            45 => Self::ModeSetFailed,
            46 => Self::GsmApnSelectFailed,
            47 => Self::GsmNotSearching,
            48 => Self::GsmRegistrationDenied,
            49 => Self::GsmRegistrationTimeout,
            50 => Self::GsmRegistrationFailed,
            51 => Self::GsmPinCheckFailed,
            52 => Self::FirmwareMissing,
            53 => Self::DeviceRemoved,
            54 => Self::Sleeping,
            55 => Self::ConnectionRemoved,
            56 => Self::UserRequested,
            57 => Self::Carrier,
            58 => Self::ConnectionAssumed,
            59 => Self::SupplicantAvailable,
            60 => Self::ModemNotFound,
            61 => Self::BluetoothFailed,
            62 => Self::GsmSimNotInserted,
            63 => Self::GsmSimPinRequired,
            64 => Self::GsmSimPukRequired,
            65 => Self::GsmSimWrong,
            70 => Self::SsidNotFound,
            71 => Self::SecondaryConnectionFailed,
            72 => Self::DcbFcoeFailed,
            73 => Self::TeamdControlFailed,
            74 => Self::ModemFailed,
            75 => Self::ModemAvailable,
            76 => Self::SimPinIncorrect,
            77 => Self::NewActivationEnqueued,
            78 => Self::ParentUnreachable,
            79 => Self::ParentChanged,
            v => Self::Other(v),
        }
    }
}

/// Display implementation for StateReason.
impl Display for StateReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::None => write!(f, "none"),
            Self::UserDisconnected => write!(f, "user disconnected"),
            Self::DeviceDisconnected => write!(f, "device disconnected"),
            Self::CarrierChanged => write!(f, "carrier changed"),
            Self::SupplicantDisconnected => write!(f, "supplicant disconnected"),
            Self::SupplicantConfigFailed => write!(f, "supplicant config failed"),
            Self::SupplicantFailed => write!(f, "supplicant failed"),
            Self::SupplicantTimeout => write!(f, "supplicant timeout"),
            Self::PppStartFailed => write!(f, "PPP start failed"),
            Self::DhcpStartFailed => write!(f, "DHCP start failed"),
            Self::DhcpError => write!(f, "DHCP error"),
            Self::DhcpFailed => write!(f, "DHCP failed"),
            Self::ModemConnectionFailed => write!(f, "modem connection failed"),
            Self::ModemInitFailed => write!(f, "modem init failed"),
            Self::InfinibandMode => write!(f, "infiniband mode"),
            Self::DependencyFailed => write!(f, "dependency failed"),
            Self::Br2684Failed => write!(f, "BR2684 failed"),
            Self::ModeSetFailed => write!(f, "mode set failed"),
            Self::GsmApnSelectFailed => write!(f, "GSM APN select failed"),
            Self::GsmNotSearching => write!(f, "GSM not searching"),
            Self::GsmRegistrationDenied => write!(f, "GSM registration denied"),
            Self::GsmRegistrationTimeout => write!(f, "GSM registration timeout"),
            Self::GsmRegistrationFailed => write!(f, "GSM registration failed"),
            Self::GsmPinCheckFailed => write!(f, "GSM PIN check failed"),
            Self::FirmwareMissing => write!(f, "firmware missing"),
            Self::DeviceRemoved => write!(f, "device removed"),
            Self::Sleeping => write!(f, "sleeping"),
            Self::ConnectionRemoved => write!(f, "connection removed"),
            Self::UserRequested => write!(f, "user requested"),
            Self::Carrier => write!(f, "carrier"),
            Self::ConnectionAssumed => write!(f, "connection assumed"),
            Self::SupplicantAvailable => write!(f, "supplicant available"),
            Self::ModemNotFound => write!(f, "modem not found"),
            Self::BluetoothFailed => write!(f, "bluetooth failed"),
            Self::GsmSimNotInserted => write!(f, "GSM SIM not inserted"),
            Self::GsmSimPinRequired => write!(f, "GSM SIM PIN required"),
            Self::GsmSimPukRequired => write!(f, "GSM SIM PUK required"),
            Self::GsmSimWrong => write!(f, "GSM SIM wrong"),
            Self::SsidNotFound => write!(f, "SSID not found"),
            Self::SecondaryConnectionFailed => write!(f, "secondary connection failed"),
            Self::DcbFcoeFailed => write!(f, "DCB/FCoE setup failed"),
            Self::TeamdControlFailed => write!(f, "teamd control failed"),
            Self::ModemFailed => write!(f, "modem failed"),
            Self::ModemAvailable => write!(f, "modem available"),
            Self::SimPinIncorrect => write!(f, "SIM PIN incorrect"),
            Self::NewActivationEnqueued => write!(f, "new activation enqueued"),
            Self::ParentUnreachable => write!(f, "parent device unreachable"),
            Self::ParentChanged => write!(f, "parent device changed"),
            Self::Other(v) => write!(f, "unknown reason ({v})"),
        }
    }
}

/// Converts a NetworkManager state reason code to a specific `ConnectionError`.
///
/// Maps authentication-related failures to `AuthFailed`, DHCP issues to `DhcpFailed`,
/// and other failures to the appropriate variant.
pub fn reason_to_error(code: u32) -> ConnectionError {
    let reason = StateReason::from(code);
    match reason {
        // Authentication failures
        StateReason::SupplicantFailed
        | StateReason::SupplicantDisconnected
        | StateReason::SimPinIncorrect
        | StateReason::GsmPinCheckFailed => ConnectionError::AuthFailed,

        // Supplicant configuration issues
        StateReason::SupplicantConfigFailed => ConnectionError::SupplicantConfigFailed,

        // Supplicant timeout
        StateReason::SupplicantTimeout => ConnectionError::SupplicantTimeout,

        // DHCP failures
        StateReason::DhcpStartFailed | StateReason::DhcpError | StateReason::DhcpFailed => {
            ConnectionError::DhcpFailed
        }

        // Network not found
        StateReason::SsidNotFound => ConnectionError::NotFound,

        // All other failures
        _ => ConnectionError::DeviceFailed(reason),
    }
}

impl From<u32> for DeviceType {
    fn from(value: u32) -> Self {
        match value {
            1 => DeviceType::Ethernet,
            2 => DeviceType::Wifi,
            5 => DeviceType::Bluetooth,
            30 => DeviceType::WifiP2P,
            32 => DeviceType::Loopback,
            v => DeviceType::Other(v),
        }
    }
}

impl From<u32> for DeviceState {
    fn from(value: u32) -> Self {
        match value {
            10 => DeviceState::Unmanaged,
            20 => DeviceState::Unavailable,
            30 => DeviceState::Disconnected,
            40 => DeviceState::Prepare,
            50 => DeviceState::Config,
            100 => DeviceState::Activated,
            110 => DeviceState::Deactivating,
            120 => DeviceState::Failed,
            v => DeviceState::Other(v),
        }
    }
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Ethernet => write!(f, "Ethernet"),
            DeviceType::Wifi => write!(f, "Wi-Fi"),
            DeviceType::WifiP2P => write!(f, "Wi-Fi P2P"),
            DeviceType::Loopback => write!(f, "Loopback"),
            DeviceType::Bluetooth => write!(f, "Bluetooth"),
            DeviceType::Other(v) => write!(
                f,
                "{}",
                crate::types::device_type_registry::display_name_for_code(*v)
            ),
        }
    }
}

impl Display for DeviceState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceState::Unmanaged => write!(f, "Unmanaged"),
            DeviceState::Unavailable => write!(f, "Unavailable"),
            DeviceState::Disconnected => write!(f, "Disconnected"),
            DeviceState::Prepare => write!(f, "Preparing"),
            DeviceState::Config => write!(f, "Configuring"),
            DeviceState::Activated => write!(f, "Activated"),
            DeviceState::Deactivating => write!(f, "Deactivating"),
            DeviceState::Failed => write!(f, "Failed"),
            DeviceState::Other(v) => write!(f, "Other({v})"),
        }
    }
}

impl From<u32> for BluetoothNetworkRole {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::PanU,
            1 => Self::Dun,
            _ => Self::PanU,
        }
    }
}

impl WifiSecurity {
    /// Returns `true` if this security type requires authentication.
    pub fn secured(&self) -> bool {
        !matches!(self, WifiSecurity::Open)
    }

    /// Returns `true` if this is a WPA-PSK (password-based) security type.
    pub fn is_psk(&self) -> bool {
        matches!(self, WifiSecurity::WpaPsk { .. })
    }

    /// Returns `true` if this is a WPA-EAP (Enterprise/802.1X) security type.
    pub fn is_eap(&self) -> bool {
        matches!(self, WifiSecurity::WpaEap { .. })
    }
}

impl Network {
    /// Merges another access point's information into this network.
    ///
    /// When multiple access points share the same SSID (e.g., mesh networks),
    /// this method keeps the strongest signal and combines security flags.
    /// Used internally during network scanning to deduplicate results.
    pub fn merge_ap(&mut self, other: &Network) {
        if other.strength.unwrap_or(0) > self.strength.unwrap_or(0) {
            self.strength = other.strength;
            self.frequency = other.frequency;
            self.bssid = other.bssid.clone();
        }

        self.secured |= other.secured;
        self.is_psk |= other.is_psk;
        self.is_eap |= other.is_eap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_type_from_u32_all_variants() {
        assert_eq!(DeviceType::from(1), DeviceType::Ethernet);
        assert_eq!(DeviceType::from(2), DeviceType::Wifi);
        assert_eq!(DeviceType::from(30), DeviceType::WifiP2P);
        assert_eq!(DeviceType::from(32), DeviceType::Loopback);
        assert_eq!(DeviceType::from(999), DeviceType::Other(999));
        assert_eq!(DeviceType::from(0), DeviceType::Other(0));
    }

    #[test]
    fn device_type_from_u32_registry_types() {
        assert_eq!(DeviceType::from(11), DeviceType::Other(11));
        assert_eq!(DeviceType::from(12), DeviceType::Other(12));
        assert_eq!(DeviceType::from(13), DeviceType::Other(13));
        assert_eq!(DeviceType::from(16), DeviceType::Other(16));
        assert_eq!(DeviceType::from(29), DeviceType::Other(29));
    }

    #[test]
    fn device_type_display() {
        assert_eq!(format!("{}", DeviceType::Ethernet), "Ethernet");
        assert_eq!(format!("{}", DeviceType::Wifi), "Wi-Fi");
        assert_eq!(format!("{}", DeviceType::WifiP2P), "Wi-Fi P2P");
        assert_eq!(format!("{}", DeviceType::Loopback), "Loopback");
        assert_eq!(format!("{}", DeviceType::Other(42)), "Other(42)");
    }

    #[test]
    fn device_type_display_registry() {
        assert_eq!(format!("{}", DeviceType::Other(13)), "Bridge");
        assert_eq!(format!("{}", DeviceType::Other(12)), "Bond");
        assert_eq!(format!("{}", DeviceType::Other(11)), "VLAN");
        assert_eq!(format!("{}", DeviceType::Other(16)), "TUN");
        assert_eq!(format!("{}", DeviceType::Other(29)), "WireGuard");
    }

    #[test]
    fn device_type_supports_scanning() {
        assert!(DeviceType::Wifi.supports_scanning());
        assert!(DeviceType::WifiP2P.supports_scanning());
        assert!(!DeviceType::Ethernet.supports_scanning());
        assert!(!DeviceType::Loopback.supports_scanning());
    }

    #[test]
    fn device_type_supports_scanning_registry() {
        assert!(DeviceType::Other(30).supports_scanning());
        assert!(!DeviceType::Other(13).supports_scanning());
        assert!(!DeviceType::Other(29).supports_scanning());
    }

    #[test]
    fn device_type_requires_specific_object() {
        assert!(DeviceType::Wifi.requires_specific_object());
        assert!(DeviceType::WifiP2P.requires_specific_object());
        assert!(!DeviceType::Ethernet.requires_specific_object());
        assert!(!DeviceType::Loopback.requires_specific_object());
    }

    #[test]
    fn device_type_requires_specific_object_registry() {
        assert!(DeviceType::Other(2).requires_specific_object());
        assert!(!DeviceType::Other(1).requires_specific_object());
        assert!(!DeviceType::Other(29).requires_specific_object());
    }

    #[test]
    fn device_type_has_global_enabled_state() {
        assert!(DeviceType::Wifi.has_global_enabled_state());
        assert!(!DeviceType::Ethernet.has_global_enabled_state());
        assert!(!DeviceType::WifiP2P.has_global_enabled_state());
    }

    #[test]
    fn device_type_has_global_enabled_state_registry() {
        assert!(DeviceType::Other(2).has_global_enabled_state());
        assert!(!DeviceType::Other(1).has_global_enabled_state());
    }

    #[test]
    fn device_type_connection_type_str() {
        assert_eq!(DeviceType::Ethernet.connection_type_str(), "802-3-ethernet");
        assert_eq!(DeviceType::Wifi.connection_type_str(), "802-11-wireless");
        assert_eq!(DeviceType::WifiP2P.connection_type_str(), "wifi-p2p");
        assert_eq!(DeviceType::Loopback.connection_type_str(), "loopback");
    }

    #[test]
    fn device_type_connection_type_str_registry() {
        assert_eq!(DeviceType::Other(13).connection_type_str(), "bridge");
        assert_eq!(DeviceType::Other(12).connection_type_str(), "bond");
        assert_eq!(DeviceType::Other(11).connection_type_str(), "vlan");
        assert_eq!(DeviceType::Other(29).connection_type_str(), "wireguard");
    }

    #[test]
    fn device_type_to_code() {
        assert_eq!(DeviceType::Ethernet.to_code(), 1);
        assert_eq!(DeviceType::Wifi.to_code(), 2);
        assert_eq!(DeviceType::WifiP2P.to_code(), 30);
        assert_eq!(DeviceType::Loopback.to_code(), 32);
        assert_eq!(DeviceType::Other(999).to_code(), 999);
    }

    #[test]
    fn device_type_to_code_registry() {
        assert_eq!(DeviceType::Other(11).to_code(), 11);
        assert_eq!(DeviceType::Other(12).to_code(), 12);
        assert_eq!(DeviceType::Other(13).to_code(), 13);
        assert_eq!(DeviceType::Other(16).to_code(), 16);
        assert_eq!(DeviceType::Other(29).to_code(), 29);
    }

    #[test]
    fn device_state_from_u32_all_variants() {
        assert_eq!(DeviceState::from(10), DeviceState::Unmanaged);
        assert_eq!(DeviceState::from(20), DeviceState::Unavailable);
        assert_eq!(DeviceState::from(30), DeviceState::Disconnected);
        assert_eq!(DeviceState::from(40), DeviceState::Prepare);
        assert_eq!(DeviceState::from(50), DeviceState::Config);
        assert_eq!(DeviceState::from(100), DeviceState::Activated);
        assert_eq!(DeviceState::from(110), DeviceState::Deactivating);
        assert_eq!(DeviceState::from(120), DeviceState::Failed);
        assert_eq!(DeviceState::from(7), DeviceState::Other(7));
        assert_eq!(DeviceState::from(0), DeviceState::Other(0));
    }

    #[test]
    fn device_state_display() {
        assert_eq!(format!("{}", DeviceState::Unmanaged), "Unmanaged");
        assert_eq!(format!("{}", DeviceState::Unavailable), "Unavailable");
        assert_eq!(format!("{}", DeviceState::Disconnected), "Disconnected");
        assert_eq!(format!("{}", DeviceState::Prepare), "Preparing");
        assert_eq!(format!("{}", DeviceState::Config), "Configuring");
        assert_eq!(format!("{}", DeviceState::Activated), "Activated");
        assert_eq!(format!("{}", DeviceState::Deactivating), "Deactivating");
        assert_eq!(format!("{}", DeviceState::Failed), "Failed");
        assert_eq!(format!("{}", DeviceState::Other(99)), "Other(99)");
    }

    #[test]
    fn wifi_security_open() {
        let open = WifiSecurity::Open;
        assert!(!open.secured());
        assert!(!open.is_psk());
        assert!(!open.is_eap());
    }

    #[test]
    fn wifi_security_psk() {
        let psk = WifiSecurity::WpaPsk {
            psk: "password123".into(),
        };
        assert!(psk.secured());
        assert!(psk.is_psk());
        assert!(!psk.is_eap());
    }

    #[test]
    fn wifi_security_eap() {
        let eap = WifiSecurity::WpaEap {
            opts: EapOptions {
                identity: "user@example.com".into(),
                password: "secret".into(),
                anonymous_identity: None,
                domain_suffix_match: None,
                ca_cert_path: None,
                system_ca_certs: false,
                method: EapMethod::Peap,
                phase2: Phase2::Mschapv2,
            },
        };
        assert!(eap.secured());
        assert!(!eap.is_psk());
        assert!(eap.is_eap());
    }

    #[test]
    fn state_reason_from_u32_known_codes() {
        assert_eq!(StateReason::from(0), StateReason::Unknown);
        assert_eq!(StateReason::from(1), StateReason::None);
        assert_eq!(StateReason::from(7), StateReason::SupplicantDisconnected);
        assert_eq!(StateReason::from(8), StateReason::SupplicantConfigFailed);
        assert_eq!(StateReason::from(9), StateReason::SupplicantFailed);
        assert_eq!(StateReason::from(10), StateReason::SupplicantTimeout);
        assert_eq!(StateReason::from(16), StateReason::DhcpError);
        assert_eq!(StateReason::from(17), StateReason::DhcpFailed);
        assert_eq!(StateReason::from(70), StateReason::SsidNotFound);
        assert_eq!(StateReason::from(76), StateReason::SimPinIncorrect);
    }

    #[test]
    fn state_reason_from_u32_unknown_code() {
        assert_eq!(StateReason::from(999), StateReason::Other(999));
        assert_eq!(StateReason::from(255), StateReason::Other(255));
    }

    #[test]
    fn state_reason_display() {
        assert_eq!(format!("{}", StateReason::Unknown), "unknown");
        assert_eq!(
            format!("{}", StateReason::SupplicantFailed),
            "supplicant failed"
        );
        assert_eq!(format!("{}", StateReason::DhcpFailed), "DHCP failed");
        assert_eq!(format!("{}", StateReason::SsidNotFound), "SSID not found");
        assert_eq!(
            format!("{}", StateReason::Other(123)),
            "unknown reason (123)"
        );
    }

    #[test]
    fn reason_to_error_auth_failures() {
        // Supplicant failures indicate auth issues
        assert!(matches!(reason_to_error(9), ConnectionError::AuthFailed));
        assert!(matches!(reason_to_error(7), ConnectionError::AuthFailed));
        assert!(matches!(reason_to_error(76), ConnectionError::AuthFailed));
        assert!(matches!(reason_to_error(51), ConnectionError::AuthFailed));
    }

    #[test]
    fn reason_to_error_supplicant_config() {
        assert!(matches!(
            reason_to_error(8),
            ConnectionError::SupplicantConfigFailed
        ));
    }

    #[test]
    fn reason_to_error_supplicant_timeout() {
        assert!(matches!(
            reason_to_error(10),
            ConnectionError::SupplicantTimeout
        ));
    }

    #[test]
    fn reason_to_error_dhcp_failures() {
        assert!(matches!(reason_to_error(15), ConnectionError::DhcpFailed));
        assert!(matches!(reason_to_error(16), ConnectionError::DhcpFailed));
        assert!(matches!(reason_to_error(17), ConnectionError::DhcpFailed));
    }

    #[test]
    fn reason_to_error_network_not_found() {
        assert!(matches!(reason_to_error(70), ConnectionError::NotFound));
    }

    #[test]
    fn reason_to_error_generic_failure() {
        // User disconnected maps to generic Failed
        match reason_to_error(2) {
            ConnectionError::DeviceFailed(reason) => {
                assert_eq!(reason, StateReason::UserDisconnected);
            }
            _ => panic!("expected ConnectionError::Failed"),
        }
    }

    #[test]
    fn connection_error_display() {
        assert_eq!(
            format!("{}", ConnectionError::NotFound),
            "network not found"
        );
        assert_eq!(
            format!("{}", ConnectionError::AuthFailed),
            "authentication failed"
        );
        assert_eq!(format!("{}", ConnectionError::DhcpFailed), "DHCP failed");
        assert_eq!(
            format!("{}", ConnectionError::Timeout),
            "connection timeout"
        );
        assert_eq!(
            format!("{}", ConnectionError::NoWifiDevice),
            "no Wi-Fi device found"
        );
        assert_eq!(
            format!("{}", ConnectionError::Stuck("config".into())),
            "connection stuck in state: config"
        );
        assert_eq!(
            format!(
                "{}",
                ConnectionError::DeviceFailed(StateReason::CarrierChanged)
            ),
            "connection failed: carrier changed"
        );
    }

    #[test]
    fn active_connection_state_from_u32() {
        assert_eq!(
            ActiveConnectionState::from(0),
            ActiveConnectionState::Unknown
        );
        assert_eq!(
            ActiveConnectionState::from(1),
            ActiveConnectionState::Activating
        );
        assert_eq!(
            ActiveConnectionState::from(2),
            ActiveConnectionState::Activated
        );
        assert_eq!(
            ActiveConnectionState::from(3),
            ActiveConnectionState::Deactivating
        );
        assert_eq!(
            ActiveConnectionState::from(4),
            ActiveConnectionState::Deactivated
        );
        assert_eq!(
            ActiveConnectionState::from(99),
            ActiveConnectionState::Other(99)
        );
    }

    #[test]
    fn active_connection_state_display() {
        assert_eq!(format!("{}", ActiveConnectionState::Unknown), "unknown");
        assert_eq!(
            format!("{}", ActiveConnectionState::Activating),
            "activating"
        );
        assert_eq!(format!("{}", ActiveConnectionState::Activated), "activated");
        assert_eq!(
            format!("{}", ActiveConnectionState::Deactivating),
            "deactivating"
        );
        assert_eq!(
            format!("{}", ActiveConnectionState::Deactivated),
            "deactivated"
        );
        assert_eq!(
            format!("{}", ActiveConnectionState::Other(42)),
            "unknown state (42)"
        );
    }

    #[test]
    fn connection_state_reason_from_u32() {
        assert_eq!(
            ConnectionStateReason::from(0),
            ConnectionStateReason::Unknown
        );
        assert_eq!(ConnectionStateReason::from(1), ConnectionStateReason::None);
        assert_eq!(
            ConnectionStateReason::from(2),
            ConnectionStateReason::UserDisconnected
        );
        assert_eq!(
            ConnectionStateReason::from(3),
            ConnectionStateReason::DeviceDisconnected
        );
        assert_eq!(
            ConnectionStateReason::from(6),
            ConnectionStateReason::ConnectTimeout
        );
        assert_eq!(
            ConnectionStateReason::from(9),
            ConnectionStateReason::NoSecrets
        );
        assert_eq!(
            ConnectionStateReason::from(10),
            ConnectionStateReason::LoginFailed
        );
        assert_eq!(
            ConnectionStateReason::from(99),
            ConnectionStateReason::Other(99)
        );
    }

    #[test]
    fn connection_state_reason_display() {
        assert_eq!(format!("{}", ConnectionStateReason::Unknown), "unknown");
        assert_eq!(
            format!("{}", ConnectionStateReason::NoSecrets),
            "no secrets (password) provided"
        );
        assert_eq!(
            format!("{}", ConnectionStateReason::LoginFailed),
            "login/authentication failed"
        );
        assert_eq!(
            format!("{}", ConnectionStateReason::ConnectTimeout),
            "connection timed out"
        );
        assert_eq!(
            format!("{}", ConnectionStateReason::Other(123)),
            "unknown reason (123)"
        );
    }

    #[test]
    fn connection_state_reason_to_error_auth_failures() {
        // NoSecrets and LoginFailed map to AuthFailed
        assert!(matches!(
            connection_state_reason_to_error(9),
            ConnectionError::AuthFailed
        ));
        assert!(matches!(
            connection_state_reason_to_error(10),
            ConnectionError::AuthFailed
        ));
    }

    #[test]
    fn connection_state_reason_to_error_timeout() {
        // ConnectTimeout and ServiceStartTimeout map to Timeout
        assert!(matches!(
            connection_state_reason_to_error(6),
            ConnectionError::Timeout
        ));
        assert!(matches!(
            connection_state_reason_to_error(7),
            ConnectionError::Timeout
        ));
    }

    #[test]
    fn connection_state_reason_to_error_dhcp() {
        // IpConfigInvalid maps to DhcpFailed
        assert!(matches!(
            connection_state_reason_to_error(5),
            ConnectionError::DhcpFailed
        ));
    }

    #[test]
    fn connection_state_reason_to_error_generic() {
        // Other reasons map to ConnectionFailed
        match connection_state_reason_to_error(2) {
            ConnectionError::ActivationFailed(reason) => {
                assert_eq!(reason, ConnectionStateReason::UserDisconnected);
            }
            _ => panic!("expected ConnectionError::ConnectionFailed"),
        }
    }

    #[test]
    fn connection_failed_error_display() {
        assert_eq!(
            format!(
                "{}",
                ConnectionError::ActivationFailed(ConnectionStateReason::NoSecrets)
            ),
            "connection activation failed: no secrets (password) provided"
        );
    }

    #[test]
    fn test_bluetooth_network_role_from_u32() {
        assert_eq!(BluetoothNetworkRole::from(0), BluetoothNetworkRole::PanU);
        assert_eq!(BluetoothNetworkRole::from(1), BluetoothNetworkRole::Dun);
        // Unknown values default to PanU
        assert_eq!(BluetoothNetworkRole::from(999), BluetoothNetworkRole::PanU);
    }

    #[test]
    fn test_bluetooth_network_role_display() {
        assert_eq!(format!("{}", BluetoothNetworkRole::PanU), "PANU");
        assert_eq!(format!("{}", BluetoothNetworkRole::Dun), "DUN");
    }

    #[test]
    fn test_bluetooth_identity_creation() {
        let identity =
            BluetoothIdentity::new("00:1A:7D:DA:71:13".into(), BluetoothNetworkRole::PanU).unwrap();

        assert_eq!(identity.bdaddr, "00:1A:7D:DA:71:13");
        assert!(matches!(
            identity.bt_device_type,
            BluetoothNetworkRole::PanU
        ));
    }

    #[test]
    fn test_bluetooth_identity_dun() {
        let identity =
            BluetoothIdentity::new("C8:1F:E8:F0:51:57".into(), BluetoothNetworkRole::Dun).unwrap();

        assert_eq!(identity.bdaddr, "C8:1F:E8:F0:51:57");
        assert!(matches!(identity.bt_device_type, BluetoothNetworkRole::Dun));
    }

    #[test]
    fn test_bluetooth_identity_creation_error() {
        let res = BluetoothIdentity::new("SomeInvalidAddress".into(), BluetoothNetworkRole::Dun);
        assert!(res.is_err());
    }

    #[test]
    fn test_bluetooth_device_creation() {
        let role = BluetoothNetworkRole::PanU as u32;
        let device = BluetoothDevice::new(
            "00:1A:7D:DA:71:13".into(),
            Some("MyPhone".into()),
            Some("Phone".into()),
            role,
            DeviceState::Activated,
        );

        assert_eq!(device.bdaddr, "00:1A:7D:DA:71:13");
        assert_eq!(device.name, Some("MyPhone".into()));
        assert_eq!(device.alias, Some("Phone".into()));
        assert!(matches!(device.bt_caps, _role));
        assert_eq!(device.state, DeviceState::Activated);
    }

    #[test]
    fn test_bluetooth_device_display() {
        let role = BluetoothNetworkRole::PanU as u32;
        let device = BluetoothDevice::new(
            "00:1A:7D:DA:71:13".into(),
            Some("MyPhone".into()),
            Some("Phone".into()),
            role,
            DeviceState::Activated,
        );

        let display_str = format!("{}", device);
        assert!(display_str.contains("Phone"));
        assert!(display_str.contains("00:1A:7D:DA:71:13"));
        assert!(display_str.contains("PANU"));
    }

    #[test]
    fn test_bluetooth_device_display_no_alias() {
        let role = BluetoothNetworkRole::Dun as u32;
        let device = BluetoothDevice::new(
            "00:1A:7D:DA:71:13".into(),
            Some("MyPhone".into()),
            None,
            role,
            DeviceState::Disconnected,
        );

        let display_str = format!("{}", device);
        assert!(display_str.contains("unknown"));
        assert!(display_str.contains("00:1A:7D:DA:71:13"));
        assert!(display_str.contains("DUN"));
    }

    #[test]
    fn test_device_is_bluetooth() {
        let bt_device = Device {
            path: "/org/freedesktop/NetworkManager/Devices/1".into(),
            interface: "bt0".into(),
            identity: DeviceIdentity::new("00:1A:7D:DA:71:13".into(), "00:1A:7D:DA:71:13".into()),
            device_type: DeviceType::Bluetooth,
            state: DeviceState::Activated,
            managed: Some(true),
            driver: Some("btusb".into()),
            ip4_address: None,
            ip6_address: None,
        };

        assert!(bt_device.is_bluetooth());
        assert!(!bt_device.is_wireless());
        assert!(!bt_device.is_wired());
    }

    #[test]
    fn test_device_type_bluetooth() {
        assert_eq!(DeviceType::from(5), DeviceType::Bluetooth);
    }

    #[test]
    fn test_device_type_bluetooth_display() {
        assert_eq!(format!("{}", DeviceType::Bluetooth), "Bluetooth");
    }

    #[test]
    fn test_connection_error_no_bluetooth_device() {
        let err = ConnectionError::NoBluetoothDevice;
        assert_eq!(format!("{}", err), "Bluetooth device not found");
    }

    // Builder pattern tests

    #[test]
    fn test_vpn_credentials_builder_basic() {
        let peer = WireGuardPeer::new(
            "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
            "vpn.example.com:51820",
            vec!["0.0.0.0/0".into()],
        );

        let creds = VpnCredentials::builder()
            .name("TestVPN")
            .wireguard()
            .gateway("vpn.example.com:51820")
            .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
            .address("10.0.0.2/24")
            .add_peer(peer)
            .build();

        assert_eq!(creds.name, "TestVPN");
        assert_eq!(creds.vpn_type, VpnType::WireGuard);
        assert_eq!(creds.gateway, "vpn.example.com:51820");
        assert_eq!(
            creds.private_key,
            "YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM="
        );
        assert_eq!(creds.address, "10.0.0.2/24");
        assert_eq!(creds.peers.len(), 1);
        assert!(creds.dns.is_none());
        assert!(creds.mtu.is_none());
    }

    #[test]
    fn test_vpn_credentials_builder_with_optionals() {
        let peer = WireGuardPeer::new(
            "public_key",
            "vpn.example.com:51820",
            vec!["0.0.0.0/0".into()],
        );

        let uuid = Uuid::new_v4();
        let creds = VpnCredentials::builder()
            .name("TestVPN")
            .wireguard()
            .gateway("vpn.example.com:51820")
            .private_key("private_key")
            .address("10.0.0.2/24")
            .add_peer(peer)
            .with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()])
            .with_mtu(1420)
            .with_uuid(uuid)
            .build();

        assert_eq!(creds.dns, Some(vec!["1.1.1.1".into(), "8.8.8.8".into()]));
        assert_eq!(creds.mtu, Some(1420));
        assert_eq!(creds.uuid, Some(uuid));
    }

    #[test]
    fn test_vpn_credentials_builder_multiple_peers() {
        let peer1 =
            WireGuardPeer::new("key1", "vpn1.example.com:51820", vec!["10.0.0.0/24".into()]);
        let peer2 = WireGuardPeer::new(
            "key2",
            "vpn2.example.com:51820",
            vec!["192.168.0.0/24".into()],
        );

        let creds = VpnCredentials::builder()
            .name("MultiPeerVPN")
            .wireguard()
            .gateway("vpn.example.com:51820")
            .private_key("private_key")
            .address("10.0.0.2/24")
            .add_peer(peer1)
            .add_peer(peer2)
            .build();

        assert_eq!(creds.peers.len(), 2);
    }

    #[test]
    fn test_vpn_credentials_builder_peers_method() {
        let peers = vec![
            WireGuardPeer::new("key1", "vpn1.example.com:51820", vec!["0.0.0.0/0".into()]),
            WireGuardPeer::new("key2", "vpn2.example.com:51820", vec!["0.0.0.0/0".into()]),
        ];

        let creds = VpnCredentials::builder()
            .name("TestVPN")
            .wireguard()
            .gateway("vpn.example.com:51820")
            .private_key("private_key")
            .address("10.0.0.2/24")
            .peers(peers)
            .build();

        assert_eq!(creds.peers.len(), 2);
    }

    #[test]
    #[should_panic(expected = "name is required")]
    fn test_vpn_credentials_builder_missing_name() {
        let peer = WireGuardPeer::new("key", "vpn.example.com:51820", vec!["0.0.0.0/0".into()]);

        VpnCredentials::builder()
            .wireguard()
            .gateway("vpn.example.com:51820")
            .private_key("private_key")
            .address("10.0.0.2/24")
            .add_peer(peer)
            .build();
    }

    #[test]
    #[should_panic(expected = "vpn_type is required")]
    fn test_vpn_credentials_builder_missing_vpn_type() {
        let peer = WireGuardPeer::new("key", "vpn.example.com:51820", vec!["0.0.0.0/0".into()]);

        VpnCredentials::builder()
            .name("TestVPN")
            .gateway("vpn.example.com:51820")
            .private_key("private_key")
            .address("10.0.0.2/24")
            .add_peer(peer)
            .build();
    }

    #[test]
    #[should_panic(expected = "at least one peer is required")]
    fn test_vpn_credentials_builder_missing_peers() {
        VpnCredentials::builder()
            .name("TestVPN")
            .wireguard()
            .gateway("vpn.example.com:51820")
            .private_key("private_key")
            .address("10.0.0.2/24")
            .build();
    }

    #[test]
    fn test_eap_options_builder_basic() {
        let opts = EapOptions::builder()
            .identity("user@example.com")
            .password("password")
            .method(EapMethod::Peap)
            .phase2(Phase2::Mschapv2)
            .build();

        assert_eq!(opts.identity, "user@example.com");
        assert_eq!(opts.password, "password");
        assert_eq!(opts.method, EapMethod::Peap);
        assert_eq!(opts.phase2, Phase2::Mschapv2);
        assert!(opts.anonymous_identity.is_none());
        assert!(opts.domain_suffix_match.is_none());
        assert!(opts.ca_cert_path.is_none());
        assert!(!opts.system_ca_certs);
    }

    #[test]
    fn test_eap_options_builder_with_optionals() {
        let opts = EapOptions::builder()
            .identity("user@company.com")
            .password("password")
            .method(EapMethod::Ttls)
            .phase2(Phase2::Pap)
            .anonymous_identity("anonymous@company.com")
            .domain_suffix_match("company.com")
            .ca_cert_path("file:///etc/ssl/certs/ca.pem")
            .system_ca_certs(true)
            .build();

        assert_eq!(opts.identity, "user@company.com");
        assert_eq!(opts.password, "password");
        assert_eq!(opts.method, EapMethod::Ttls);
        assert_eq!(opts.phase2, Phase2::Pap);
        assert_eq!(
            opts.anonymous_identity,
            Some("anonymous@company.com".into())
        );
        assert_eq!(opts.domain_suffix_match, Some("company.com".into()));
        assert_eq!(
            opts.ca_cert_path,
            Some("file:///etc/ssl/certs/ca.pem".into())
        );
        assert!(opts.system_ca_certs);
    }

    #[test]
    fn test_eap_options_builder_peap_mschapv2() {
        let opts = EapOptions::builder()
            .identity("employee@corp.com")
            .password("secret")
            .method(EapMethod::Peap)
            .phase2(Phase2::Mschapv2)
            .system_ca_certs(true)
            .build();

        assert_eq!(opts.method, EapMethod::Peap);
        assert_eq!(opts.phase2, Phase2::Mschapv2);
        assert!(opts.system_ca_certs);
    }

    #[test]
    fn test_eap_options_builder_ttls_pap() {
        let opts = EapOptions::builder()
            .identity("student@university.edu")
            .password("password")
            .method(EapMethod::Ttls)
            .phase2(Phase2::Pap)
            .ca_cert_path("file:///etc/ssl/certs/university.pem")
            .build();

        assert_eq!(opts.method, EapMethod::Ttls);
        assert_eq!(opts.phase2, Phase2::Pap);
        assert_eq!(
            opts.ca_cert_path,
            Some("file:///etc/ssl/certs/university.pem".into())
        );
    }

    #[test]
    #[should_panic(expected = "identity is required")]
    fn test_eap_options_builder_missing_identity() {
        EapOptions::builder()
            .password("password")
            .method(EapMethod::Peap)
            .phase2(Phase2::Mschapv2)
            .build();
    }

    #[test]
    #[should_panic(expected = "password is required")]
    fn test_eap_options_builder_missing_password() {
        EapOptions::builder()
            .identity("user@example.com")
            .method(EapMethod::Peap)
            .phase2(Phase2::Mschapv2)
            .build();
    }

    #[test]
    #[should_panic(expected = "method is required")]
    fn test_eap_options_builder_missing_method() {
        EapOptions::builder()
            .identity("user@example.com")
            .password("password")
            .phase2(Phase2::Mschapv2)
            .build();
    }

    #[test]
    #[should_panic(expected = "phase2 is required")]
    fn test_eap_options_builder_missing_phase2() {
        EapOptions::builder()
            .identity("user@example.com")
            .password("password")
            .method(EapMethod::Peap)
            .build();
    }

    #[test]
    fn test_vpn_credentials_builder_equivalence_to_new() {
        let peer = WireGuardPeer::new(
            "public_key",
            "vpn.example.com:51820",
            vec!["0.0.0.0/0".into()],
        );

        let creds_new = VpnCredentials::new(
            VpnType::WireGuard,
            "TestVPN",
            "vpn.example.com:51820",
            "private_key",
            "10.0.0.2/24",
            vec![peer.clone()],
        );

        let creds_builder = VpnCredentials::builder()
            .name("TestVPN")
            .wireguard()
            .gateway("vpn.example.com:51820")
            .private_key("private_key")
            .address("10.0.0.2/24")
            .add_peer(peer)
            .build();

        assert_eq!(creds_new.name, creds_builder.name);
        assert_eq!(creds_new.vpn_type, creds_builder.vpn_type);
        assert_eq!(creds_new.gateway, creds_builder.gateway);
        assert_eq!(creds_new.private_key, creds_builder.private_key);
        assert_eq!(creds_new.address, creds_builder.address);
        assert_eq!(creds_new.peers.len(), creds_builder.peers.len());
    }

    #[test]
    fn test_eap_options_builder_equivalence_to_new() {
        let opts_new = EapOptions::new("user@example.com", "password")
            .with_method(EapMethod::Peap)
            .with_phase2(Phase2::Mschapv2);

        let opts_builder = EapOptions::builder()
            .identity("user@example.com")
            .password("password")
            .method(EapMethod::Peap)
            .phase2(Phase2::Mschapv2)
            .build();

        assert_eq!(opts_new.identity, opts_builder.identity);
        assert_eq!(opts_new.password, opts_builder.password);
        assert_eq!(opts_new.method, opts_builder.method);
        assert_eq!(opts_new.phase2, opts_builder.phase2);
    }

    // Timeout configuration tests

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connection_timeout, Duration::from_secs(30));
        assert_eq!(config.disconnect_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_timeout_config_new() {
        let config = TimeoutConfig::new();
        assert_eq!(config.connection_timeout, Duration::from_secs(30));
        assert_eq!(config.disconnect_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_timeout_config_with_connection_timeout() {
        let config = TimeoutConfig::new().with_connection_timeout(Duration::from_secs(60));
        assert_eq!(config.connection_timeout, Duration::from_secs(60));
        assert_eq!(config.disconnect_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_timeout_config_with_disconnect_timeout() {
        let config = TimeoutConfig::new().with_disconnect_timeout(Duration::from_secs(20));
        assert_eq!(config.connection_timeout, Duration::from_secs(30));
        assert_eq!(config.disconnect_timeout, Duration::from_secs(20));
    }

    #[test]
    fn test_timeout_config_with_both_timeouts() {
        let config = TimeoutConfig::new()
            .with_connection_timeout(Duration::from_secs(90))
            .with_disconnect_timeout(Duration::from_secs(30));
        assert_eq!(config.connection_timeout, Duration::from_secs(90));
        assert_eq!(config.disconnect_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_config_chaining() {
        let config = TimeoutConfig::default()
            .with_connection_timeout(Duration::from_secs(45))
            .with_disconnect_timeout(Duration::from_secs(15))
            .with_connection_timeout(Duration::from_secs(60)); // Override previous value

        assert_eq!(config.connection_timeout, Duration::from_secs(60));
        assert_eq!(config.disconnect_timeout, Duration::from_secs(15));
    }

    #[test]
    fn test_timeout_config_copy() {
        let config1 = TimeoutConfig::new().with_connection_timeout(Duration::from_secs(120));
        let config2 = config1; // Should copy, not move

        assert_eq!(config1.connection_timeout, Duration::from_secs(120));
        assert_eq!(config2.connection_timeout, Duration::from_secs(120));
    }
}
