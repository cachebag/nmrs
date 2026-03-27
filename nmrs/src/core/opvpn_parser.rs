use std::collections::HashMap;

use crate::api::models::ConnectionError;

// FIXME: Change when #309 lands
// https://github.com/cachebag/nmrs/pull/309/changes
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum OpenVpnAuthType {
    Password,
    Tls,
    PasswordTls,
    StaticKey,
}

#[derive(Debug, Clone)]
pub struct OvpnFile {
    // All remote entries. Each defines a possible server endpoint.
    // OpenVPN tries them in order unless configured otherwise.
    pub remotes: Vec<Remote>,

    // device directive (e.g. "tun", "tap").
    // Controls the virtual network interface type.
    pub dev: Option<String>,

    // protocol directive (e.g. "udp", "tcp-client").
    pub proto: Option<String>,

    // ca directive. Certificate Authority used to verify server cert.
    // Supports file path or inline block.
    pub ca: Option<CertSource>,

    // cert directive. Client certificate.
    pub cert: Option<CertSource>,

    // key directive. Private key corresponding to cert.
    pub key: Option<CertSource>,

    // tls-auth directive. HMAC key used for additional packet auth.
    // This may include key-direction (0/1).
    pub tls_auth: Option<TlsAuth>,

    // tls-crypt directive. Encrypts control channel metadata.
    pub tls_crypt: Option<CertSource>,

    // cipher directive. Legacy data channel cipher (deprecated in newer configs).
    pub cipher: Option<String>,

    // data-ciphers directive. Preferred list of ciphers (this is colon-separated).
    pub data_ciphers: Vec<String>,

    // auth directive. HMAC digest algorithm (e.g. SHA256).
    pub auth: Option<String>,

    // compress directive. Either enabled or specifies algorithm (e.g. "lz4").
    pub compress: Option<Compress>,

    // All route directives.
    // Each represents a network route pushed or defined locally.
    pub routes: Vec<Route>,

    // redirect-gateway flag.
    // Forces all traffic through VPN if present.
    pub redirect_gateway: bool,

    // Standalone flag directives with no arguments.
    // Examples: client, nobind, persist-key, persist-tun.
    pub flags: Vec<String>,

    // Catch-all for unmodeled or less common directives.
    // Key = directive name, Value = list of argument lists.
    // Preserves information for round-tripping / forward compatibility.
    pub options: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Remote {
    pub host: String,
    pub port: Option<u16>,
    pub proto: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CertSource {
    File(String),
    Inline(String),
}

#[derive(Debug, Clone)]
pub struct TlsAuth {
    pub source: CertSource,
    pub key_direction: Option<u8>,
}

#[derive(Debug, Clone)]
pub enum Compress {
    Enabled,
    Algorithm(String),
}

#[derive(Debug, Clone)]
pub struct Route {
    pub network: String,
    pub netmask: Option<String>,
    pub gateway: Option<String>,
}

pub fn parse_ovpn(content: &str) -> Result<OvpnFile, ConnectionError> {
    todo!()
}
