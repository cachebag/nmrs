use super::device::DeviceState;
use super::openvpn::OpenVpnConfig;
use super::wireguard::WireGuardConfig;
use uuid::Uuid;

/// VPN connection type.
///
/// Identifies the VPN protocol/technology used for the connection.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnType {
    /// WireGuard - modern, high-performance VPN protocol.
    WireGuard,
    /// OpenVPN - widely-used open-source VPN protocol.
    OpenVpn,
}

/// VPN connection configuration
///
/// Type-safe wrapper for VPN configurations that enables protocol dispatch.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum VpnConfiguration {
    /// WireGuard VPN configuration.
    WireGuard(WireGuardConfig),
    /// OpenVPN configuration
    OpenVpn(Box<OpenVpnConfig>),
}

impl From<WireGuardConfig> for VpnConfiguration {
    fn from(config: WireGuardConfig) -> Self {
        Self::WireGuard(config)
    }
}

impl From<OpenVpnConfig> for VpnConfiguration {
    fn from(config: OpenVpnConfig) -> Self {
        Self::OpenVpn(Box::new(config))
    }
}

impl VpnConfig for VpnConfiguration {
    fn vpn_type(&self) -> VpnType {
        match self {
            Self::WireGuard(_) => VpnType::WireGuard,
            Self::OpenVpn(_) => VpnType::OpenVpn,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::WireGuard(c) => &c.name,
            Self::OpenVpn(c) => &c.name,
        }
    }

    fn dns(&self) -> Option<&[String]> {
        match self {
            Self::WireGuard(c) => c.dns.as_deref(),
            Self::OpenVpn(c) => c.dns.as_deref(),
        }
    }

    fn mtu(&self) -> Option<u32> {
        match self {
            Self::WireGuard(c) => c.mtu,
            Self::OpenVpn(c) => c.mtu,
        }
    }

    fn uuid(&self) -> Option<Uuid> {
        match self {
            Self::WireGuard(c) => c.uuid,
            Self::OpenVpn(c) => c.uuid,
        }
    }
}

/// Common metadata shared by VPN connection configurations.
pub trait VpnConfig: Send + Sync + std::fmt::Debug {
    /// Returns the VPN protocol used by this configuration.
    fn vpn_type(&self) -> VpnType;

    /// Returns the connection name.
    fn name(&self) -> &str;

    /// Returns the configured DNS servers, if any.
    fn dns(&self) -> Option<&[String]>;

    /// Returns the configured MTU, if any.
    fn mtu(&self) -> Option<u32>;

    /// Returns the configured UUID, if any.
    fn uuid(&self) -> Option<Uuid>;
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

/// Protocol-specific details for an active VPN connection.
///
/// Provides configuration details extracted from the NetworkManager connection
/// profile, varying by VPN type.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum VpnDetails {
    /// WireGuard-specific connection details.
    WireGuard {
        /// The local interface's public key.
        public_key: Option<String>,
        /// The peer endpoint (e.g. "vpn.example.com:51820").
        endpoint: Option<String>,
    },
    /// OpenVPN-specific connection details.
    OpenVpn {
        /// Remote server address (e.g. "vpn.example.com:1194").
        remote: String,
        /// Remote server port.
        port: u16,
        /// Transport protocol ("udp" or "tcp").
        protocol: String,
        /// Data channel cipher (e.g. "AES-256-GCM").
        cipher: Option<String>,
        /// HMAC digest algorithm (e.g. "SHA256").
        auth: Option<String>,
        /// Compression mode if enabled (e.g. "lz4-v2").
        compression: Option<String>,
    },
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
    /// Protocol-specific connection details, if available.
    pub details: Option<VpnDetails>,
}
