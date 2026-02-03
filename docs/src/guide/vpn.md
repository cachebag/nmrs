# VPN Connections

nmrs provides full support for WireGuard VPN connections through NetworkManager. This guide covers everything you need to know about managing VPNs with nmrs.

## Overview

VPN support includes:
- **WireGuard** - Modern, fast, secure VPN protocol
- **Profile Management** - Save and reuse VPN configurations
- **Connection Control** - Connect, disconnect, monitor VPN status
- **Multiple Peers** - Support for multiple WireGuard peers
- **Custom DNS** - Override DNS servers for VPN connections
- **MTU Configuration** - Optimize packet sizes

## Quick Start

Basic WireGuard VPN connection:

```rust
use nmrs::{NetworkManager, VpnCredentials, VpnType, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // Create a WireGuard peer
    let peer = WireGuardPeer::new(
        "server_public_key_here",
        "vpn.example.com:51820",
        vec!["0.0.0.0/0".into()],  // Route all traffic through VPN
    ).with_persistent_keepalive(25);
    
    // Create VPN credentials
    let creds = VpnCredentials::new(
        VpnType::WireGuard,
        "MyVPN",
        "vpn.example.com:51820",
        "your_private_key_here",
        "10.0.0.2/24",  // Your VPN IP
        vec![peer],
    ).with_dns(vec!["1.1.1.1".into()]);
    
    // Connect
    nm.connect_vpn(creds).await?;
    println!("Connected to VPN!");
    
    Ok(())
}
```

## VPN Credentials

The `VpnCredentials` struct contains all necessary VPN configuration:

```rust
pub struct VpnCredentials {
    pub vpn_type: VpnType,
    pub name: String,
    pub gateway: String,
    pub private_key: String,
    pub address: String,
    pub peers: Vec<WireGuardPeer>,
    pub dns: Option<Vec<String>>,
    pub mtu: Option<u32>,
    pub uuid: Option<String>,
}
```

### Creating Credentials

```rust
use nmrs::{VpnCredentials, VpnType, WireGuardPeer};

let peer = WireGuardPeer::new(
    "base64_public_key",
    "vpn.example.com:51820",
    vec!["0.0.0.0/0".into()],
);

let creds = VpnCredentials::new(
    VpnType::WireGuard,
    "WorkVPN",                    // Connection name
    "vpn.example.com:51820",      // Gateway
    "base64_private_key",         // Your private key
    "10.0.0.2/24",               // Your VPN IP address
    vec![peer],                   // WireGuard peers
);
```

### With Custom DNS

```rust
let creds = VpnCredentials::new(
    VpnType::WireGuard,
    "MyVPN",
    "vpn.example.com:51820",
    "private_key",
    "10.0.0.2/24",
    vec![peer],
).with_dns(vec![
    "1.1.1.1".into(),
    "8.8.8.8".into(),
]);
```

### With Custom MTU

```rust
let creds = creds.with_mtu(1420);  // Standard WireGuard MTU
```

## WireGuard Peers

Each WireGuard connection can have multiple peers:

```rust
pub struct WireGuardPeer {
    pub public_key: String,
    pub gateway: String,
    pub allowed_ips: Vec<String>,
    pub preshared_key: Option<String>,
    pub persistent_keepalive: Option<u32>,
}
```

### Creating Peers

```rust
use nmrs::WireGuardPeer;

// Basic peer
let peer = WireGuardPeer::new(
    "peer_public_key",
    "vpn.example.com:51820",
    vec!["0.0.0.0/0".into()],
);

// Peer with keepalive
let peer = WireGuardPeer::new(
    "peer_public_key",
    "vpn.example.com:51820",
    vec!["0.0.0.0/0".into()],
).with_persistent_keepalive(25);

// Peer with preshared key
let peer = peer.with_preshared_key("base64_preshared_key");
```

### Multiple Peers

```rust
let peer1 = WireGuardPeer::new(
    "peer1_public_key",
    "vpn1.example.com:51820",
    vec!["10.0.0.0/8".into()],
);

let peer2 = WireGuardPeer::new(
    "peer2_public_key",
    "vpn2.example.com:51820",
    vec!["192.168.0.0/16".into()],
);

let creds = VpnCredentials::new(
    VpnType::WireGuard,
    "MultiPeerVPN",
    "vpn1.example.com:51820",
    "private_key",
    "10.0.0.2/24",
    vec![peer1, peer2],  // Multiple peers
);
```

## VPN Operations

### Connect to VPN

```rust
nm.connect_vpn(creds).await?;
```

### Disconnect from VPN

```rust
nm.disconnect_vpn("MyVPN").await?;
```

### List VPN Connections

```rust
let vpns = nm.list_vpn_connections().await?;

for vpn in vpns {
    println!("Name: {}", vpn.name);
    println!("Type: {:?}", vpn.vpn_type);
    println!("State: {:?}", vpn.state);
}
```

### Get VPN Information

```rust
let info = nm.get_vpn_info("MyVPN").await?;

println!("VPN State: {:?}", info.state);
if let Some(ip) = info.ip4_address {
    println!("VPN IP: {}", ip);
}
if let Some(device) = info.device {
    println!("Device: {}", device);
}
```

### Check if VPN is Active

```rust
let vpns = nm.list_vpn_connections().await?;
let active = vpns.iter().any(|v| {
    matches!(v.state, nmrs::models::ActiveConnectionState::Activated)
});

if active {
    println!("VPN is active");
} else {
    println!("VPN is not active");
}
```

## Routing Configuration

### Route All Traffic

Send all traffic through the VPN:

```rust
let peer = WireGuardPeer::new(
    "public_key",
    "vpn.example.com:51820",
    vec!["0.0.0.0/0".into()],  // All IPv4
);
```

### Split Tunnel

Route only specific networks through VPN:

```rust
let peer = WireGuardPeer::new(
    "public_key",
    "vpn.example.com:51820",
    vec![
        "10.0.0.0/8".into(),      // Private network
        "192.168.0.0/16".into(),   // Another private network
    ],
);
```

### IPv6 Support

```rust
let peer = WireGuardPeer::new(
    "public_key",
    "vpn.example.com:51820",
    vec![
        "0.0.0.0/0".into(),        // All IPv4
        "::/0".into(),             // All IPv6
    ],
);
```

## Error Handling

Handle VPN-specific errors:

```rust
use nmrs::ConnectionError;

match nm.connect_vpn(creds).await {
    Ok(_) => println!("VPN connected"),
    
    Err(ConnectionError::AuthFailed) => {
        eprintln!("VPN authentication failed - check keys");
    }
    
    Err(ConnectionError::Timeout) => {
        eprintln!("VPN connection timed out - check gateway");
    }
    
    Err(ConnectionError::NotFound) => {
        eprintln!("VPN gateway not reachable");
    }
    
    Err(e) => eprintln!("VPN error: {}", e),
}
```

## Complete Example

Here's a complete VPN client:

```rust
use nmrs::{NetworkManager, VpnCredentials, VpnType, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // Check if already connected
    let vpns = nm.list_vpn_connections().await?;
    if let Some(active_vpn) = vpns.iter().find(|v| {
        matches!(v.state, nmrs::models::ActiveConnectionState::Activated)
    }) {
        println!("Already connected to: {}", active_vpn.name);
        return Ok(());
    }
    
    // Create WireGuard configuration
    let peer = WireGuardPeer::new(
        std::env::var("WG_PUBLIC_KEY")?,
        std::env::var("WG_ENDPOINT")?,
        vec!["0.0.0.0/0".into()],
    ).with_persistent_keepalive(25);
    
    let creds = VpnCredentials::new(
        VpnType::WireGuard,
        "AutoVPN",
        std::env::var("WG_ENDPOINT")?,
        std::env::var("WG_PRIVATE_KEY")?,
        std::env::var("WG_ADDRESS")?,
        vec![peer],
    ).with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()]);
    
    // Connect
    println!("Connecting to VPN...");
    nm.connect_vpn(creds).await?;
    
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

- [WireGuard Setup](./vpn-wireguard.md) - Detailed WireGuard guide
- [VPN Management](./vpn-management.md) - Managing VPN profiles
- [Examples](../examples/wireguard-client.md) - Complete VPN client example

## Security Best Practices

1. **Never hardcode keys** - Use environment variables or secure storage
2. **Rotate keys regularly** - Update WireGuard keys periodically
3. **Use preshared keys** - Add extra layer of security with PSK
4. **Verify endpoints** - Ensure gateway addresses are correct
5. **Monitor connection** - Check VPN status regularly

## Troubleshooting

### VPN Won't Connect

```rust
// Check if WireGuard is available
// NetworkManager should handle this automatically

// Verify your credentials are correct
println!("Gateway: {}", creds.gateway);
println!("Address: {}", creds.address);
// Don't print private keys!
```

### Connection Drops

Use persistent keepalive:

```rust
let peer = peer.with_persistent_keepalive(25);  // Send keepalive every 25s
```

### DNS Not Working

Explicitly set DNS servers:

```rust
let creds = creds.with_dns(vec![
    "1.1.1.1".into(),
    "8.8.8.8".into(),
]);
```

## Next Steps

- [WireGuard Setup Guide](./vpn-wireguard.md)
- [VPN Management](./vpn-management.md)
- [Complete VPN Client Example](../examples/wireguard-client.md)
