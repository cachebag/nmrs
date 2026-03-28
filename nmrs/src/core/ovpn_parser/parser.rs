use std::collections::HashMap;

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
    todo!()
}
