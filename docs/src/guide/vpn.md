# VPN Connections

nmrs provides full support for WireGuard and OpenVPN connections through NetworkManager. This guide covers everything you need to know about managing VPNs with nmrs.

## Overview

VPN support includes:
- **WireGuard** — Modern, fast, secure VPN protocol (native NetworkManager integration)
- **OpenVPN** — Widely deployed VPN protocol (via NetworkManager OpenVPN plugin)
- **`.ovpn` Import** — Import existing OpenVPN configuration files
- **Profile Management** — Save and reuse VPN configurations
- **Connection Control** — Connect, disconnect, monitor VPN status
- **Multiple Peers** — Support for multiple WireGuard peers
- **Custom DNS** — Override DNS servers for VPN connections
- **MTU Configuration** — Optimize packet sizes

## WireGuard Quick Start

```rust
use nmrs::{NetworkManager, WireGuardConfig, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let peer = WireGuardPeer::new(
        "server_public_key",
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".into()],
    ).with_persistent_keepalive(25);

    let config = WireGuardConfig::new(
        "MyVPN",
        "vpn.example.com:51820",
        "your_private_key",
        "10.0.0.2/24",
        vec![peer],
    ).with_dns(vec!["1.1.1.1".into()]);

    nm.connect_vpn(config).await?;
    println!("Connected to WireGuard VPN!");

    Ok(())
}
```

## OpenVPN Quick Start

```rust
use nmrs::{NetworkManager, OpenVpnConfig, OpenVpnAuthType};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let config = OpenVpnConfig::new("CorpVPN", "vpn.example.com", 1194, false)
        .with_auth_type(OpenVpnAuthType::PasswordTls)
        .with_username("user")
        .with_password("secret")
        .with_ca_cert("/etc/openvpn/ca.crt")
        .with_client_cert("/etc/openvpn/client.crt")
        .with_client_key("/etc/openvpn/client.key");

    nm.connect_vpn(config).await?;
    println!("Connected to OpenVPN!");

    Ok(())
}
```

## `.ovpn` File Import

Import an existing OpenVPN configuration file directly:

```rust
nm.import_ovpn("client.ovpn", Some("user"), Some("secret")).await?;
```

For certificate-only configs that don't require credentials:

```rust
nm.import_ovpn("client.ovpn", None, None).await?;
```

See the [`.ovpn` Import Example](../examples/ovpn-import.md) for builder-based import and inline certificate handling.

## VPN Operations

### Connect

```rust
// WireGuard
nm.connect_vpn(wireguard_config).await?;

// OpenVPN
nm.connect_vpn(openvpn_config).await?;
```

### Connect by Name or UUID

Reconnect to a saved VPN profile without rebuilding the config:

```rust
nm.connect_vpn_by_id("MyVPN").await?;
nm.connect_vpn_by_uuid("a1b2c3d4-e5f6-...").await?;
```

### Disconnect

```rust
nm.disconnect_vpn("MyVPN").await?;
nm.disconnect_vpn_by_uuid("a1b2c3d4-e5f6-...").await?;
```

### List VPN Connections

```rust
let vpns = nm.list_vpn_connections().await?;

for vpn in &vpns {
    println!("{} ({:?}) — active: {}", vpn.name, vpn.vpn_type, vpn.active);
    if let Some(iface) = &vpn.interface {
        println!("  Interface: {iface}");
    }
}
```

`list_vpn_connections()` returns `Vec<VpnConnection>` with fields:

| Field | Type | Description |
|-------|------|-------------|
| `uuid` | `String` | Connection UUID |
| `id` | `String` | Connection name (alias for `name`) |
| `name` | `String` | Connection profile name |
| `vpn_type` | `VpnType` | VPN protocol (`WireGuard`, `OpenVpn`, etc.) |
| `state` | `DeviceState` | Current state (`Activated`, `Disconnected`, etc.) |
| `interface` | `Option<String>` | Network interface when active |
| `active` | `bool` | Whether the connection is currently active |
| `kind` | `VpnKind` | `VpnKind::Plugin` (OpenVPN) or `VpnKind::WireGuard` |

### Active VPN Connections

Get only currently active VPN connections:

```rust
let active = nm.active_vpn_connections().await?;

for vpn in &active {
    println!("Active: {} ({:?})", vpn.name, vpn.vpn_type);
}
```

### Get VPN Information

```rust
let info = nm.get_vpn_info("MyVPN").await?;

println!("Name:      {}", info.name);
println!("Kind:      {:?}", info.vpn_kind);
println!("State:     {:?}", info.state);
println!("Interface: {:?}", info.interface);
println!("Gateway:   {:?}", info.gateway);
println!("IPv4:      {:?}", info.ip4_address);
println!("IPv6:      {:?}", info.ip6_address);
println!("DNS:       {:?}", info.dns_servers);

if let Some(details) = &info.details {
    println!("Details:   {:?}", details);
}
```

### Remove a VPN Profile

```rust
nm.forget_vpn("MyVPN").await?;
```

## Routing Configuration

### Route All Traffic (WireGuard)

```rust
let peer = WireGuardPeer::new(
    "public_key",
    "vpn.example.com:51820",
    vec!["0.0.0.0/0".into()],
);
```

### Split Tunnel (WireGuard)

Route only specific networks through VPN:

```rust
let peer = WireGuardPeer::new(
    "public_key",
    "vpn.example.com:51820",
    vec![
        "10.0.0.0/8".into(),
        "192.168.0.0/16".into(),
    ],
);
```

### IPv6 Support

```rust
let peer = WireGuardPeer::new(
    "public_key",
    "vpn.example.com:51820",
    vec![
        "0.0.0.0/0".into(),
        "::/0".into(),
    ],
);
```

## Error Handling

```rust
use nmrs::ConnectionError;

match nm.connect_vpn(config).await {
    Ok(_) => println!("VPN connected"),

    Err(ConnectionError::AuthFailed) => {
        eprintln!("Authentication failed — check keys or credentials");
    }

    Err(ConnectionError::Timeout) => {
        eprintln!("Connection timed out — check gateway address");
    }

    Err(ConnectionError::VpnFailed) => {
        eprintln!("VPN activation failed — check plugin or config");
    }

    Err(ConnectionError::NotFound) => {
        eprintln!("VPN gateway not reachable");
    }

    Err(e) => eprintln!("VPN error: {e}"),
}
```

## Complete Example

```rust
use nmrs::{NetworkManager, WireGuardConfig, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Check if already connected
    let active = nm.active_vpn_connections().await?;
    if let Some(vpn) = active.first() {
        println!("Already connected to: {}", vpn.name);
        return Ok(());
    }

    // Create WireGuard configuration
    let peer = WireGuardPeer::new(
        std::env::var("WG_PUBLIC_KEY")?,
        std::env::var("WG_ENDPOINT")?,
        vec!["0.0.0.0/0".into()],
    ).with_persistent_keepalive(25);

    let config = WireGuardConfig::new(
        "AutoVPN",
        &std::env::var("WG_ENDPOINT")?,
        &std::env::var("WG_PRIVATE_KEY")?,
        &std::env::var("WG_ADDRESS")?,
        vec![peer],
    ).with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()]);

    // Connect
    println!("Connecting to VPN...");
    nm.connect_vpn(config).await?;

    // Verify connection
    let info = nm.get_vpn_info("AutoVPN").await?;
    println!("Connected! VPN IP: {:?}", info.ip4_address);

    // Keep connection alive
    println!("Press Ctrl+C to disconnect...");
    tokio::signal::ctrl_c().await?;

    // Disconnect
    nm.disconnect_vpn("AutoVPN").await?;
    println!("Disconnected from VPN");

    Ok(())
}
```

## Advanced Topics

For more advanced VPN usage, see:

- [WireGuard Setup](./vpn-wireguard.md) — Detailed WireGuard configuration
- [OpenVPN Setup](./vpn-openvpn.md) — Detailed OpenVPN configuration
- [VPN Management](./vpn-management.md) — Managing VPN profiles
- [WireGuard Client Example](../examples/wireguard-client.md) — Complete WireGuard example
- [OpenVPN Client Example](../examples/openvpn-client.md) — Complete OpenVPN example
- [`.ovpn` Import Example](../examples/ovpn-import.md) — Import `.ovpn` files

## Security Best Practices

1. **Never hardcode keys or passwords** — Use environment variables or secure storage
2. **Rotate keys regularly** — Update WireGuard keys periodically
3. **Use preshared keys** — Add extra layer of security with PSK (WireGuard)
4. **Protect certificates** — Store OpenVPN certs with restrictive file permissions (`chmod 600`)
5. **Use TLS authentication** — Prefer `PasswordTls` or `Tls` over `Password` alone for OpenVPN
6. **Verify endpoints** — Ensure gateway addresses are correct
7. **Monitor connection** — Check VPN status regularly

## Troubleshooting

### VPN Won't Connect

```rust
let vpns = nm.list_vpn_connections().await?;
for vpn in &vpns {
    println!("{}: {:?} (active: {})", vpn.name, vpn.state, vpn.active);
}
```

### Connection Drops (WireGuard)

Use persistent keepalive:

```rust
let peer = peer.with_persistent_keepalive(25);
```

### DNS Not Working

Explicitly set DNS servers:

```rust
let config = config.with_dns(vec![
    "1.1.1.1".into(),
    "8.8.8.8".into(),
]);
```

### OpenVPN Plugin Not Found

Ensure the NetworkManager OpenVPN plugin is installed:

```bash
# Arch Linux
sudo pacman -S networkmanager-openvpn

# Debian/Ubuntu
sudo apt install network-manager-openvpn

# Fedora
sudo dnf install NetworkManager-openvpn
```

## Next Steps

- [WireGuard Setup Guide](./vpn-wireguard.md)
- [OpenVPN Setup Guide](./vpn-openvpn.md)
- [VPN Management](./vpn-management.md)
- [OpenVPN Client Example](../examples/openvpn-client.md)
- [WireGuard Client Example](../examples/wireguard-client.md)
