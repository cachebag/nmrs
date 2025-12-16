use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use thiserror::Error;
use uuid::Uuid;

/// NetworkManager active connection state.
///
/// These values represent the lifecycle states of an active connection
/// as reported by the NM D-Bus API.
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
///         if let Some(speed) == device.speed {
///             println!("  Link speed: {speed} Mb/s");
///         }
///     }
///     
///     if let Some(driver) == &device.driver {
///         println!("  Driver: {}", driver);
///     }
/// }
/// # Ok(())
/// # }
/// ```
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
    /// Link speed in Mb/s (wired devices)
    pub speed: Option<u32>,
}

/// Represents the hardware identity of a network device.
///
/// Contains MAC addresses that uniquely identify the device. The permanent
/// MAC is burned into the hardware, while the current MAC may be different
/// if MAC address randomization or spoofing is enabled.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceIdentity {
    /// The permanent (factory-assigned) MAC address.
    pub permanent_mac: String,
    /// The current MAC address in use (may differ if randomized/spoofed).
    pub current_mac: String,
}

/// EAP (Extensible Authentication Protocol) method for WPA-Enterprise Wi-Fi.
///
/// These are the outer authentication methods used in 802.1X authentication.
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
/// let opts = EapOptions {
///     identity: "employee@company.com".into(),
///     password: "my_password".into(),
///     anonymous_identity: Some("anonymous@company.com".into()),
///     domain_suffix_match: Some("company.com".into()),
///     ca_cert_path: None,
///     system_ca_certs: true,  // Use system certificate store
///     method: EapMethod::Peap,
///     phase2: Phase2::Mschapv2,
/// };
/// ```
///
/// ## TTLS with PAP (Alternative Setup)
///
/// ```rust
/// use nmrs::{EapOptions, EapMethod, Phase2};
///
/// let opts = EapOptions {
///     identity: "student@university.edu".into(),
///     password: "password".into(),
///     anonymous_identity: None,
///     domain_suffix_match: None,
///     ca_cert_path: Some("file:///etc/ssl/certs/university-ca.pem".into()),
///     system_ca_certs: false,
///     method: EapMethod::Ttls,
///     phase2: Phase2::Pap,
/// };
/// ```
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
/// let opts_priority = ConnectionOptions {
///     autoconnect: true,
///     autoconnect_priority: Some(10),  // Higher = more preferred
///     autoconnect_retries: Some(3),    // Retry up to 3 times
/// };
///
/// // Manual connection only
/// let opts_manual = ConnectionOptions {
///     autoconnect: false,
///     autoconnect_priority: None,
///     autoconnect_retries: None,
/// };
/// ```
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
/// nm.connect("CorpWiFi", WifiSecurity::WpaEap {
///     opts: EapOptions {
///         identity: "user@company.com".into(),
///         password: "password".into(),
///         anonymous_identity: None,
///         domain_suffix_match: Some("company.com".into()),
///         ca_cert_path: None,
///         system_ca_certs: true,
///         method: EapMethod::Peap,
///         phase2: Phase2::Mschapv2,
///     }
/// }).await?;
/// # Ok(())
/// # }
/// ```
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
/// let creds = VpnCredentials {
///     vpn_type: VpnType::WireGuard,
///     name: "HomeVPN".into(),
///     gateway: "vpn.home.com:51820".into(),
///     private_key: "aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789=".into(),
///     address: "10.0.0.2/24".into(),
///     peers: vec![WireGuardPeer {
///         public_key: "server_public_key".into(),
///         gateway: "vpn.home.com:51820".into(),
///         allowed_ips: vec!["0.0.0.0/0".into()],
///         preshared_key: None,
///         persistent_keepalive: Some(25),
///     }],
///     dns: Some(vec!["1.1.1.1".into()]),
///     mtu: None,
///     uuid: None,
/// };
/// ```
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
/// let peer = WireGuardPeer {
///     public_key: "aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789=".into(),
///     gateway: "vpn.example.com:51820".into(),
///     allowed_ips: vec!["0.0.0.0/0".into(), "::/0".into()],
///     preshared_key: None,
///     persistent_keepalive: Some(25),
/// };
/// ```
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
/// ```rust
/// use nmrs::{VpnConnection, VpnType, DeviceState};
///
/// let vpn = VpnConnection {
///     name: "WorkVPN".into(),
///     vpn_type: VpnType::WireGuard,
///     state: DeviceState::Activated,
///     interface: Some("wg0".into()),
/// };
/// ```
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
/// # Limitations
///
/// - `ip6_address`: IPv6 address parsing is not currently implemented and will
///   always return `None`. IPv4 addresses are fully supported.
///
/// # Example
///
/// ```rust
/// use nmrs::{VpnConnectionInfo, VpnType, DeviceState};
///
/// let info = VpnConnectionInfo {
///     name: "WorkVPN".into(),
///     vpn_type: VpnType::WireGuard,
///     state: DeviceState::Activated,
///     interface: Some("wg0".into()),
///     gateway: Some("vpn.example.com:51820".into()),
///     ip4_address: Some("10.0.0.2/24".into()),
///     ip6_address: None,  // IPv6 not yet implemented
///     dns_servers: vec!["1.1.1.1".into()],
/// };
/// ```
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
    /// IPv6 address (currently always `None` - IPv6 parsing not yet implemented).
    pub ip6_address: Option<String>,
    /// DNS servers configured for this VPN.
    pub dns_servers: Vec<String>,
}

/// NetworkManager device types.
///
/// Represents the type of network hardware managed by NetworkManager.
/// This enum uses a registry-based system to support adding new device
/// types without breaking the API.
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
            DeviceType::Other(v) => write!(f, "Other({v})"),
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
}
