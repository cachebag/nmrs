#![allow(deprecated)]

use std::sync::Arc;

use uuid::Uuid;

use super::device::DeviceState;

/// VPN connection type.
///
/// Identifies the VPN protocol/technology used for the connection.
/// Currently only WireGuard is supported.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpnType {
    /// WireGuard - modern, high-performance VPN protocol.
    WireGuard,
    OpenVpn,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum OpenVpnAuthType {
    Password,
    Tls,
    PasswordTls,
    StaticKey,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct OpenVpnCredentials {
    pub username: String,
    pub password: String,
}
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct OpenVpnCerts {
    pub ca_cert: String,
    pub client_cert: String,
    pub client_key: String,
    pub key_password: String,
}
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct OpenVpnCommon {
    pub auth: String,
    pub cipher: String,
    pub dns: Option<Vec<String>>,
    pub mtu: Option<u32>,
    pub uuid: Option<Uuid>,
}
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct OpenVpnConfig {
    pub name: String,
    pub remote: String,
    pub port: u16,
    pub tcp: bool,
    pub credentials: OpenVpnCredentials,
    pub certs: OpenVpnCerts,
    pub auth_type: OpenVpnAuthType,
    pub common: OpenVpnCommon,
}

impl OpenVpnConfig {
    /// Creates new `OpenVpnConfig` with required fields
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{OpenVpnConfig, OpenVpnCredentials, OpenVpnCerts, OpenVpnCommon};
    ///
    /// let config = OpenVpnConfig::new(
    ///     "MyVpn",
    ///     "vpn.example.com",
    ///     51820,
    ///     True,
    ///     new_credentials,
    ///     new_certs,
    ///     new_auth_type,
    ///     common_fields
    ///     );
    ///
    ///
    ///
    ///
    ///
    /// ```

    pub fn new(
        name: impl Into<String>,
        remote: impl Into<String>,
        port: impl Into<u16>,
        tcp: impl Into<bool>,
        credentials: impl Into<OpenVpnCredentials>,
        certs: impl Into<OpenVpnCerts>,
        auth_type: impl Into<OpenVpnAuthType>,
        common: impl Into<OpenVpnCommon>,
    ) -> Self {
        Self {
            name: name.into(),
            remote: remote.into(),
            port: port.into(),
            tcp: tcp.into(),
            credentials: credentials.into(),
            certs: certs.into(),
            auth_type: auth_type.into(),
            common: common.into(),
        }
    }
        /// Sets the DNS servers to use when connected.
    #[must_use]
    pub fn with_dns(mut self, dns: Vec<String>) -> Self {
        self.common.dns = Some(dns);
        self
    }

    /// Sets the MTU (Maximum Transmission Unit) size.
    #[must_use]
    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.common.mtu = Some(mtu);
        self
    }

    /// Sets the UUID for the connection.
    #[must_use]
    pub fn with_uuid(mut self, uuid: Uuid) -> Self {
        self.common.uuid = Some(uuid);
        self
    }
}

impl VpnConfig for OpenVpnConfig{
    fn vpn_type(&self) -> VpnType {
        VpnType::OpenVpn
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn gateway(&self) -> &str {
        &self.remote
    }

    fn dns(&self) -> Option<&[String]> {
        self.common.dns.as_deref()
    }

    fn mtu(&self) -> Option<u32> {
        self.common.mtu
    }

    fn uuid(&self) -> Option<Uuid> {
        self.common.uuid
    }
}

//how to satisfy the common vpn config with the one above


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

/// WireGuard configuration for establishing a VPN connection.
///
/// Stores the necessary information to configure and connect to a VPN.
///
/// # Fields
///
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
/// use nmrs::{WireGuardConfig, WireGuardPeer};
///
/// let peer = WireGuardPeer::new(
///     "server_public_key",
///     "vpn.home.com:51820",
///     vec!["0.0.0.0/0".into()],
/// ).with_persistent_keepalive(25);
///
/// let config = WireGuardConfig::new(
///     "HomeVPN",
///     "vpn.home.com:51820",
///     "aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789=",
///     "10.0.0.2/24",
///     vec![peer],
/// ).with_dns(vec!["1.1.1.1".into()]);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct WireGuardConfig {
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

impl WireGuardConfig {
    /// Creates new `WireGuardConfig` with the required fields.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nmrs::{WireGuardConfig, WireGuardPeer};
    ///
    /// let peer = WireGuardPeer::new(
    ///     "server_public_key",
    ///     "vpn.example.com:51820",
    ///     vec!["0.0.0.0/0".into()],
    /// );
    ///
    /// let config = WireGuardConfig::new(
    ///     "MyVPN",
    ///     "vpn.example.com:51820",
    ///     "client_private_key",
    ///     "10.0.0.2/24",
    ///     vec![peer],
    /// );
    /// ```
    pub fn new(
        name: impl Into<String>,
        gateway: impl Into<String>,
        private_key: impl Into<String>,
        address: impl Into<String>,
        peers: Vec<WireGuardPeer>,
    ) -> Self {
        Self {
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

    /// Sets the DNS servers to use when connected.
    #[must_use]
    pub fn with_dns(mut self, dns: Vec<String>) -> Self {
        self.dns = Some(dns);
        self
    }

    /// Sets the MTU (Maximum Transmission Unit) size.
    #[must_use]
    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Sets the UUID for the connection.
    #[must_use]
    pub fn with_uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }
}

impl VpnConfig for WireGuardConfig {
    fn vpn_type(&self) -> VpnType {
        VpnType::WireGuard
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn dns(&self) -> Option<&[String]> {
        self.dns.as_deref()
    }

    fn mtu(&self) -> Option<u32> {
        self.mtu
    }

    fn uuid(&self) -> Option<Uuid> {
        self.uuid
    }
}

impl From<WireGuardConfig> for VpnCredentials {
    fn from(config: WireGuardConfig) -> Self {
        Self {
            vpn_type: VpnType::WireGuard,
            name: config.name,
            gateway: config.gateway,
            private_key: config.private_key,
            address: config.address,
            peers: config.peers,
            dns: config.dns,
            mtu: config.mtu,
            uuid: config.uuid,
        }
    }
}

impl From<VpnCredentials> for WireGuardConfig {
    fn from(config: VpnCredentials) -> Self {
        Self {
            name: config.name,
            gateway: config.gateway,
            private_key: config.private_key,
            address: config.address,
            peers: config.peers,
            dns: config.dns,
            mtu: config.mtu,
            uuid: config.uuid,
        }
    }
}

/// Legacy VPN credentials for establishing a VPN connection.
///
/// Prefer [`WireGuardConfig`] for new WireGuard connections.
#[deprecated(note = "Use WireGuardConfig instead.")]
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
    /// Prefer [`WireGuardConfig::new`] for new code.
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
    #[must_use]
    pub fn builder() -> VpnCredentialsBuilder {
        VpnCredentialsBuilder::default()
    }

    /// Sets the DNS servers to use when connected.
    #[must_use]
    pub fn with_dns(mut self, dns: Vec<String>) -> Self {
        self.dns = Some(dns);
        self
    }

    /// Sets the MTU (Maximum Transmission Unit) size.
    #[must_use]
    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Sets the UUID for the connection.
    #[must_use]
    pub fn with_uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }
}

impl VpnConfig for VpnCredentials {
    fn vpn_type(&self) -> VpnType {
        self.vpn_type
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn dns(&self) -> Option<&[String]> {
        self.dns.as_deref()
    }

    fn mtu(&self) -> Option<u32> {
        self.mtu
    }

    fn uuid(&self) -> Option<Uuid> {
        self.uuid
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
    #[must_use]
    pub fn wireguard(mut self) -> Self {
        self.vpn_type = Some(VpnType::WireGuard);
        self
    }

    /// Sets the VPN type.
    ///
    /// For most use cases, prefer using [`wireguard()`](Self::wireguard) instead.
    #[must_use]
    pub fn vpn_type(mut self, vpn_type: VpnType) -> Self {
        self.vpn_type = Some(vpn_type);
        self
    }

    /// Sets the connection name.
    ///
    /// This is the unique identifier for the VPN connection profile.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the VPN gateway endpoint.
    ///
    /// Should be in "host:port" format (e.g., "vpn.example.com:51820").
    #[must_use]
    pub fn gateway(mut self, gateway: impl Into<String>) -> Self {
        self.gateway = Some(gateway.into());
        self
    }

    /// Sets the client's WireGuard private key.
    ///
    /// The private key should be base64 encoded.
    #[must_use]
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
    #[must_use]
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Adds a WireGuard peer to the connection.
    ///
    /// Multiple peers can be added by calling this method multiple times.
    #[must_use]
    pub fn add_peer(mut self, peer: WireGuardPeer) -> Self {
        self.peers.push(peer);
        self
    }

    /// Sets all WireGuard peers at once.
    ///
    /// This replaces any previously added peers.
    #[must_use]
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
    #[must_use]
    pub fn with_dns(mut self, dns: Vec<String>) -> Self {
        self.dns = Some(dns);
        self
    }

    /// Sets the MTU (Maximum Transmission Unit) size.
    ///
    /// Typical values are 1420 for WireGuard over standard networks.
    #[must_use]
    pub fn with_mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Sets a specific UUID for the connection.
    ///
    /// If not set, NetworkManager will generate one automatically.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn with_preshared_key(mut self, psk: impl Into<String>) -> Self {
        self.preshared_key = Some(psk.into());
        self
    }

    /// Sets the persistent keepalive interval in seconds.
    #[must_use]
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
