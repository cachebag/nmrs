# OpenVPN Setup

[OpenVPN](https://openvpn.net/) is a widely-deployed, battle-tested VPN protocol that uses TLS for key exchange and supports a variety of authentication methods. nmrs provides full OpenVPN support through the NetworkManager OpenVPN plugin, letting you create, import, and manage OpenVPN connections programmatically.

## Prerequisites

- NetworkManager 1.2+
- The OpenVPN plugin for NetworkManager:
  - Fedora / RHEL: `NetworkManager-openvpn`
  - Debian / Ubuntu: `network-manager-openvpn`
  - Arch Linux: `networkmanager-openvpn`
- OpenVPN certificates and/or credentials from your VPN provider

## Quick Start

Connect to an OpenVPN server using password + TLS certificate authentication:

```rust
use nmrs::{NetworkManager, OpenVpnConfig, OpenVpnAuthType};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let config = OpenVpnConfig::new("CorpVPN", "vpn.example.com", 1194, false)
        .with_auth_type(OpenVpnAuthType::PasswordTls)
        .with_ca_cert("/etc/openvpn/ca.crt")
        .with_client_cert("/etc/openvpn/client.crt")
        .with_client_key("/etc/openvpn/client.key")
        .with_username("alice")
        .with_password("hunter2")
        .with_dns(vec!["1.1.1.1".into()]);

    nm.connect_vpn(config).await?;

    println!("VPN connected!");
    Ok(())
}
```

## Authentication Types

OpenVPN supports four authentication modes, selected with `OpenVpnAuthType`:

| Variant | Description | Required Fields |
|---------|-------------|-----------------|
| `Password` | Username/password only | `username` |
| `Tls` | TLS certificate only | `ca_cert`, `client_cert`, `client_key` |
| `PasswordTls` | Password + TLS certificates | `username`, `ca_cert`, `client_cert`, `client_key` |
| `StaticKey` | Pre-shared static key | (static key file via TLS auth) |

### Password Authentication

```rust
use nmrs::{OpenVpnConfig, OpenVpnAuthType};

let config = OpenVpnConfig::new("SimpleVPN", "vpn.example.com", 1194, false)
    .with_auth_type(OpenVpnAuthType::Password)
    .with_username("alice")
    .with_password("secret")
    .with_ca_cert("/etc/openvpn/ca.crt");
```

### TLS Certificate Authentication

```rust
use nmrs::{OpenVpnConfig, OpenVpnAuthType};

let config = OpenVpnConfig::new("CertVPN", "vpn.example.com", 1194, false)
    .with_auth_type(OpenVpnAuthType::Tls)
    .with_ca_cert("/etc/openvpn/ca.crt")
    .with_client_cert("/etc/openvpn/client.crt")
    .with_client_key("/etc/openvpn/client.key");
```

If the client key is encrypted:

```rust
let config = config.with_key_password("keyfile-passphrase");
```

### Password + TLS Authentication

The most common configuration for corporate VPNs — the server verifies both your certificate and your credentials:

```rust
use nmrs::{OpenVpnConfig, OpenVpnAuthType};

let config = OpenVpnConfig::new("CorpVPN", "vpn.example.com", 443, true)
    .with_auth_type(OpenVpnAuthType::PasswordTls)
    .with_ca_cert("/etc/openvpn/ca.crt")
    .with_client_cert("/etc/openvpn/client.crt")
    .with_client_key("/etc/openvpn/client.key")
    .with_username("alice")
    .with_password("secret");
```

## Configuration Reference

| Field | Builder Method | Required | Description |
|-------|---------------|----------|-------------|
| `name` | constructor | Yes | Connection profile name |
| `remote` | constructor | Yes | Server hostname or IP |
| `port` | constructor | Yes | Server port (typically 1194 or 443) |
| `tcp` | constructor | Yes | `true` for TCP, `false` for UDP |
| `auth_type` | `with_auth_type` | No | Authentication mode (see above) |
| `ca_cert` | `with_ca_cert` | No* | Path to CA certificate |
| `client_cert` | `with_client_cert` | No* | Path to client certificate |
| `client_key` | `with_client_key` | No* | Path to client private key |
| `key_password` | `with_key_password` | No | Password for encrypted key file |
| `username` | `with_username` | No* | Username for password auth |
| `password` | `with_password` | No | Password for password auth |
| `cipher` | `with_cipher` | No | Data channel cipher (e.g. `"AES-256-GCM"`) |
| `auth` | `with_auth` | No | HMAC digest algorithm (e.g. `"SHA256"`) |
| `dns` | `with_dns` | No | DNS servers while connected |
| `mtu` | `with_mtu` | No | MTU size |
| `uuid` | `with_uuid` | No | Custom UUID (auto-generated if omitted) |
| `compression` | `with_compression` | No | Compression mode (see below) |
| `proxy` | `with_proxy` | No | Proxy configuration |
| `redirect_gateway` | `with_redirect_gateway` | No | Full tunnel (`false` by default) |
| `routes` | `with_routes` | No | Split tunnel routes |

\* Required depending on the chosen `auth_type`.

## Importing .ovpn Files

If you already have an `.ovpn` configuration file, you can import it directly.

### High-Level Import

The simplest approach — import and connect in one call:

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    nm.import_ovpn("corp.ovpn", Some("alice"), Some("secret")).await?;

    println!("Connected via imported .ovpn profile");
    Ok(())
}
```

`import_ovpn` parses the file, creates a NetworkManager connection profile, and activates it. The connection name defaults to the filename stem (e.g., `corp` from `corp.ovpn`).

### Builder-Based Import

For more control, use `OpenVpnBuilder::from_ovpn_file` to parse the file into a builder, then customise before connecting:

```rust
use nmrs::builders::OpenVpnBuilder;
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let config = OpenVpnBuilder::from_ovpn_file("corp.ovpn")?
        .username("alice")
        .dns(vec!["1.1.1.1".into()])
        .mtu(1400)
        .remote_cert_tls("server")
        .build()?;

    nm.connect_vpn(config).await?;

    Ok(())
}
```

You can also parse from a string with `OpenVpnBuilder::from_ovpn_str(content, name)` if the configuration is fetched from a remote source.

## TLS Hardening

OpenVPN's TLS layer can be hardened with several options. These are independent of the authentication type and can be combined.

### TLS Auth

Adds an HMAC firewall to the control channel, providing DoS protection. Both sides must share the same static key and agree on direction:

```rust
let config = config
    .with_tls_auth("/etc/openvpn/ta.key", Some(1));
```

### TLS-Crypt

Encrypts **and** authenticates the entire control channel with a pre-shared key — stronger than `tls-auth` because the TLS handshake itself is hidden:

```rust
let config = config
    .with_tls_crypt("/etc/openvpn/tls-crypt.key");
```

### TLS-Crypt-v2

Per-client key wrapping, allowing the server to issue unique keys to each client while retaining the benefits of TLS-Crypt:

```rust
let config = config
    .with_tls_crypt_v2("/etc/openvpn/client-tls-crypt-v2.key");
```

> **Note:** `tls-auth`, `tls-crypt`, and `tls-crypt-v2` are mutually exclusive. Use only one.

### Certificate Verification

Verify the server's certificate identity to prevent man-in-the-middle attacks:

```rust
use nmrs::OpenVpnConfig;

let config = OpenVpnConfig::new("SecureVPN", "vpn.example.com", 1194, false)
    .with_auth_type(nmrs::OpenVpnAuthType::Tls)
    .with_ca_cert("/etc/openvpn/ca.crt")
    .with_client_cert("/etc/openvpn/client.crt")
    .with_client_key("/etc/openvpn/client.key")
    .with_remote_cert_tls("server")
    .with_verify_x509_name("vpn.example.com", "name");
```

| Method | Purpose |
|--------|---------|
| `with_remote_cert_tls("server")` | Require the remote cert to have server (TLS Web Server) usage |
| `with_verify_x509_name(name, type)` | Verify the CN or subject of the server certificate |
| `with_tls_version_min("1.2")` | Enforce minimum TLS version |
| `with_tls_version_max("1.3")` | Cap the maximum TLS version |
| `with_tls_cipher(suite)` | Restrict control-channel cipher suites |

## Split Tunneling

By default, `redirect_gateway` is `false` — only traffic matching explicit routes goes through the VPN.

### Full Tunnel

Route all traffic through the VPN:

```rust
let config = config.with_redirect_gateway(true);
```

### Split Tunnel with Routes

Route only specific networks through the VPN using `VpnRoute`:

```rust
use nmrs::{OpenVpnConfig, OpenVpnAuthType, VpnRoute};

let config = OpenVpnConfig::new("OfficeVPN", "vpn.example.com", 1194, false)
    .with_auth_type(OpenVpnAuthType::Tls)
    .with_ca_cert("/etc/openvpn/ca.crt")
    .with_client_cert("/etc/openvpn/client.crt")
    .with_client_key("/etc/openvpn/client.key")
    .with_routes(vec![
        VpnRoute::new("10.0.0.0", 8),
        VpnRoute::new("192.168.1.0", 24).next_hop("10.0.0.1").metric(100),
    ]);
```

| `VpnRoute` Method | Description |
|--------------------|-------------|
| `VpnRoute::new(dest, prefix)` | Destination network and CIDR prefix length |
| `.next_hop(gateway)` | Optional gateway for the route |
| `.metric(m)` | Optional route metric (lower = higher priority) |

## Compression

OpenVPN supports several compression algorithms, but **compression is disabled by default for security reasons**.

```rust
use nmrs::OpenVpnCompression;

let config = config.with_compression(OpenVpnCompression::No);
```

| Variant | Description |
|---------|-------------|
| `No` | Disabled (recommended default) |
| `Lzo` | LZO compression (deprecated) |
| `Lz4` | LZ4 compression |
| `Lz4V2` | LZ4 v2 compression |
| `Yes` | Adaptive compression |

> **Security Warning:** Enabling compression on an OpenVPN tunnel that carries TLS traffic (HTTPS, etc.) exposes the connection to the [VORACLE attack](https://openvpn.net/security-advisory/the-voracle-attack-vulnerability/). An attacker who can observe encrypted VPN traffic and induce the victim to visit attacker-controlled content can recover plaintext via compression oracle side-channels. **Leave compression disabled unless you have a specific need and understand the risk.**

## Proxy Support

Route OpenVPN traffic through an HTTP or SOCKS proxy:

### HTTP Proxy

```rust
use nmrs::OpenVpnProxy;

let config = config.with_proxy(OpenVpnProxy::Http {
    server: "proxy.example.com".into(),
    port: 8080,
    username: Some("proxyuser".into()),
    password: Some("proxypass".into()),
    retry: true,
});
```

### SOCKS Proxy

```rust
use nmrs::OpenVpnProxy;

let config = config.with_proxy(OpenVpnProxy::Socks {
    server: "socks.example.com".into(),
    port: 1080,
    retry: false,
});
```

When using a proxy, TCP mode (`tcp: true` in the constructor) is typically required.

## Error Handling

Handle OpenVPN-specific errors:

```rust
use nmrs::{ConnectionError, NetworkManager, OpenVpnConfig, OpenVpnAuthType};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let config = OpenVpnConfig::new("CorpVPN", "vpn.example.com", 1194, false)
        .with_auth_type(OpenVpnAuthType::PasswordTls)
        .with_ca_cert("/etc/openvpn/ca.crt")
        .with_client_cert("/etc/openvpn/client.crt")
        .with_client_key("/etc/openvpn/client.key")
        .with_username("alice")
        .with_password("secret");

    match nm.connect_vpn(config).await {
        Ok(()) => println!("VPN connected"),

        Err(ConnectionError::VpnFailed(msg)) => {
            eprintln!("OpenVPN activation failed: {msg}");
        }

        Err(ConnectionError::AuthFailed) => {
            eprintln!("Authentication failed — check username/password and certificates");
        }

        Err(ConnectionError::Timeout) => {
            eprintln!("Connection timed out — verify server address and port");
        }

        Err(ConnectionError::InvalidGateway(msg)) => {
            eprintln!("Bad server address: {msg}");
        }

        Err(e) => eprintln!("Unexpected error: {e}"),
    }

    Ok(())
}
```

| Error | Cause |
|-------|-------|
| `VpnFailed` | Plugin missing, config rejected, or activation failed |
| `AuthFailed` | Bad username/password or certificate rejected |
| `Timeout` | Server unreachable or handshake timed out |
| `InvalidGateway` | Empty or invalid remote address |

## Next Steps

- [VPN Connections](./vpn.md) – VPN overview and general operations
- [VPN Management](./vpn-management.md) – list, disconnect, and remove VPN profiles
- [Error Handling](./error-handling.md) – comprehensive error reference
