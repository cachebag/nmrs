# Requirements

This page details all the requirements needed to use nmrs effectively.

## System Requirements

### Operating System

nmrs is **Linux-only** and requires:

- Any modern Linux distribution (kernel 3.10+)
- NetworkManager 1.0 or later
- D-Bus system bus

Tested on:
- Arch Linux
- Ubuntu 20.04+
- Fedora 35+
- Debian 11+
- NixOS

### NetworkManager

NetworkManager must be:
- Installed on your system
- Running and accessible via D-Bus
- Version 1.0 or later (1.46+ recommended for latest features)

Check your NetworkManager version:

```bash
NetworkManager --version
```

Ensure it's running:

```bash
systemctl status NetworkManager
```

### D-Bus

The D-Bus system bus must be available and running. This is standard on all modern Linux distributions.

Verify D-Bus is working:

```bash
dbus-send --system --print-reply \
  --dest=org.freedesktop.NetworkManager \
  /org/freedesktop/NetworkManager \
  org.freedesktop.DBus.Properties.Get \
  string:'org.freedesktop.NetworkManager' \
  string:'Version'
```

## Rust Requirements

### For nmrs Library

- **Rust**: 1.78.0 or later
- **Edition**: 2021

The library uses stable Rust features only.

### For nmrs-gui

- **Rust**: 1.85.1 or later
- **Edition**: 2021

The GUI requires a newer Rust version due to GTK4 bindings.

## Dependencies

### nmrs Library Dependencies

The library depends on:

- `zbus` 5.x - D-Bus communication
- `tokio` or another async runtime
- `serde` - Serialization
- `thiserror` - Error handling
- `futures` - Async utilities

All dependencies are automatically handled by Cargo.

### nmrs-gui Dependencies

Additional system libraries required:

**Arch Linux:**
```bash
sudo pacman -S gtk4 libadwaita
```

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel
```

**NixOS:**
```nix
# Handled automatically by the Nix package
```

## Permissions

### PolicyKit

nmrs needs permission to manage network connections. This is typically handled by PolicyKit on modern Linux systems.

### User Groups

Your user should be in the appropriate groups. On most systems:

```bash
# Check current groups
groups

# Add to network group (may vary by distribution)
sudo usermod -aG network $USER
```

On some distributions, no special group is needed if PolicyKit is properly configured.

### Running as Root

While nmrs can run as root, it's **not recommended** for security reasons. Use PolicyKit instead.

## Hardware Requirements

### WiFi

- A WiFi adapter supported by NetworkManager
- WiFi hardware must be recognized by the Linux kernel

Check your WiFi adapter:

```bash
nmcli device status
```

### Ethernet

- Network interface card (NIC)
- Recognized by the Linux kernel

### Bluetooth

For Bluetooth network features:
- Bluetooth adapter
- BlueZ stack installed and running

### VPN

For WireGuard VPN:
- WireGuard kernel module or userspace implementation
- WireGuard tools (usually bundled with NetworkManager)

Check WireGuard support:

```bash
modprobe wireguard
lsmod | grep wireguard
```

Or use userspace implementation (automatic with NetworkManager 1.16+).

## Development Requirements

Additional requirements for developing nmrs:

### Testing

- `docker` and `docker-compose` (for containerized testing)
- WiFi hardware or `mac80211_hwsim` kernel module

### Building Documentation

- `mdbook` for this documentation
- `cargo-doc` for API documentation

### IDE/Editor

Recommended:
- rust-analyzer
- clippy
- rustfmt

## Optional Dependencies

### Logging

For detailed logging, use any logger that implements the `log` facade:

```toml
[dependencies]
env_logger = "0.11"
```

### TLS/Certificates

For WPA-EAP with certificate validation:
- CA certificates installed in system certificate store
- OpenSSL or rustls (handled by NetworkManager)

## Troubleshooting

### NetworkManager Not Found

If you get "Failed to connect to D-Bus":

```bash
# Check if NetworkManager is running
systemctl status NetworkManager

# Start it if needed
sudo systemctl start NetworkManager

# Enable it to start on boot
sudo systemctl enable NetworkManager
```

### Permission Denied

If you get permission errors:

1. Check PolicyKit rules: `/usr/share/polkit-1/actions/org.freedesktop.NetworkManager.policy`
2. Ensure D-Bus is accessible: `ls -l /var/run/dbus/system_bus_socket`
3. Try with PolicyKit agent running

### Dependency Issues

For build issues:

```bash
# Update Rust
rustup update stable

# Clear Cargo cache
cargo clean

# Update dependencies
cargo update
```

## Version Compatibility

### nmrs Library

| nmrs Version | Minimum Rust | NetworkManager | Notable Features |
|--------------|--------------|----------------|------------------|
| 2.0.0        | 1.78.0       | 1.0+           | Full API rewrite |
| 1.x          | 1.70.0       | 1.0+           | Initial release  |

### nmrs-gui

| GUI Version | Minimum Rust | GTK | Notable Features |
|-------------|--------------|-----|------------------|
| 1.1.0       | 1.85.1       | 4.0 | Themes support   |
| 1.0.0       | 1.82.0       | 4.0 | Initial release  |

## Next Steps

Once you have all requirements met:

1. [Install nmrs](./installation.md)
2. [Follow the Quick Start guide](./quick-start.md)
3. Start building with [WiFi Management](../guide/wifi.md)

If you encounter issues, see [Troubleshooting](../appendix/troubleshooting.md).
