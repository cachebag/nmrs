//! OpenVPN connection builder with validation.
//!
//! Provides a type-safe builder API for constructing [`OpenVpnConfig`] with
//! validation of required fields and auth-type-specific requirements at
//! build time.
//!
//! Unlike [`super::vpn::build_wireguard_connection`] which returns NM-ready
//! D-Bus settings directly, this builder produces an [`OpenVpnConfig`] domain
//! struct. Use [`super::vpn::build_openvpn_connection`] to convert it into
//! NetworkManager connection settings.

use uuid::Uuid;

use crate::api::models::{
    ConnectionError, OpenVpnAuthType, OpenVpnCompression, OpenVpnConfig, OpenVpnProxy,
};
use crate::util::validation::validate_connection_name;

/// Builder for OpenVPN connections.
///
/// Validates at build time:
/// - `remote` must be set and non-empty
/// - `auth_type` must be set
/// - `Password` or `PasswordTls`: `username` required
/// - `Tls` or `PasswordTls`: `ca_cert`, `client_cert`, `client_key` required
/// - port must be 1–65535
///
/// # Example
///
/// ```rust
/// use nmrs::builders::OpenVpnBuilder;
/// use nmrs::OpenVpnAuthType;
///
/// let config = OpenVpnBuilder::new("CorpVPN")
///     .remote("vpn.example.com")
///     .port(1194)
///     .auth_type(OpenVpnAuthType::Tls)
///     .ca_cert("/etc/openvpn/ca.crt")
///     .client_cert("/etc/openvpn/client.crt")
///     .client_key("/etc/openvpn/client.key")
///     .build()
///     .expect("Failed to build OpenVPN config");
/// ```
pub struct OpenVpnBuilder {
    name: String,
    remote: Option<String>,
    port: Option<u16>,
    tcp: bool,
    auth_type: Option<OpenVpnAuthType>,
    auth: Option<String>,
    cipher: Option<String>,
    dns: Option<Vec<String>>,
    mtu: Option<u32>,
    uuid: Option<Uuid>,
    ca_cert: Option<String>,
    client_cert: Option<String>,
    client_key: Option<String>,
    key_password: Option<String>,
    username: Option<String>,
    password: Option<String>,
    compression: Option<OpenVpnCompression>,
    proxy: Option<OpenVpnProxy>,
    tls_auth_key: Option<String>,
    tls_auth_direction: Option<u8>,
    tls_crypt: Option<String>,
    tls_crypt_v2: Option<String>,
    tls_version_min: Option<String>,
    tls_version_max: Option<String>,
    tls_cipher: Option<String>,
    remote_cert_tls: Option<String>,
    verify_x509_name: Option<(String, String)>,
    crl_verify: Option<String>,
}

impl OpenVpnBuilder {
    /// Creates a new OpenVPN connection builder.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            remote: None,
            port: None,
            tcp: false,
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
            tls_auth_key: None,
            tls_auth_direction: None,
            tls_crypt: None,
            tls_crypt_v2: None,
            tls_version_min: None,
            tls_version_max: None,
            tls_cipher: None,
            remote_cert_tls: None,
            verify_x509_name: None,
            crl_verify: None,
        }
    }

    /// Sets the remote server hostname or IP address.
    #[must_use]
    pub fn remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = Some(remote.into());
        self
    }

    /// Sets the remote server port (1–65535).
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Use TCP instead of UDP.
    #[must_use]
    pub fn tcp(mut self, tcp: bool) -> Self {
        self.tcp = tcp;
        self
    }

    /// Sets the authentication type.
    #[must_use]
    pub fn auth_type(mut self, auth_type: OpenVpnAuthType) -> Self {
        self.auth_type = Some(auth_type);
        self
    }

    /// Sets the HMAC digest algorithm (e.g. "SHA256").
    #[must_use]
    pub fn auth(mut self, auth: impl Into<String>) -> Self {
        self.auth = Some(auth.into());
        self
    }

    /// Sets the data channel cipher (e.g. "AES-256-GCM").
    #[must_use]
    pub fn cipher(mut self, cipher: impl Into<String>) -> Self {
        self.cipher = Some(cipher.into());
        self
    }

    /// Sets DNS servers for the connection.
    #[must_use]
    pub fn dns(mut self, servers: Vec<String>) -> Self {
        self.dns = Some(servers);
        self
    }

    /// Sets the MTU size.
    #[must_use]
    pub fn mtu(mut self, mtu: u32) -> Self {
        self.mtu = Some(mtu);
        self
    }

    /// Sets a specific UUID for the connection.
    #[must_use]
    pub fn uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    /// Sets the CA certificate path.
    #[must_use]
    pub fn ca_cert(mut self, path: impl Into<String>) -> Self {
        self.ca_cert = Some(path.into());
        self
    }

    /// Sets the client certificate path.
    #[must_use]
    pub fn client_cert(mut self, path: impl Into<String>) -> Self {
        self.client_cert = Some(path.into());
        self
    }

    /// Sets the client private key path.
    #[must_use]
    pub fn client_key(mut self, path: impl Into<String>) -> Self {
        self.client_key = Some(path.into());
        self
    }

    /// Sets the password for an encrypted private key.
    #[must_use]
    pub fn key_password(mut self, password: impl Into<String>) -> Self {
        self.key_password = Some(password.into());
        self
    }

    /// Sets the username for password authentication.
    #[must_use]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the password for password authentication.
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
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
    pub fn compression(mut self, compression: OpenVpnCompression) -> Self {
        self.compression = Some(compression);
        self
    }

    /// Sets the proxy configuration.
    #[must_use]
    pub fn proxy(mut self, proxy: OpenVpnProxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Sets the TLS authentication key path and optional direction.
    #[must_use]
    pub fn tls_auth(mut self, key_path: impl Into<String>, direction: Option<u8>) -> Self {
        self.tls_auth_key = Some(key_path.into());
        self.tls_auth_direction = direction;
        self
    }

    /// Sets the TLS-Crypt key path.
    #[must_use]
    pub fn tls_crypt(mut self, key_path: impl Into<String>) -> Self {
        self.tls_crypt = Some(key_path.into());
        self
    }

    /// Sets the TLS-Crypt-v2 key path.
    #[must_use]
    pub fn tls_crypt_v2(mut self, key_path: impl Into<String>) -> Self {
        self.tls_crypt_v2 = Some(key_path.into());
        self
    }

    /// Sets the minimum TLS protocol version.
    #[must_use]
    pub fn tls_version_min(mut self, version: impl Into<String>) -> Self {
        self.tls_version_min = Some(version.into());
        self
    }

    /// Sets the maximum TLS protocol version.
    #[must_use]
    pub fn tls_version_max(mut self, version: impl Into<String>) -> Self {
        self.tls_version_max = Some(version.into());
        self
    }

    /// Sets the control channel TLS cipher suites.
    #[must_use]
    pub fn tls_cipher(mut self, cipher: impl Into<String>) -> Self {
        self.tls_cipher = Some(cipher.into());
        self
    }

    /// Requires the remote certificate to be of a specific type.
    #[must_use]
    pub fn remote_cert_tls(mut self, cert_type: impl Into<String>) -> Self {
        self.remote_cert_tls = Some(cert_type.into());
        self
    }

    /// Sets X.509 name verification for the remote certificate.
    #[must_use]
    pub fn verify_x509_name(
        mut self,
        name: impl Into<String>,
        name_type: impl Into<String>,
    ) -> Self {
        self.verify_x509_name = Some((name.into(), name_type.into()));
        self
    }

    /// Sets the path to a Certificate Revocation List.
    #[must_use]
    pub fn crl_verify(mut self, path: impl Into<String>) -> Self {
        self.crl_verify = Some(path.into());
        self
    }

    /// Builds and validates the `OpenVpnConfig`.
    ///
    /// # Errors
    ///
    /// - `ConnectionError::InvalidGateway` if `remote` is not set or empty
    /// - `ConnectionError::InvalidGateway` if `port` is 0
    /// - `ConnectionError::VpnFailed` if `auth_type` is not set
    /// - `ConnectionError::VpnFailed` if `username` is required but missing
    /// - `ConnectionError::VpnFailed` if TLS certs are required but missing
    pub fn build(self) -> Result<OpenVpnConfig, ConnectionError> {
        validate_connection_name(&self.name)?;

        let remote = self
            .remote
            .ok_or_else(|| ConnectionError::InvalidGateway("remote must be set".into()))?;
        if remote.trim().is_empty() {
            return Err(ConnectionError::InvalidGateway(
                "remote must not be empty".into(),
            ));
        }

        // Validate port
        let port = self.port.unwrap_or(1194);
        if port == 0 {
            return Err(ConnectionError::InvalidGateway(
                "port must be between 1 and 65535".into(),
            ));
        }

        // Validate auth_type
        let auth_type = self
            .auth_type
            .ok_or_else(|| ConnectionError::VpnFailed("auth_type must be set".into()))?;

        // auth_type-specific validation
        match &auth_type {
            OpenVpnAuthType::Password | OpenVpnAuthType::PasswordTls => {
                if self.username.is_none() {
                    return Err(ConnectionError::VpnFailed(
                        "username is required for Password and PasswordTls auth".into(),
                    ));
                }
            }
            _ => {}
        }

        if matches!(auth_type, OpenVpnAuthType::StaticKey) {
            return Err(ConnectionError::VpnFailed(
                "StaticKey auth validation is not yet implemented".into(),
            ));
        }

        match &auth_type {
            OpenVpnAuthType::Tls | OpenVpnAuthType::PasswordTls => {
                if self.ca_cert.is_none() {
                    return Err(ConnectionError::VpnFailed(
                        "ca_cert is required for Tls and PasswordTls auth".into(),
                    ));
                }
                if self.client_cert.is_none() {
                    return Err(ConnectionError::VpnFailed(
                        "client_cert is required for Tls and PasswordTls auth".into(),
                    ));
                }
                if self.client_key.is_none() {
                    return Err(ConnectionError::VpnFailed(
                        "client_key is required for Tls and PasswordTls auth".into(),
                    ));
                }
            }
            _ => {}
        }

        Ok(OpenVpnConfig {
            name: self.name,
            remote,
            port,
            tcp: self.tcp,
            auth_type: Some(auth_type),
            auth: self.auth,
            cipher: self.cipher,
            dns: self.dns,
            mtu: self.mtu,
            uuid: self.uuid,
            ca_cert: self.ca_cert,
            client_cert: self.client_cert,
            client_key: self.client_key,
            key_password: self.key_password,
            username: self.username,
            password: self.password,
            compression: self.compression,
            proxy: self.proxy,
            tls_auth_key: self.tls_auth_key,
            tls_auth_direction: self.tls_auth_direction,
            tls_crypt: self.tls_crypt,
            tls_crypt_v2: self.tls_crypt_v2,
            tls_version_min: self.tls_version_min,
            tls_version_max: self.tls_version_max,
            tls_cipher: self.tls_cipher,
            remote_cert_tls: self.remote_cert_tls,
            verify_x509_name: self.verify_x509_name,
            crl_verify: self.crl_verify,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tls_builder() -> OpenVpnBuilder {
        OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .port(1194)
            .auth_type(OpenVpnAuthType::Tls)
            .ca_cert("/etc/openvpn/ca.crt")
            .client_cert("/etc/openvpn/client.crt")
            .client_key("/etc/openvpn/client.key")
    }

    fn password_builder() -> OpenVpnBuilder {
        OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .port(1194)
            .auth_type(OpenVpnAuthType::Password)
            .username("user")
    }

    #[test]
    fn builds_tls_connection() {
        let config = tls_builder().build();
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.name, "TestVPN");
        assert_eq!(config.remote, "vpn.example.com");
        assert_eq!(config.port, 1194);
        assert!(!config.tcp);
    }

    #[test]
    fn builds_password_connection() {
        let config = password_builder().build();
        assert!(config.is_ok());
    }

    #[test]
    fn builds_password_tls_connection() {
        let config = OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .auth_type(OpenVpnAuthType::PasswordTls)
            .username("user")
            .ca_cert("/etc/openvpn/ca.crt")
            .client_cert("/etc/openvpn/client.crt")
            .client_key("/etc/openvpn/client.key")
            .build();
        assert!(config.is_ok());
    }

    #[test]
    fn rejects_static_key_unimplemented() {
        let result = OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .auth_type(OpenVpnAuthType::StaticKey)
            .build();
        assert!(matches!(result.unwrap_err(), ConnectionError::VpnFailed(_)));
    }

    #[test]
    fn defaults_port_to_1194() {
        let config = tls_builder().build().unwrap();
        assert_eq!(config.port, 1194);
    }

    #[test]
    fn sets_tcp_flag() {
        let config = tls_builder().tcp(true).build().unwrap();
        assert!(config.tcp);
    }

    #[test]
    fn sets_optional_fields() {
        let config = tls_builder()
            .auth("SHA256")
            .cipher("AES-256-GCM")
            .mtu(1400)
            .dns(vec!["1.1.1.1".into()])
            .build()
            .unwrap();
        assert_eq!(config.auth, Some("SHA256".into()));
        assert_eq!(config.cipher, Some("AES-256-GCM".into()));
        assert_eq!(config.mtu, Some(1400));
        assert!(config.dns.is_some());
    }

    #[test]
    fn sets_compression() {
        let config = tls_builder()
            .compression(OpenVpnCompression::Lz4V2)
            .build()
            .unwrap();
        assert_eq!(config.compression, Some(OpenVpnCompression::Lz4V2));
    }

    #[test]
    fn sets_proxy() {
        let config = tls_builder()
            .proxy(OpenVpnProxy::Http {
                server: "proxy.example.com".into(),
                port: 8080,
                username: None,
                password: None,
                retry: false,
            })
            .build()
            .unwrap();
        assert!(config.proxy.is_some());
    }

    #[test]
    fn rejects_empty_name() {
        let result = OpenVpnBuilder::new("")
            .remote("vpn.example.com")
            .auth_type(OpenVpnAuthType::Tls)
            .ca_cert("/etc/openvpn/ca.crt")
            .client_cert("/etc/openvpn/client.crt")
            .client_key("/etc/openvpn/client.key")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn requires_remote() {
        let result = OpenVpnBuilder::new("TestVPN")
            .auth_type(OpenVpnAuthType::Tls)
            .ca_cert("/etc/openvpn/ca.crt")
            .client_cert("/etc/openvpn/client.crt")
            .client_key("/etc/openvpn/client.key")
            .build();
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_empty_remote() {
        let result = OpenVpnBuilder::new("TestVPN")
            .remote("")
            .auth_type(OpenVpnAuthType::Tls)
            .ca_cert("/etc/openvpn/ca.crt")
            .client_cert("/etc/openvpn/client.crt")
            .client_key("/etc/openvpn/client.key")
            .build();
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn rejects_zero_port() {
        let result = tls_builder().port(0).build();
        assert!(matches!(
            result.unwrap_err(),
            ConnectionError::InvalidGateway(_)
        ));
    }

    #[test]
    fn requires_auth_type() {
        let result = OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .build();
        assert!(matches!(result.unwrap_err(), ConnectionError::VpnFailed(_)));
    }

    #[test]
    fn requires_username_for_password_auth() {
        let result = OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .auth_type(OpenVpnAuthType::Password)
            .build();
        assert!(matches!(result.unwrap_err(), ConnectionError::VpnFailed(_)));
    }

    #[test]
    fn requires_username_for_password_tls_auth() {
        let result = OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .auth_type(OpenVpnAuthType::PasswordTls)
            .ca_cert("/etc/openvpn/ca.crt")
            .client_cert("/etc/openvpn/client.crt")
            .client_key("/etc/openvpn/client.key")
            .build();
        assert!(matches!(result.unwrap_err(), ConnectionError::VpnFailed(_)));
    }

    #[test]
    fn requires_ca_cert_for_tls_auth() {
        let result = OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .auth_type(OpenVpnAuthType::Tls)
            .client_cert("/etc/openvpn/client.crt")
            .client_key("/etc/openvpn/client.key")
            .build();
        assert!(matches!(result.unwrap_err(), ConnectionError::VpnFailed(_)));
    }

    #[test]
    fn requires_client_cert_for_tls_auth() {
        let result = OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .auth_type(OpenVpnAuthType::Tls)
            .ca_cert("/etc/openvpn/ca.crt")
            .client_key("/etc/openvpn/client.key")
            .build();
        assert!(matches!(result.unwrap_err(), ConnectionError::VpnFailed(_)));
    }

    #[test]
    fn requires_client_key_for_tls_auth() {
        let result = OpenVpnBuilder::new("TestVPN")
            .remote("vpn.example.com")
            .auth_type(OpenVpnAuthType::Tls)
            .ca_cert("/etc/openvpn/ca.crt")
            .client_cert("/etc/openvpn/client.crt")
            .build();
        assert!(matches!(result.unwrap_err(), ConnectionError::VpnFailed(_)));
    }
}
