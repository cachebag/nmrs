# <p align="center"> nmrs 🦀 

[![Crates.io](https://img.shields.io/crates/v/nmrs)](https://crates.io/crates/nmrs)
[![Discord](https://img.shields.io/badge/chat-on%20discord-7289da?logo=discord&logoColor=white)](https://discord.gg/Sk3VfrHrN4)
[![Documentation](https://docs.rs/nmrs/badge.svg)](https://docs.rs/nmrs)
[![User Guide](https://img.shields.io/badge/docs-mdBook-blue)](https://cachebag.github.io/nmrs/)
[![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)](https://github.com/cachebag/nmrs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/nmrs)](LICENSE)

An async-first Rust API for [NetworkManager](https://networkmanager.dev/) over [D-Bus](https://dbus.freedesktop.org/doc/dbus-specification.html). The goal is to provide a safe and simple high-level API for managing Wi-Fi connections on Linux systems, built on [`zbus`](https://docs.rs/zbus) for reliable D-Bus communication.

## Documentation

- **[User Guide](https://cachebag.github.io/nmrs/)** - Comprehensive guide with tutorials and examples
- **[API Documentation](https://docs.rs/nmrs)** - Complete API reference on docs.rs
- **[Discord](https://discord.gg/Sk3VfrHrN4)** - Join our community for help and discussion

## Getting Started

_Please consider joining the [**Discord**](https://discord.gg/Sk3VfrHrN4). It's a welcoming community to both developers who want to contribute and/or learn about and discuss nmrs as well as users that would like to be engaged with the development process._

The best way to get started with `nmrs` is the [User Guide](https://cachebag.github.io/nmrs/), which includes comprehensive tutorials and examples. For detailed API information, see the [API documentation](https://docs.rs/nmrs).

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
    let networks = nm.list_networks(None).await?;
    
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
    nm.connect("MyNetwork", None, WifiSecurity::WpaPsk {
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
    
    match nm.connect("MyNetwork", None, WifiSecurity::WpaPsk {
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

<details>
<summary><strong>Roadmap / Implementation Status</strong></summary>

### Devices

- [x] Generic  
- [x] Wireless  
- [ ] Any  
- [x] Wired  
- [ ] ADSL  
- [x] Bluetooth  
- [ ] Bond  
- [ ] Bridge  
- [ ] Dummy  
- [ ] HSR *(NetworkManager ≥ 1.46)*  
- [ ] Infiniband  
- [ ] IP Tunnel  
- [ ] IPVLAN *(NetworkManager ≥ 1.52)*  
- [ ] Lowpan  
- [x] Loopback  
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
- [x] Agent Manager  
- [ ] Checkpoint  
- [ ] DNS Manager  
- [ ] PPP  
- [x] Secret Agent  
- [x] VPN Connection (WireGuard + OpenVPN)  
- [x] General Plugin VPN (OpenConnect, strongSwan, PPTP, L2TP, etc.)  
- [ ] Wi-Fi P2P  
- [ ] WiMAX NSP  

</details>

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## Requirements

- **Rust**: 1.90.0+
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

## Contributors

Thank you to everyone who has helped build, test, document, and review `nmrs`.

<!-- readme: contributors -start -->
<table>
  <tr>
    <td align="center"><a href="https://github.com/cachebag"><img src="https://avatars.githubusercontent.com/u/111914307?v=4" width="100px;" alt="cachebag"/><br /><sub><b>cachebag</b></sub></a></td>
    <td align="center"><a href="https://github.com/stoutes"><img src="https://avatars.githubusercontent.com/u/31317041?v=4" width="100px;" alt="stoutes"/><br /><sub><b>stoutes</b></sub></a></td>
    <td align="center"><a href="https://github.com/pluiee"><img src="https://avatars.githubusercontent.com/u/93393389?v=4" width="100px;" alt="pluiee"/><br /><sub><b>pluiee</b></sub></a></td>
    <td align="center"><a href="https://github.com/JonnieCache"><img src="https://avatars.githubusercontent.com/u/211093?v=4" width="100px;" alt="JonnieCache"/><br /><sub><b>JonnieCache</b></sub></a></td>
    <td align="center"><a href="https://github.com/tristanmsct"><img src="https://avatars.githubusercontent.com/u/69300092?v=4" width="100px;" alt="tristanmsct"/><br /><sub><b>tristanmsct</b></sub></a></td>
    <td align="center"><a href="https://github.com/Rifat-R"><img src="https://avatars.githubusercontent.com/u/81259132?v=4" width="100px;" alt="Rifat-R"/><br /><sub><b>Rifat-R</b></sub></a></td>
  </tr>
  <tr>
    <td align="center"><a href="https://github.com/of-the-stars"><img src="https://avatars.githubusercontent.com/u/47869156?v=4" width="100px;" alt="of-the-stars"/><br /><sub><b>of-the-stars</b></sub></a></td>
    <td align="center"><a href="https://github.com/okhsunrog"><img src="https://avatars.githubusercontent.com/u/42293787?v=4" width="100px;" alt="okhsunrog"/><br /><sub><b>okhsunrog</b></sub></a></td>
    <td align="center"><a href="https://github.com/ruthwik-01"><img src="https://avatars.githubusercontent.com/u/235033610?v=4" width="100px;" alt="ruthwik-01"/><br /><sub><b>ruthwik-01</b></sub></a></td>
    <td align="center"><a href="https://github.com/joncorv"><img src="https://avatars.githubusercontent.com/u/151096562?v=4" width="100px;" alt="joncorv"/><br /><sub><b>joncorv</b></sub></a></td>
    <td align="center"><a href="https://github.com/AK78gz"><img src="https://avatars.githubusercontent.com/u/89071188?v=4" width="100px;" alt="AK78gz"/><br /><sub><b>AK78gz</b></sub></a></td>
    <td align="center"><a href="https://github.com/pwsandoval"><img src="https://avatars.githubusercontent.com/u/15174704?v=4" width="100px;" alt="pwsandoval"/><br /><sub><b>pwsandoval</b></sub></a></td>
  </tr>
  <tr>
    <td align="center"><a href="https://github.com/ritiek"><img src="https://avatars.githubusercontent.com/u/20314742?v=4" width="100px;" alt="ritiek"/><br /><sub><b>ritiek</b></sub></a></td>
    <td align="center"><a href="https://github.com/shubhsingh5901"><img src="https://avatars.githubusercontent.com/u/110416544?v=4" width="100px;" alt="shubhsingh5901"/><br /><sub><b>shubhsingh5901</b></sub></a></td>
    <td align="center"><a href="https://github.com/cinnamonstic"><img src="https://avatars.githubusercontent.com/u/182801542?v=4" width="100px;" alt="cinnamonstic"/><br /><sub><b>cinnamonstic</b></sub></a></td>
    <td align="center"><a href="https://github.com/tuned-willow"><img src="https://avatars.githubusercontent.com/u/250158319?v=4" width="100px;" alt="tuned-willow"/><br /><sub><b>tuned-willow</b></sub></a></td>
    <td align="center"><a href="https://github.com/dandiggas"><img src="https://avatars.githubusercontent.com/u/78769670?v=4" width="100px;" alt="dandiggas"/><br /><sub><b>dandiggas</b></sub></a></td>  
  </tr>
</table>
<!-- readme: contributors -end -->
