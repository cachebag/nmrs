# VPN Management

Once you've set up a WireGuard or OpenVPN connection, nmrs provides methods to list, inspect, connect, disconnect, and remove VPN profiles.

## Listing VPN Connections

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let vpns = nm.list_vpn_connections().await?;
    for vpn in &vpns {
        println!("{}: {:?} [{:?}] (active: {})",
            vpn.name,
            vpn.vpn_type,
            vpn.state,
            vpn.active,
        );
        if let Some(iface) = &vpn.interface {
            println!("  Interface: {iface}");
        }
    }

    Ok(())
}
```

`list_vpn_connections()` returns all saved VPN profiles with their current state. The `VpnConnection` struct contains:

| Field | Type | Description |
|-------|------|-------------|
| `uuid` | `String` | Connection UUID |
| `id` | `String` | Connection name (alias for `name`) |
| `name` | `String` | Connection profile name |
| `vpn_type` | `VpnType` | VPN protocol — a data-carrying enum with `WireGuard`, `OpenVpn`, and other variants |
| `state` | `DeviceState` | Current state (`Activated`, `Disconnected`, etc.) |
| `interface` | `Option<String>` | Network interface when active (e.g., `wg0`, `tun0`) |
| `active` | `bool` | Whether the connection is currently active |
| `kind` | `VpnKind` | `VpnKind::Plugin` (OpenVPN) or `VpnKind::WireGuard` |

## Active VPN Connections

Get only currently active VPN connections:

```rust
let active = nm.active_vpn_connections().await?;

for vpn in &active {
    println!("Active: {} ({:?}) on {:?}", vpn.name, vpn.vpn_type, vpn.interface);
}
```

## Getting VPN Details

For an active VPN connection, retrieve detailed information including IP configuration:

```rust
let nm = NetworkManager::new().await?;

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

The `VpnConnectionInfo` struct provides:

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Connection name |
| `vpn_kind` | `VpnKind` | `VpnKind::Plugin` or `VpnKind::WireGuard` |
| `state` | `DeviceState` | Current state |
| `interface` | `Option<String>` | Interface name |
| `gateway` | `Option<String>` | VPN gateway address |
| `ip4_address` | `Option<String>` | Assigned IPv4 address |
| `ip6_address` | `Option<String>` | Assigned IPv6 address |
| `dns_servers` | `Vec<String>` | Active DNS servers |
| `details` | `Option<VpnDetails>` | Additional VPN-specific details |

> **Note:** `get_vpn_info()` returns `ConnectionError::NoVpnConnection` if the VPN is not currently active.

## Connecting to a Saved VPN

Reconnect to an existing VPN profile by name or UUID without rebuilding the config:

```rust
let nm = NetworkManager::new().await?;

// By profile name
nm.connect_vpn_by_id("MyVPN").await?;

// By UUID
nm.connect_vpn_by_uuid("a1b2c3d4-e5f6-7890-abcd-ef1234567890").await?;
```

## Disconnecting a VPN

```rust
let nm = NetworkManager::new().await?;

// By name
nm.disconnect_vpn("MyVPN").await?;

// By UUID
nm.disconnect_vpn_by_uuid("a1b2c3d4-e5f6-7890-abcd-ef1234567890").await?;

println!("VPN disconnected");
```

`disconnect_vpn()` deactivates the VPN connection by name. If the VPN is not currently active or doesn't exist, it returns `Ok(())` — the operation is idempotent.

## Removing a VPN Profile

To permanently delete a saved VPN connection:

```rust
let nm = NetworkManager::new().await?;

nm.forget_vpn("MyVPN").await?;
println!("VPN profile deleted");
```

`forget_vpn()` searches for a saved VPN profile with the given name. If the VPN is currently connected, it disconnects first, then deletes the profile. Returns `Ok(())` if no matching profile is found.

## Complete Example

```rust
use nmrs::{NetworkManager, WireGuardConfig, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // List existing VPNs
    println!("Saved VPN connections:");
    for vpn in nm.list_vpn_connections().await? {
        println!("  {} ({:?}) - {:?} [active: {}]",
            vpn.name, vpn.vpn_type, vpn.state, vpn.active);
    }

    // Connect a new WireGuard VPN
    let peer = WireGuardPeer::new(
        "SERVER_PUBLIC_KEY",
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".into()],
    ).with_persistent_keepalive(25);

    let config = WireGuardConfig::new(
        "ExampleVPN",
        "vpn.example.com:51820",
        "CLIENT_PRIVATE_KEY",
        "10.0.0.2/24",
        vec![peer],
    ).with_dns(vec!["1.1.1.1".into()]);

    nm.connect_vpn(config).await?;

    // Show details
    let info = nm.get_vpn_info("ExampleVPN").await?;
    println!("\nConnected: IP = {:?}", info.ip4_address);

    // Disconnect
    nm.disconnect_vpn("ExampleVPN").await?;
    println!("Disconnected");

    // Clean up
    nm.forget_vpn("ExampleVPN").await?;
    println!("Profile removed");

    Ok(())
}
```

## Error Handling

| Error | Method | Meaning |
|-------|--------|---------|
| `NoVpnConnection` | `get_vpn_info` | VPN not active |
| `VpnFailed` | `connect_vpn` | Connection activation failed |
| `InvalidPrivateKey` | `connect_vpn` | Bad WireGuard key |
| `InvalidAddress` | `connect_vpn` | Bad IP/CIDR |
| `InvalidGateway` | `connect_vpn` | Bad endpoint format |
| `AuthFailed` | `connect_vpn` | OpenVPN authentication failed |
| `InvalidConfig` | `connect_vpn` | OpenVPN configuration error (missing certs, bad options) |

## Next Steps

- [WireGuard Setup](./vpn-wireguard.md) — credential configuration details
- [OpenVPN Setup](./vpn-openvpn.md) — OpenVPN configuration details
- [Error Handling](./error-handling.md) — comprehensive error reference
- [Real-Time Monitoring](./monitoring.md) — monitor VPN state changes
