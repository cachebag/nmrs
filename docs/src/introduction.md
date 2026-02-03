# Introduction

Welcome to the **nmrs** documentation! This guide will help you understand and use nmrs, a powerful Rust library for managing network connections on Linux via NetworkManager.

## What is nmrs?

**nmrs** is a high-level, async Rust API for [NetworkManager](https://networkmanager.dev/) over [D-Bus](https://dbus.freedesktop.org/doc/dbus-specification.html). It provides:

- **Simple WiFi Management** - Scan, connect, and manage wireless networks
- **VPN Support** - Full WireGuard VPN integration
- **Ethernet Control** - Manage wired network connections
- **Bluetooth** - Connect to Bluetooth network devices
- **Real-Time Monitoring** - Event-driven network state updates
- **Type Safety** - Comprehensive error handling with specific failure reasons
- **Async/Await** - Built on modern async Rust with runtime flexibility

## Project Structure

The nmrs project consists of two main components:

### nmrs (Library)
The core Rust library providing NetworkManager bindings and network management capabilities. This is what you'll use if you're building applications that need to manage network connections programmatically.

### nmrs-gui (Application)
A beautiful, Wayland-compatible GTK4 graphical interface for NetworkManager. Perfect for desktop users who want a modern network management GUI.

## Why nmrs?

### For Developers
- **Safe Abstractions** - No unsafe code, leveraging Rust's type system
- **Async-First** - Built for modern async Rust applications
- **Signal-Based** - Efficient D-Bus signal monitoring instead of polling
- **Well-Documented** - Comprehensive docs with examples for every feature
- **Runtime Agnostic** - Works with Tokio, async-std, smol, and more

### For Users (nmrs-gui)
- **Modern UI** - Clean GTK4 interface with multiple themes
- **Wayland Native** - First-class Wayland support
- **Lightweight** - Fast and efficient
- **Customizable** - CSS-based theming system
- **DE Integration** - Works great with tiling WMs (Hyprland, Sway, i3)

## Quick Example

Here's a taste of what nmrs can do:

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // Scan for networks
    let networks = nm.list_networks().await?;
    for net in networks {
        println!("{} - {}%", net.ssid, net.strength.unwrap_or(0));
    }
    
    // Connect to a network
    nm.connect("MyWiFi", WifiSecurity::WpaPsk {
        psk: "password123".into()
    }).await?;
    
    Ok(())
}
```

## Community

- **Discord**: Join our [Discord server](https://discord.gg/Sk3VfrHrN4) to chat with developers and users
- **GitHub**: Report issues, contribute, or browse the code at [github.com/cachebag/nmrs](https://github.com/cachebag/nmrs)
- **Crates.io**: Install from [crates.io/crates/nmrs](https://crates.io/crates/nmrs)
- **API Docs**: Full API reference at [docs.rs/nmrs](https://docs.rs/nmrs)

## License

nmrs is dual-licensed under MIT and Apache 2.0, giving you flexibility in how you use it.

---

Ready to get started? Head to the [Installation](./getting-started/installation.md) guide!
