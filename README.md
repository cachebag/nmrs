# nmrs

[![Crates.io](https://img.shields.io/crates/v/nmrs)](https://crates.io/crates/nmrs)
[![Documentation](https://docs.rs/nmrs/badge.svg)](https://docs.rs/nmrs)
[![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)](https://github.com/cachebag/nmrs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/nmrs)](LICENSE)

**Rust bindings for NetworkManager via D-Bus**

A high-level, async API for managing Wi-Fi connections on Linux systems. Built on `zbus` for reliable D-Bus communication with NetworkManager.

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
nmrs = "0.4.0"
tokio = { version = "1.48.0", features = ["full"] }
```

### Example

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // List available networks
    let networks = nm.list_networks().await?;
    for net in networks {
        println!("{} - Signal: {}%", net.ssid, net.strength.unwrap_or(0));
    }
    
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

### Features

- **Network Management**: Connect to WPA-PSK, WPA-EAP, and open networks
- **Discovery**: Scan and list available access points
- **Profile Management**: Query, create, and delete saved connections
- **Status Monitoring**: Get current connection state and signal strength
- **Typed Errors**: Structured error types with NetworkManager state reasons
- **Async/Await**: Fully asynchronous API using `tokio` or `async-std`

[View full API documentation →](https://docs.rs/nmrs)

---

## GUI Application

This repository also includes `nmrs-gui`, a Wayland compatible `NetworkManager` frontend built with GTK4.

<p align="center">
  <img src="https://github.com/user-attachments/assets/276b448d-8a7d-4b66-9318-160b2c966571" width="100%">
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

---

### Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.


## License

MIT License. See [LICENSE](./LICENSE) for details.
