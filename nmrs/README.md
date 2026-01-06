# nmrs

[![Crates.io](https://img.shields.io/crates/v/nmrs)](https://crates.io/crates/nmrs)
[![Documentation](https://docs.rs/nmrs/badge.svg)](https://docs.rs/nmrs)
[![License](https://img.shields.io/crates/l/nmrs)](LICENSE)

Rust bindings for NetworkManager via D-Bus.

## Why?

`nmrs` provides a high-level, async API for managing Wi-Fi connections on Linux systems. It abstracts the complexity of D-Bus communication with NetworkManager, offering typed error handling and an ergonomic interface.

## Features

- **WiFi Management**: Connect to WPA-PSK, WPA-EAP, and open networks
- **VPN Support**: WireGuard VPN connections with full configuration
- **Ethernet**: Wired network connection management
- **Network Discovery**: Scan and list available access points with signal strength
- **Profile Management**: Create, query, and delete saved connection profiles
- **Real-Time Monitoring**: Signal-based network and device state change notifications
- **Typed Errors**: Structured error types with specific failure reasons
- **Fully Async**: Built on `zbus` with async/await throughout

## Installation

```toml
[dependencies]
nmrs = "1.1.0"
```
or
```bash
caargo add nmrs
```

## Quick Start

Below are a few examples of the different ways to interface with things like WiFi (wireless) devices, WireGuard VPN configs, or EAP connections.

### WiFi Connection

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // List networks
    let networks = nm.list_networks().await?;
    for net in &networks {
        println!("{} - Signal: {}%", net.ssid, net.strength.unwrap_or(0));
    }
    
    // Connect to WPA-PSK network
    nm.connect("MyNetwork", WifiSecurity::WpaPsk {
        psk: "password".into()
    }).await?;
    
    // Check current connection
    if let Some(ssid) = nm.current_ssid().await {
        println!("Connected to: {}", ssid);
    }
    
    Ok(())
}
```

### WireGuard VPN

```rust
use nmrs::{NetworkManager, VpnCredentials, VpnType, WireGuardPeer};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    let creds = VpnCredentials {
        vpn_type: VpnType::WireGuard,
        name: "WorkVPN".into(),
        gateway: "vpn.example.com:51820".into(),
        private_key: "your_private_key_here".into(),
        address: "10.0.0.2/24".into(),
        peers: vec![WireGuardPeer {
            public_key: "server_public_key".into(),
            gateway: "vpn.example.com:51820".into(),
            allowed_ips: vec!["0.0.0.0/0".into()],
            preshared_key: None,
            persistent_keepalive: Some(25),
        }],
        dns: Some(vec!["1.1.1.1".into()]),
        mtu: None,
        uuid: None,
    };
    
    // Connect to VPN
    nm.connect_vpn(creds).await?;
    
    // Get connection details
    let info = nm.get_vpn_info("WorkVPN").await?;
    println!("VPN IP: {:?}", info.ip4_address);
    
    // Disconnect
    nm.disconnect_vpn("WorkVPN").await?;
    
    Ok(())
}
```

### WPA-Enterprise (EAP)

```rust
use nmrs::{NetworkManager, WifiSecurity, EapOptions, EapMethod, Phase2};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    nm.connect("CorpNetwork", WifiSecurity::WpaEap {
        opts: EapOptions {
            identity: "user@company.com".into(),
            password: "password".into(),
            anonymous_identity: None,
            domain_suffix_match: Some("company.com".into()),
            ca_cert_path: None,
            system_ca_certs: true,
            method: EapMethod::Peap,
            phase2: Phase2::Mschapv2,
        }
    }).await?;
    
    Ok(())
}
```

### Device Management

We also handle agnostic device management, as many of the devicees supported by NetworkManager can queried in similar ways.

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // List all network devices
    let devices = nm.list_devices().await?;
    for device in devices {
        println!("{}: {} ({})", device.interface, device.device_type, device.state);
    }
    
    // Control WiFi radio
    nm.set_wifi_enabled(false).await?;
    nm.set_wifi_enabled(true).await?;
    
    Ok(())
}
```

### Real-Time Monitoring

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // Monitor network changes
    nm.monitor_network_changes(|| {
        println!("Network list changed");
    }).await?;
    
    Ok(())
}
```

## Error Handling

All operations return `Result<T, ConnectionError>` with specific variants:

```rust
use nmrs::{NetworkManager, WifiSecurity, ConnectionError};

match nm.connect("MyNetwork", WifiSecurity::WpaPsk { 
    psk: "wrong".into() 
}).await {
    Ok(_) => println!("Connected"),
    Err(ConnectionError::AuthFailed) => eprintln!("Authentication failed"),
    Err(ConnectionError::NotFound) => eprintln!("Network not in range"),
    Err(ConnectionError::Timeout) => eprintln!("Connection timed out"),
    Err(ConnectionError::DhcpFailed) => eprintln!("Failed to obtain IP address"),
    Err(e) => eprintln!("Error: {}", e),
}
```
## Async Runtime Support

`nmrs` is **runtime-agnostic** and works with any async runtime:

- **Tokio** 
- **async-std**
- **smol**
- Any runtime supporting standard Rust `async/await`

All examples use Tokio, but you can use your preferred runtime:

**With Tokio:**
```rust
#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = nmrs::NetworkManager::new().await?;
    // ...
    Ok(())
}
```
**With async-std:**
```rust
#[async_std::main]
async fn main() -> nmrs::Result<()> {
    let nm = nmrs::NetworkManager::new().await?;
    // ...
    Ok(())
}
```

**With smol:**
```rust
fn main() -> nmrs::Result<()> {
    smol::block_on(async {
        let nm = nmrs::NetworkManager::new().await?;
        // ...
        Ok(())
    })
}
```

`nmrs` uses [`zbus`](https://github.com/z-galaxy/zbus) for D-Bus communication, which launches a background thread to handle D-Bus message processing. This design ensures compatibility across all async runtimes without requiring manual executor management.

## Documentation

Complete API documentation: [docs.rs/nmrs](https://docs.rs/nmrs)

## Requirements

- Linux with NetworkManager (1.0+)
- D-Bus system bus access
- Appropriate permissions for network management

## Logging

Enable logging via the `log` crate:

```rust
env_logger::init();
```

Set `RUST_LOG=nmrs=debug` for detailed logs.

## License

MIT
