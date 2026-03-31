use std::collections::HashMap;
use std::net::Ipv4Addr;

use crate::api::models::ConnectionError;
use crate::core::ovpn_parser::error::OvpnParseError;

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
    pub compress: Option<String>,

    // OpenVPN 2.5+ specifies a allow-compress directive for safety
    // https://community.openvpn.net/Security%20Announcements/VORACLE
    pub allow_compress: Option<AllowCompress>,

    // All route directives.
    // Each represents a network route pushed or defined locally.
    pub routes: Vec<Route>,

    // redirect-gateway flag.
    // Forces all traffic through VPN if present.
    pub redirect_gateway: Option<RedirectGateway>,

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
pub enum AllowCompress {
    Yes,
    No,
    Asym,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Route {
    pub network: Ipv4Addr,
    pub netmask: Option<Ipv4Addr>,
    pub gateway: Option<Ipv4Addr>,
}

#[derive(Debug, Clone)]
pub struct RedirectGateway {
    pub def1: bool,
    pub bypass_dhcp: bool,
    pub bypass_dns: bool,
    pub local: bool,
    pub ipv6: bool,
}

enum OvpnItem {
    Directive { key: String, args: Vec<String> },
    Block { key: String, content: String },
}

fn lexer(input: &str) -> Result<Vec<OvpnItem>, OvpnParseError> {
    let mut items = Vec::new();

    let mut current_line = String::new();
    let mut continuing = false;

    let mut in_block: Option<String> = None;
    let mut block_buffer = String::new();
    let mut block_line_start = 0;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_number = idx + 1;
        let line = raw_line;

        // We're in a block
        if let Some(block_name) = &in_block {
            let trimmed = line.trim();

            if trimmed.starts_with("</") && trimmed.ends_with(">") {
                let end_tag = trimmed[2..trimmed.len() - 1].trim().to_lowercase();

                if end_tag == *block_name {
                    items.push(OvpnItem::Block {
                        key: block_name.clone(),
                        content: block_buffer.clone(),
                    });

                    in_block = None;
                    block_buffer.clear();
                    continue;
                } else {
                    return Err(OvpnParseError::UnexpectedBlockEnd {
                        block: end_tag,
                        line: line_number,
                    });
                }
            }

            block_buffer.push_str(line);
            block_buffer.push('\n');

            continue;
        }

        // Continuation
        if continuing {
            current_line.push(' ');
            current_line.push_str(line.trim_start());
        } else {
            current_line.clear();
            current_line.push_str(line);
        }

        if current_line.ends_with('\\') {
            continuing = true;
            current_line.pop();
            continue;
        } else {
            continuing = false;
        }

        let line = current_line.trim();

        // Remove comments
        let mut cleaned = String::new();
        let mut prev_whitespace = true;

        for c in line.chars() {
            if (c == '#' || c == ';') && prev_whitespace {
                break;
            }

            prev_whitespace = c.is_whitespace();
            cleaned.push(c);
        }

        current_line.clear();
        let line = cleaned.trim();

        if line.is_empty() {
            continue;
        }

        if line.starts_with('<') && line.ends_with('>') && !line.starts_with("</") {
            let key = line[1..line.len() - 1].trim().to_lowercase();

            if key.is_empty() {
                return Err(OvpnParseError::InvalidDirectiveSyntax { line: line_number });
            }

            in_block = Some(key);
            block_line_start = line_number;
            block_buffer.clear();
            continue;
        }

        if line.starts_with("</") && line.ends_with('>') {
            let key = line[2..line.len() - 1].trim().to_lowercase();

            return Err(OvpnParseError::UnexpectedBlockEnd {
                block: key,
                line: line_number,
            });
        }

        let mut parts = line.split_whitespace();
        let key = match parts.next() {
            Some(k) => k.to_lowercase(),
            None => {
                return Err(OvpnParseError::InvalidDirectiveSyntax { line: line_number });
            }
        };

        let args: Vec<String> = parts.map(|s| s.to_string()).collect();

        items.push(OvpnItem::Directive { key, args });
    }

    if continuing {
        return Err(OvpnParseError::InvalidContinuation {
            line: input.lines().count(),
        });
    }

    if let Some(block) = in_block {
        return Err(OvpnParseError::UnterminatedBlock {
            block,
            line: block_line_start,
        });
    }

    Ok(items)
}

pub fn parse_ovpn(content: &str) -> Result<OvpnFile, ConnectionError> {
    let mut remotes: Vec<Remote> = Vec::new();
    let mut dev: Option<String> = None;
    let mut proto: Option<String> = None;
    let mut ca: Option<CertSource> = None;
    let mut cert: Option<CertSource> = None;
    let mut key: Option<CertSource> = None;
    let mut tls_auth: Option<TlsAuth> = None;
    let mut tls_crypt: Option<CertSource> = None;
    let mut cipher: Option<String> = None;
    let mut data_ciphers: Vec<String> = Vec::new();
    let mut auth: Option<String> = None;
    let mut compress: Option<String> = None;
    let mut allow_compress: Option<AllowCompress> = None;
    let mut routes: Vec<Route> = Vec::new();
    let mut redirect_gateway: Option<RedirectGateway> = None;
    let mut flags: Vec<String> = Vec::new();
    let mut options: HashMap<String, Vec<String>> = HashMap::new();

    let items = lexer(content)?;

    for item in items {
        match item {
            OvpnItem::Directive { key, args } => {
                match key.as_str() {
                    "remote" => {
                        // remote <HOST> [PORT] [PROTO]

                        let host = args
                            .get(0)
                            .ok_or(OvpnParseError::InvalidArgument {
                                key: key.clone(),
                                // FIXME: Add `MissingArgument` variant to allow for
                                // omitting args
                                arg: "".into(),
                                line: 0, // FIXME: track lines
                            })?
                            .clone();

                        let port = args.get(1).and_then(|p| p.parse::<u16>().ok()); // FIXME: ehh...

                        let proto = args.get(2).cloned();

                        remotes.push(Remote { host, port, proto });
                    }
                    "dev" => {
                        // dev <DEVICE>

                        if args.len() != 1 {
                            return Err(OvpnParseError::InvalidArgument {
                                key: key.clone(),
                                arg: format!("{args:?}"),
                                line: 0,
                            })?;
                        }

                        let value = args.get(0).ok_or(OvpnParseError::InvalidArgument {
                            key,
                            arg: "".into(),
                            line: 0,
                        })?;

                        dev = Some(value.clone());
                    }
                    "proto" => {
                        // proto <PROTOCOL>

                        let value = args.get(0).ok_or(OvpnParseError::InvalidArgument {
                            key,
                            arg: "".into(),
                            line: 0,
                        })?;

                        proto = Some(value.clone());
                    }
                    "cipher" => {
                        // cipher <CIPHER>
                        // Note: This is deprecated in new configs

                        let value = args.get(0).ok_or(OvpnParseError::InvalidArgument {
                            key,
                            arg: "".into(),
                            line: 0,
                        })?;

                        cipher = Some(value.clone());
                    }
                    "data-ciphers" => {
                        // data-ciphers <[cipher1]:[cipher2]...>

                        let ciphers = args.get(0).ok_or(OvpnParseError::InvalidArgument {
                            key,
                            arg: "".into(),
                            line: 0,
                        })?;

                        data_ciphers.extend(ciphers.split(':').map(String::from));
                    }
                    "auth" => {
                        // auth <ALGORITHM>

                        let value = args.get(0).ok_or(OvpnParseError::InvalidArgument {
                            key,
                            arg: "".into(),
                            line: 0,
                        })?;

                        auth = Some(value.clone());
                    }
                    "compress" => {
                        // compress <ALGORITHM>

                        let value = args.get(0).ok_or(OvpnParseError::InvalidArgument {
                            key,
                            arg: "".into(),
                            line: 0,
                        })?;

                        compress = Some(value.clone());
                    }
                    "allow-compress" => {
                        // allow-compress asym (default) <- receive compressed data but don't send
                        // allow-compress [yes/no]

                        let value = args.get(0).ok_or(OvpnParseError::InvalidArgument {
                            key,
                            arg: "".into(),
                            line: 0,
                        })?;

                        let parsed = match value.as_str() {
                            "yes" => AllowCompress::Yes,
                            "no" => AllowCompress::No,
                            "asym" => AllowCompress::Asym,
                            other => AllowCompress::Other(other.to_string()),
                        };

                        allow_compress = Some(parsed);
                    }
                    "route" => {
                        // route <NETWORK> [NETMASK] [GATEWAY]

                        let network = parse_ipv4_arg(&key, args.get(0), 0)?;
                        let netmask = args
                            .get(1)
                            .map(|v| parse_ipv4_arg(&key, Some(v), 0))
                            .transpose()?;
                        let gateway = args
                            .get(2)
                            .map(|v| parse_ipv4_arg(&key, Some(v), 0))
                            .transpose()?;

                        routes.push(Route {
                            network,
                            netmask,
                            gateway,
                        });
                    }
                    "redirect-gateway" => {
                        let mut rg = RedirectGateway {
                            def1: false,
                            bypass_dhcp: false,
                            bypass_dns: false,
                            local: false,
                            ipv6: false,
                        };

                        for arg in args {
                            match arg.as_str() {
                                "def1" => rg.def1 = true,
                                "bypass-dhcp" => rg.bypass_dhcp = true,
                                "bypass-dns" => rg.bypass_dns = true,
                                "local" => rg.local = true,
                                "ipv6" => rg.ipv6 = true,
                                _ => {}
                            }
                        }

                        redirect_gateway = Some(rg);
                    }
                    _ => {
                        if args.is_empty() {
                            flags.push(key);
                        } else {
                            options.entry(key).or_default().extend(args);
                        }
                    }
                }
            }
            OvpnItem::Block {
                key: block_key,
                content,
            } => {
                match block_key.as_str() {
                    "ca" => {
                        ca = Some(CertSource::Inline(content));
                    }

                    "cert" => {
                        cert = Some(CertSource::Inline(content));
                    }

                    "key" => {
                        key = Some(CertSource::Inline(content));
                    }

                    "tls-auth" => {
                        tls_auth = Some(TlsAuth {
                            source: CertSource::Inline(content),
                            key_direction: None, // FIXME: handle seperately
                        });
                    }

                    "tls-crypt" => {
                        tls_crypt = Some(CertSource::Inline(content));
                    }

                    _ => {
                        options.entry(block_key).or_default().push(content);
                    }
                }
            }
        }
    }

    Ok(OvpnFile {
        remotes,
        dev,
        proto,
        ca,
        cert,
        key,
        tls_auth,
        tls_crypt,
        cipher,
        data_ciphers,
        auth,
        compress,
        allow_compress,
        routes,
        redirect_gateway,
        flags,
        options,
    })
}

fn parse_ipv4_arg(
    key: &str,
    value: Option<&String>,
    line: usize,
) -> Result<Ipv4Addr, OvpnParseError> {
    let v = value.ok_or(OvpnParseError::InvalidArgument {
        key: key.to_string(),
        arg: "".into(),
        line,
    })?;

    v.parse::<Ipv4Addr>()
        .map_err(|_| OvpnParseError::InvalidNumber {
            key: key.to_string(),
            value: v.clone(),
            line,
        })
}
