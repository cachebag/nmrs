use serde::{Deserialize, Serialize};

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
    #[must_use]
    pub fn builder() -> EapOptionsBuilder {
        EapOptionsBuilder::default()
    }

    /// Sets the anonymous identity for privacy.
    #[must_use]
    pub fn with_anonymous_identity(mut self, anonymous_identity: impl Into<String>) -> Self {
        self.anonymous_identity = Some(anonymous_identity.into());
        self
    }

    /// Sets the domain suffix to match against the server certificate.
    #[must_use]
    pub fn with_domain_suffix_match(mut self, domain: impl Into<String>) -> Self {
        self.domain_suffix_match = Some(domain.into());
        self
    }

    /// Sets the path to the CA certificate file (must start with `file://`).
    #[must_use]
    pub fn with_ca_cert_path(mut self, path: impl Into<String>) -> Self {
        self.ca_cert_path = Some(path.into());
        self
    }

    /// Sets whether to use the system CA certificate store.
    #[must_use]
    pub fn with_system_ca_certs(mut self, use_system: bool) -> Self {
        self.system_ca_certs = use_system;
        self
    }

    /// Sets the EAP method (PEAP or TTLS).
    #[must_use]
    pub fn with_method(mut self, method: EapMethod) -> Self {
        self.method = method;
        self
    }

    /// Sets the Phase 2 authentication method.
    #[must_use]
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
    #[must_use]
    pub fn identity(mut self, identity: impl Into<String>) -> Self {
        self.identity = Some(identity.into());
        self
    }

    /// Sets the password for authentication.
    ///
    /// This is a required field.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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

impl WifiSecurity {
    /// Returns `true` if this security type requires authentication.
    #[must_use]
    pub fn secured(&self) -> bool {
        !matches!(self, WifiSecurity::Open)
    }

    /// Returns `true` if this is a WPA-PSK (password-based) security type.
    #[must_use]
    pub fn is_psk(&self) -> bool {
        matches!(self, WifiSecurity::WpaPsk { .. })
    }

    /// Returns `true` if this is a WPA-EAP (Enterprise/802.1X) security type.
    #[must_use]
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
