//! WireGuard VPN connection builder with validation.
//!
//! Provides a type-safe builder API for constructing WireGuard VPN connections
//! with comprehensive validation of keys, addresses, and peer configurations.

use std::collections::HashMap;
use std::net::Ipv4Addr;
use uuid::Uuid;
use zvariant::Value;

use super::connection_builder::{ConnectionBuilder, IpConfig};
use crate::api::models::{ConnectionError, ConnectionOptions, WireGuardPeer};

/// Builder for WireGuard VPN connections.
///
/// This builder provides a fluent API for creating WireGuard VPN connection settings
/// with validation at build time.
///
/// # Example
///
/// ```rust
/// use nmrs::builders::WireGuardBuilder;
/// use nmrs::{WireGuardPeer, ConnectionOptions};
///
/// let peer = WireGuardPeer::new(
///     "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=",
///     "vpn.example.com:51820",
///     vec!["0.0.0.0/0".into()],
/// ).with_persistent_keepalive(25);
///
/// let settings = WireGuardBuilder::new("MyVPN")
///     .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
///     .address("10.0.0.2/24")
///     .add_peer(peer)
///     .autoconnect(false)
///     .build()
///     .expect("Failed to build WireGuard connection");
/// ```
pub struct WireGuardBuilder {
    inner: ConnectionBuilder,
    name: String,
    private_key: Option<String>,
    address: Option<String>,
    peers: Vec<WireGuardPeer>,
    dns: Option<Vec<String>>,
    mtu: Option<u32>,
    uuid: Option<Uuid>,
}

impl WireGuardBuilder {
    /// Creates a new WireGuard connection builder.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable connection name
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let inner = ConnectionBuilder::new("wireguard", &name);

        Self {
            inner,
            name,
            private_key: None,
            address: None,
            peers: Vec::new(),
            dns: None,
            mtu: None,
            uuid: None,
        }
    }

    /// Sets the WireGuard private key.
    ///
    /// The key must be a valid base64-encoded 32-byte WireGuard key (44 characters).
    pub fn private_key(mut self, key: impl Into<String>) -> Self {
        self.private_key = Some(key.into());
        self
    }

    /// Sets the VPN interface IP address with CIDR notation.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nmrs::builders::WireGuardBuilder;
    /// let builder = WireGuardBuilder::new("MyVPN")
    ///     .address("10.0.0.2/24");
    /// ```
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Adds a WireGuard peer to the connection.
    ///
    /// At least one peer must be added before building.
    pub fn add_peer(mut self, peer: WireGuardPeer) -> Self {
        self.peers.push(peer);
        self
    }

    /// Adds multiple WireGuard peers at once.
    pub fn add_peers(mut self, peers: impl IntoIterator<Item = WireGuardPeer>) -> Self {
        self.peers.extend(peers);
        self
    }

    /// Sets DNS servers for the VPN connection.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use nmrs::builders::WireGuardBuilder;
    /// let builder = WireGuardBuilder::new("MyVPN")
    ///     .dns(vec!["1.1.1.1".into(), "8.8.8.8".into()]);
    /// ```
    pub fn dns(mut self, servers: Vec<String>) -> Self {
        self.dns = Some(servers);
        self
    }

    /// Sets the MTU (Maximum Transmission Unit) for the WireGuard interface.
    ///
    /// Typical value is 1420 for WireGuard over IPv4.
    pub fn mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Sets a specific UUID for the connection.
    ///
    /// If not set, a deterministic UUID will be generated based on the
    /// connection name.
    pub fn uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    // Delegation methods to inner ConnectionBuilder

    /// Applies connection options.
    pub fn options(mut self, opts: &ConnectionOptions) -> Self {
        self.inner = self.inner.options(opts);
        self
    }

    /// Enables or disables automatic connection.
    pub fn autoconnect(mut self, enabled: bool) -> Self {
        self.inner = self.inner.autoconnect(enabled);
        self
    }

    /// Sets autoconnect priority.
    pub fn autoconnect_priority(mut self, priority: i32) -> Self {
        self.inner = self.inner.autoconnect_priority(priority);
        self
    }

    /// Sets autoconnect retry limit.
    pub fn autoconnect_retries(mut self, retries: i32) -> Self {
        self.inner = self.inner.autoconnect_retries(retries);
        self
    }

    /// Builds the final WireGuard connection settings.
    ///
    /// This method validates all required fields and returns an error if
    /// any validation fails.
    ///
    /// # Errors
    ///
    /// - `ConnectionError::InvalidPrivateKey` if private key is missing or invalid
    /// - `ConnectionError::InvalidAddress` if address is missing or invalid
    /// - `ConnectionError::InvalidPeers` if no peers are configured or peer validation fails
    /// - `ConnectionError::InvalidGateway` if any peer gateway is invalid
    pub fn build(
        mut self,
    ) -> Result<HashMap<&'static str, HashMap<&'static str, Value<'static>>>, ConnectionError> {
        // Validate required fields
        let private_key = self
            .private_key
            .ok_or_else(|| ConnectionError::InvalidPrivateKey("Private key not set".into()))?;

        let address = self
            .address
            .ok_or_else(|| ConnectionError::InvalidAddress("Address not set".into()))?;

        if self.peers.is_empty() {
            return Err(ConnectionError::InvalidPeers("No peers configured".into()));
        }

        // Validate private key
        validate_wireguard_key(&private_key, "Private key")?;

        // Validate address
        let (ip, prefix) = validate_address(&address)?;

        // Validate each peer
        for (i, peer) in self.peers.iter().enumerate() {
            validate_wireguard_key(&peer.public_key, &format!("Peer {} public key", i))?;
            validate_gateway(&peer.gateway)?;

            if peer.allowed_ips.is_empty() {
                return Err(ConnectionError::InvalidPeers(format!(
                    "Peer {} has no allowed IPs",
                    i
                )));
            }
        }

        // Generate interface name
        let interface_name = format!(
            "wg-{}",
            self.name
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .take(10)
                .collect::<String>()
        );

        self.inner = self.inner.interface_name(&interface_name);

        // Set UUID (deterministic or provided)
        let uuid = self.uuid.unwrap_or_else(|| {
            // Generate deterministic UUID based on name
            Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("wg:{}", self.name).as_bytes())
        });

        self.inner = self.inner.uuid(uuid);

        // Build wireguard section
        let mut wireguard = HashMap::new();
        wireguard.insert(
            "service-type",
            Value::from("org.freedesktop.NetworkManager.wireguard"),
        );
        wireguard.insert("private-key", Value::from(private_key));

        // Build peers array
        let mut peers_array: Vec<HashMap<String, zvariant::Value<'static>>> = Vec::new();

        for peer in self.peers {
            let mut peer_dict: HashMap<String, zvariant::Value<'static>> = HashMap::new();

            peer_dict.insert("public-key".into(), Value::from(peer.public_key));
            peer_dict.insert("endpoint".into(), Value::from(peer.gateway));
            peer_dict.insert("allowed-ips".into(), Value::from(peer.allowed_ips));

            if let Some(psk) = peer.preshared_key {
                peer_dict.insert("preshared-key".into(), Value::from(psk));
            }

            if let Some(ka) = peer.persistent_keepalive {
                peer_dict.insert("persistent-keepalive".into(), Value::from(ka));
            }

            peers_array.push(peer_dict);
        }

        wireguard.insert("peers", Value::from(peers_array));

        if let Some(mtu) = self.mtu {
            wireguard.insert("mtu", Value::from(mtu));
        }

        self.inner = self.inner.with_section("wireguard", wireguard);

        // Configure IPv4 with manual addressing
        self.inner = self.inner.ipv4_manual(vec![IpConfig::new(ip, prefix)]);

        // Add DNS if configured
        if let Some(dns) = self.dns {
            let dns_addrs: Result<Vec<Ipv4Addr>, _> =
                dns.iter().map(|s| s.parse::<Ipv4Addr>()).collect();

            match dns_addrs {
                Ok(addrs) => {
                    self.inner = self.inner.ipv4_dns(addrs);
                }
                Err(_) => {
                    return Err(ConnectionError::VpnFailed(
                        "Invalid DNS server address".into(),
                    ));
                }
            }
        }

        // Add MTU to IPv4 if configured
        if let Some(mtu) = self.mtu {
            self.inner = self.inner.update_section("ipv4", |ipv4| {
                ipv4.insert("mtu", Value::from(mtu));
            });
        }

        // Set IPv6 to ignore
        self.inner = self.inner.ipv6_ignore();

        Ok(self.inner.build())
    }
}

// Validation functions (same as in vpn.rs)

fn validate_wireguard_key(key: &str, key_type: &str) -> Result<(), ConnectionError> {
    if key.trim().is_empty() {
        return Err(ConnectionError::InvalidPrivateKey(format!(
            "{} cannot be empty",
            key_type
        )));
    }

    let len = key.trim().len();
    if !(40..=50).contains(&len) {
        return Err(ConnectionError::InvalidPrivateKey(format!(
            "{} has invalid length: {} (expected ~44 characters)",
            key_type, len
        )));
    }

    let is_valid_base64 = key
        .trim()
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=');

    if !is_valid_base64 {
        return Err(ConnectionError::InvalidPrivateKey(format!(
            "{} contains invalid base64 characters",
            key_type
        )));
    }

    Ok(())
}

fn validate_address(address: &str) -> Result<(String, u32), ConnectionError> {
    let (ip, prefix) = address.split_once('/').ok_or_else(|| {
        ConnectionError::InvalidAddress(format!(
            "missing CIDR prefix (e.g., '10.0.0.2/24'): {}",
            address
        ))
    })?;

    if ip.trim().is_empty() {
        return Err(ConnectionError::InvalidAddress(
            "IP address cannot be empty".into(),
        ));
    }

    let prefix: u32 = prefix
        .parse()
        .map_err(|_| ConnectionError::InvalidAddress(format!("invalid CIDR prefix: {}", prefix)))?;

    if prefix > 128 {
        return Err(ConnectionError::InvalidAddress(format!(
            "CIDR prefix too large: {} (max 128)",
            prefix
        )));
    }

    // Basic IPv4 validation
    if ip.contains('.') {
        let octets: Vec<&str> = ip.split('.').collect();
        if octets.len() != 4 {
            return Err(ConnectionError::InvalidAddress(format!(
                "invalid IPv4 address: {}",
                ip
            )));
        }

        for octet in octets {
            let num: u32 = octet.parse().map_err(|_| {
                ConnectionError::InvalidAddress(format!("invalid IPv4 octet: {}", octet))
            })?;
            if num > 255 {
                return Err(ConnectionError::InvalidAddress(format!(
                    "IPv4 octet out of range: {}",
                    num
                )));
            }
        }

        if prefix > 32 {
            return Err(ConnectionError::InvalidAddress(format!(
                "IPv4 CIDR prefix too large: {} (max 32)",
                prefix
            )));
        }
    }

    Ok((ip.to_string(), prefix))
}

fn validate_gateway(gateway: &str) -> Result<(), ConnectionError> {
    if gateway.trim().is_empty() {
        return Err(ConnectionError::InvalidGateway(
            "gateway cannot be empty".into(),
        ));
    }

    if !gateway.contains(':') {
        return Err(ConnectionError::InvalidGateway(format!(
            "gateway must be in 'host:port' format: {}",
            gateway
        )));
    }

    let parts: Vec<&str> = gateway.rsplitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(ConnectionError::InvalidGateway(format!(
            "invalid gateway format: {}",
            gateway
        )));
    }

    let port_str = parts[0];
    let port: u16 = port_str.parse().map_err(|_| {
        ConnectionError::InvalidGateway(format!("invalid port number: {}", port_str))
    })?;

    if port == 0 {
        return Err(ConnectionError::InvalidGateway("port cannot be 0".into()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_peer() -> WireGuardPeer {
        WireGuardPeer {
            public_key: "HIgo9xNzJMWLKAShlKl6/bUT1VI9Q0SDBXGtLXkPFXc=".into(),
            gateway: "vpn.example.com:51820".into(),
            allowed_ips: vec!["0.0.0.0/0".into()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }
    }

    #[test]
    fn builds_basic_wireguard_connection() {
        let settings = WireGuardBuilder::new("TestVPN")
            .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
            .address("10.0.0.2/24")
            .add_peer(create_test_peer())
            .autoconnect(false)
            .build()
            .expect("Failed to build");

        assert!(settings.contains_key("connection"));
        assert!(settings.contains_key("wireguard"));
        assert!(settings.contains_key("ipv4"));
        assert!(settings.contains_key("ipv6"));

        let conn = settings.get("connection").unwrap();
        assert_eq!(conn.get("type"), Some(&Value::from("wireguard")));
    }

    #[test]
    fn requires_private_key() {
        let result = WireGuardBuilder::new("TestVPN")
            .address("10.0.0.2/24")
            .add_peer(create_test_peer())
            .build();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidPrivateKey(_)
        ));
    }

    #[test]
    fn requires_address() {
        let result = WireGuardBuilder::new("TestVPN")
            .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
            .add_peer(create_test_peer())
            .build();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidAddress(_)
        ));
    }

    #[test]
    fn requires_at_least_one_peer() {
        let result = WireGuardBuilder::new("TestVPN")
            .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
            .address("10.0.0.2/24")
            .build();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidPeers(_)
        ));
    }

    #[test]
    fn adds_dns_servers() {
        let settings = WireGuardBuilder::new("TestVPN")
            .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
            .address("10.0.0.2/24")
            .add_peer(create_test_peer())
            .dns(vec!["1.1.1.1".into(), "8.8.8.8".into()])
            .build()
            .expect("Failed to build");

        let ipv4 = settings.get("ipv4").unwrap();
        assert!(ipv4.contains_key("dns"));
    }

    #[test]
    fn sets_mtu() {
        let settings = WireGuardBuilder::new("TestVPN")
            .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
            .address("10.0.0.2/24")
            .add_peer(create_test_peer())
            .mtu(1420)
            .build()
            .expect("Failed to build");

        let wireguard = settings.get("wireguard").unwrap();
        assert_eq!(wireguard.get("mtu"), Some(&Value::from(1420u32)));
    }

    #[test]
    fn supports_multiple_peers() {
        let peer1 = create_test_peer();
        let peer2 = WireGuardPeer {
            public_key: "xScVkH3fUGUVRvGLFcjkx+GGD7cf5eBVyN3Gh4FLjmI=".into(),
            gateway: "peer2.example.com:51821".into(),
            allowed_ips: vec!["192.168.0.0/16".into()],
            preshared_key: None,
            persistent_keepalive: None,
        };

        let settings = WireGuardBuilder::new("TestVPN")
            .private_key("YBk6X3pP8KjKz7+HFWzVHNqL3qTZq8hX9VxFQJ4zVmM=")
            .address("10.0.0.2/24")
            .add_peers(vec![peer1, peer2])
            .build()
            .expect("Failed to build");

        assert!(settings.contains_key("wireguard"));
    }
}
