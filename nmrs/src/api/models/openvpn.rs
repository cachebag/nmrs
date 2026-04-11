#![allow(deprecated)]

use super::vpn::{VpnConfig, VpnType};
use crate::api::models::error::ConnectionError;
use std::convert::TryFrom;
use uuid::Uuid;

/// OpenVPN authentication type.
///
/// Specifies how the client authenticates with the OpenVPN server.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenVpnAuthType {
    /// Username/password authentication only.
    Password,
    /// TLS certificate authentication only.
    Tls,
    /// Both password and TLS certificate authentication.
    PasswordTls,
    /// Static key authentication (pre-shared key).
    StaticKey,
}

/// OpenVPN connection configuration.
///
/// Stores the necessary information to configure and connect to an OpenVPN server.
///
/// # Example
///
/// ```rust
/// use nmrs::{OpenVpnConfig, OpenVpnAuthType};
///
/// let config = OpenVpnConfig::new("MyVPN", "vpn.example.com", 1194, false)
///     .with_auth_type(OpenVpnAuthType::PasswordTls)
///     .with_username("user")
///     .with_password("secret")
///     .with_ca_cert("/path/to/ca.crt")
///     .with_dns(vec!["1.1.1.1".into()]);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct OpenVpnConfig {
    /// Connection name.
    pub name: String,
    /// Remote server hostname or IP.
    pub remote: String,
    /// Remote server port (default: 1194).
    pub port: u16,
    /// Use TCP instead of UDP.
    pub tcp: bool,
    /// Authentication type.
    pub auth_type: Option<OpenVpnAuthType>,
    /// HMAC digest algorithm (e.g., "SHA256").
    pub auth: Option<String>,
    /// Data channel cipher (e.g., "AES-256-GCM").
    pub cipher: Option<String>,
    /// DNS servers to use when connected.
    pub dns: Option<Vec<String>>,
    /// MTU size.
    pub mtu: Option<u32>,
    /// Connection UUID.
    pub uuid: Option<Uuid>,
    /// Path to CA certificate.
    pub ca_cert: Option<String>,
    /// Path to client certificate.
    pub client_cert: Option<String>,
    /// Path to client private key.
    pub client_key: Option<String>,
    /// Password for encrypted private key.
    pub key_password: Option<String>,
    /// Username for password authentication.
    pub username: Option<String>,
    /// Password for password authentication.
    pub password: Option<String>,
    /// Compression algorithm. See [`OpenVpnCompression`] for security considerations.
    pub compression: Option<OpenVpnCompression>,
    /// Proxy configuration.
    pub proxy: Option<OpenVpnProxy>,
}

impl OpenVpnConfig {
    /// Creates a new `OpenVpnConfig` with required fields.
    pub fn new(name: impl Into<String>, remote: impl Into<String>, port: u16, tcp: bool) -> Self {
        Self {
            name: name.into(),
            remote: remote.into(),
            port,
            tcp,
            auth_type: None,
            auth: None,
            cipher: None,
            dns: None,
            mtu: None,
            uuid: None,
            ca_cert: None,
            client_cert: None,
            client_key: None,
            key_password: None,
            username: None,
            password: None,
            compression: None,
            proxy: None,
        }
    }

    /// Sets the authentication type.
    #[must_use]
    pub fn with_auth_type(mut self, auth_type: OpenVpnAuthType) -> Self {
        self.auth_type = Some(auth_type);
        self
    }

    /// Sets the HMAC digest algorithm.
    #[must_use]
    pub fn with_auth(mut self, auth: impl Into<String>) -> Self {
        self.auth = Some(auth.into());
        self
    }

    /// Sets the data channel cipher.
    #[must_use]
    pub fn with_cipher(mut self, cipher: impl Into<String>) -> Self {
        self.cipher = Some(cipher.into());
        self
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

    /// Sets the CA certificate path.
    #[must_use]
    pub fn with_ca_cert(mut self, path: impl Into<String>) -> Self {
        self.ca_cert = Some(path.into());
        self
    }

    /// Sets the client certificate path.
    #[must_use]
    pub fn with_client_cert(mut self, path: impl Into<String>) -> Self {
        self.client_cert = Some(path.into());
        self
    }

    /// Sets the client private key path.
    #[must_use]
    pub fn with_client_key(mut self, path: impl Into<String>) -> Self {
        self.client_key = Some(path.into());
        self
    }

    /// Sets the password for an encrypted private key.
    #[must_use]
    pub fn with_key_password(mut self, password: impl Into<String>) -> Self {
        self.key_password = Some(password.into());
        self
    }

    /// Sets the username for password authentication.
    #[must_use]
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the password for password authentication.
    #[must_use]
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the compression algorithm.
    ///
    /// # Security Warning
    ///
    /// Some compression modes are subject to the VORACLE vulnerability.
    /// See [`OpenVpnCompression`] for details and recommendations.
    #[must_use]
    pub fn with_compression(mut self, compression: OpenVpnCompression) -> Self {
        self.compression = Some(compression);
        self
    }

    /// Sets the proxy configuration.
    #[must_use]
    pub fn with_proxy(mut self, proxy: OpenVpnProxy) -> Self {
        self.proxy = Some(proxy);
        self
    }
}

impl TryFrom<crate::core::ovpn_parser::parser::OvpnFile> for OpenVpnConfig {
    type Error = ConnectionError;

    fn try_from(f: crate::core::ovpn_parser::parser::OvpnFile) -> Result<Self, Self::Error> {
        use crate::core::ovpn_parser::parser::{AllowCompress, CertSource, Compress};

        let first_remote = f
            .remotes
            .into_iter()
            .next()
            .ok_or_else(|| ConnectionError::InvalidGateway("no remote in .ovpn file".into()))?;

        let tcp = first_remote
            .proto
            .as_deref()
            .map(|p: &str| p.starts_with("tcp"))
            .unwrap_or_else(|| {
                f.proto
                    .as_deref()
                    .map(|p: &str| p.starts_with("tcp"))
                    .unwrap_or(false)
            });

        let compression = match (f.compress, f.allow_compress) {
            (Some(Compress::Algorithm(ref s)), _) => Some(match s.as_str() {
                "lz4" => OpenVpnCompression::Lz4,
                "lz4-v2" => OpenVpnCompression::Lz4V2,
                _ => OpenVpnCompression::Yes,
            }),
            (Some(Compress::Stub | Compress::StubV2), _) => Some(OpenVpnCompression::No),
            (None, Some(AllowCompress::No)) => Some(OpenVpnCompression::No),
            _ => None,
        };

        // FIXME: inline certs (<ca>, <cert>, <key> blocks) are parsed by
        // the .ovpn parser but NM needs them written to temp files or passed
        // via vpn.secrets. For now we return None so the caller knows the cert
        // field wasn't usable.
        let cert_path = |src: CertSource| -> Option<String> {
            match src {
                CertSource::File(p) => Some(p),
                CertSource::Inline(_) => None,
            }
        };

        Ok(OpenVpnConfig {
            name: String::new(),
            remote: first_remote.host,
            port: first_remote.port.unwrap_or(1194),
            tcp,
            auth_type: None,
            auth: f.auth,
            cipher: f.cipher,
            dns: None,
            mtu: None,
            uuid: None,
            ca_cert: f.ca.and_then(cert_path),
            client_cert: f.cert.and_then(cert_path),
            client_key: f.key.and_then(cert_path),
            key_password: None,
            username: None,
            password: None,
            compression,
            proxy: None,
        })
    }
}

impl VpnConfig for OpenVpnConfig {
    fn vpn_type(&self) -> VpnType {
        VpnType::OpenVpn
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

/// Compression algorithm for OpenVPN connections.
///
/// Maps to the NM `compress` and `comp-lzo` keys in the `vpn.data` dict.
///
/// # Security Warning
///
/// Compression is generally discouraged due to the VORACLE vulnerability,
/// where compression oracles can be exploited to recover plaintext from
/// encrypted tunnels. OpenVPN 2.5+ defaults to `--allow-compression no`.
/// Prefer [`No`](OpenVpnCompression::No) unless you have a specific need
/// and understand the risk. See <https://community.openvpn.net/openvpn/wiki/VORACLE>.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenVpnCompression {
    /// Disable compression explicitly. Recommended default.
    ///
    /// Maps to `compress no` in the NM `vpn.data` dict.
    No,

    /// LZO compression.
    ///
    /// Maps to `comp-lzo yes` in the NM `vpn.data` dict.
    ///
    /// # Security Warning
    ///
    /// Subject to the VORACLE vulnerability. See [`OpenVpnCompression`] docs.
    ///
    /// # Deprecation
    ///
    /// `comp-lzo` is deprecated upstream in OpenVPN in favour of the newer
    /// `compress` directive. Use [`Lz4V2`](OpenVpnCompression::Lz4V2) if
    /// you need compression, or [`No`](OpenVpnCompression::No) to disable it.
    #[deprecated(note = "comp-lzo is deprecated upstream. Use Lz4V2 or No instead.")]
    Lzo,

    /// LZ4 compression.
    ///
    /// Maps to `compress lz4` in the NM `vpn.data` dict.
    ///
    /// # Security Warning
    ///
    /// Subject to the VORACLE vulnerability. See [`OpenVpnCompression`] docs.
    Lz4,

    /// LZ4 v2 compression.
    ///
    /// Maps to `compress lz4-v2` in the NM `vpn.data` dict.
    ///
    /// # Security Warning
    ///
    /// Subject to the VORACLE vulnerability. See [`OpenVpnCompression`] docs.
    Lz4V2,

    /// Adaptive compression — algorithm negotiated at runtime.
    ///
    /// Maps to `compress yes` in the NM `vpn.data` dict.
    ///
    /// # Security Warning
    ///
    /// Subject to the VORACLE vulnerability. See [`OpenVpnCompression`] docs.
    Yes,
}

/// Proxy configuration for OpenVPN connections.
///
/// Maps to the NM `proxy-type`, `proxy-server`, `proxy-port`,
/// `proxy-retry`, `http-proxy-username`, and `http-proxy-password` keys.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenVpnProxy {
    /// HTTP proxy.
    Http {
        server: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
        retry: bool,
    },
    /// SOCKS proxy.
    Socks {
        server: String,
        port: u16,
        retry: bool,
    },
}
