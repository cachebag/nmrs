# <p align="center"> nmrs 🦀 

[![Crates.io](https://img.shields.io/crates/v/nmrs)](https://crates.io/crates/nmrs)
[![Discord](https://img.shields.io/badge/chat-on%20discord-7289da?logo=discord&logoColor=white)](https://discord.gg/Sk3VfrHrN4)
[![Documentation](https://docs.rs/nmrs/badge.svg)](https://docs.rs/nmrs)
[![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)](https://github.com/cachebag/nmrs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/nmrs)](LICENSE)

A Rust API for [NetworkManager](https://networkmanager.dev/) over [D-Bus](https://dbus.freedesktop.org/doc/dbus-specification.html). The goal is to provide a safe and simple high-level API for managing Wi-Fi connections on Linux systems, built on [`zbus`](https://docs.rs/zbus) for reliable D-Bus communication.

The project is divided into the following crates:

* `nmrs`: The core library providing NetworkManager bindings and Wi-Fi management API.
* `nmrs-gui`: A Wayland-compatible GTK4 graphical interface for NetworkManager.

## Getting Started

The best way to get started with `nmrs` is the [API documentation](https://docs.rs/nmrs), which includes examples for common operations like scanning networks, connecting to Wi-Fi, and managing connection profiles.

## Sample usage
We'll create a simple example that scans for available networks and connects to one. Note that these examples require NetworkManager to be running on your Linux system with D-Bus access, obviously.

### Listing Networks

Scan for and display available Wi-Fi networks:

```rust,no_run
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // Scan for networks
    let networks = nm.list_networks().await?;
    
    for net in networks {
        println!(
            "{} - Signal: {}%, Security: {:?}",
            net.ssid,
            net.strength.unwrap_or(0),
            net.security
        );
    }
    
    Ok(())
}
```

### Now let's connect to a network...

Connect to a WPA-PSK protected network:

```rust,no_run
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // Connect to a network
    nm.connect("MyNetwork", WifiSecurity::WpaPsk {
        psk: "password123".into()
    }).await?;
    
    // Check current connection
    if let Some(ssid) = nm.current_ssid().await {
        println!("Connected to: {}", ssid);
    }
    
    Ok(())
}
```

### Error Handling

All operations return `Result<T, ConnectionError>` with specific error variants:

```rust,no_run
use nmrs::{NetworkManager, WifiSecurity, ConnectionError};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    match nm.connect("MyNetwork", WifiSecurity::WpaPsk {
        psk: "wrong_password".into()
    }).await {
        Ok(_) => println!("Connected successfully"),
        Err(ConnectionError::AuthFailed) => eprintln!("Authentication failed - wrong password"),
        Err(ConnectionError::NotFound) => eprintln!("Network not found or out of range"),
        Err(ConnectionError::Timeout) => eprintln!("Connection timed out"),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}
```

To follow and/or discuss the development of nmrs, you can join the [public Discord channel](https://discord.gg/Sk3VfrHrN4).
 
# <p align="center"> nmrs-gui </p>

![Version](https://img.shields.io/badge/nmrs--gui-1.1.0-orange?style=flat-square)
[![Nix](https://github.com/cachebag/nmrs/actions/workflows/nix.yml/badge.svg)](https://github.com/cachebag/nmrs/actions/workflows/nix.yml)

This repository also includes `nmrs-gui`, a Wayland-compatible NetworkManager frontend built with GTK4.

<p align="center">
<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/fc0fc636-2fa3-4d80-b43e-71f830b10053" />
</p>

### Installation

**Arch Linux (AUR)**

```bash
yay -S nmrs
# or
paru -S nmrs
```

**Nix**

```bash
nix-shell -p nmrs
```

### Configuration

**Waybar Integration**

```json
"network": {
    "on-click": "nmrs"
}
```

**Tiling Window Managers** (Hyprland, Sway, i3)

```
windowrulev2 = float, class:^(org\.nmrs\.ui)$
```

**Custom Styling**

Edit `~/.config/nmrs/style.css` to customize the interface. There are also pre-defined themes you can pick from in the interface itself.

<details>
<summary><strong>Roadmap / Implementation Status</strong></summary>

### Devices

- [x] Generic  
- [x] Wireless  
- [ ] Any  
- [X] Wired  
- [ ] ADSL  
- [X] Bluetooth  
- [ ] Bond  
- [ ] Bridge  
- [ ] Dummy  
- [ ] HSR *(NetworkManager ≥ 1.46)*  
- [ ] Infiniband  
- [ ] IP Tunnel  
- [ ] IPVLAN *(NetworkManager ≥ 1.52)*  
- [ ] Lowpan  
- [ ] Loopback  
- [ ] MACsec  
- [ ] MACVLAN  
- [ ] Modem  
- [ ] OLPC Mesh  
- [ ] OVS Bridge  
- [ ] OVS Interface  
- [ ] OVS Port  
- [ ] PPP  
- [ ] Statistics  
- [ ] Team  
- [ ] TUN/TAP  
- [ ] VETH  
- [ ] VLAN  
- [ ] VRF  
- [ ] VXLAN  
- [ ] Wi-Fi P2P  
- [ ] WiMAX  
- [X] WireGuard  
- [ ] WPAN  

### Configurations

- [x] IPv4  
- [x] IPv6  
- [x] DHCPv4  
- [x] DHCPv6  

### Core Interfaces

- [x] NetworkManager *(partial)*  
- [x] Device  
- [x] Access Point  
- [x] Active Connection  
- [x] Settings  
- [x] Settings Connection  
- [ ] Agent Manager  
- [ ] Checkpoint  
- [ ] DNS Manager  
- [ ] PPP  
- [ ] Secret Agent  
- [X] VPN Connection (WireGuard)  
- [ ] VPN Plugin  
- [ ] Wi-Fi P2P  
- [ ] WiMAX NSP  

</details>

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## Requirements

- **Rust**: 1.78.0 or later (for `nmrs` library)
- **Rust**: 1.85.1 or later (for `nmrs-gui` with GTK4)
- **NetworkManager**: Running and accessible via D-Bus
- **Linux**: This library is Linux-specific

## License

This project is dual-licensed under either of the following licenses, at your option:

- MIT License  
- Apache License, Version 2.0

You may use, copy, modify, and distribute this software under the terms of either license.

See the following files for full license texts:
- [MIT License](./LICENSE-MIT)
- [Apache License 2.0](./LICENSE-APACHE)
