# Changelog

Each crate maintains its own changelog. See the full changelogs on GitHub:

- [**nmrs** (library) CHANGELOG](https://github.com/cachebag/nmrs/blob/master/nmrs/CHANGELOG.md)
- [**nmrs-gui** (GUI) CHANGELOG](https://github.com/cachebag/nmrs/blob/master/nmrs-gui/CHANGELOG.md)

## nmrs (Library) Highlights

### 2.2.0

- Concurrency protection — `is_connecting()` API
- `WirelessHardwareEnabled` property support
- BDADDR to BlueZ path resolution
- Mixed WPA1+WPA2 network support

### 2.1.0

- `#[must_use]` annotations on public builder APIs

### 2.0.1

- IPv6 address support for devices and networks
- `WifiMode` enum for builder API
- Input validation for SSIDs, credentials, and addresses
- Idempotent `forget_vpn()` behavior

### 2.0.0

- Bluetooth support (PAN and DUN)
- Configurable timeouts via `TimeoutConfig`
- `VpnCredentials` and `EapOptions` builder patterns
- `ConnectionOptions` for autoconnect configuration
- `ConnectionBuilder` for advanced connection settings
- `WireGuardBuilder` with validation

### 1.x

- WireGuard VPN support
- VPN error handling improvements
- Docker image for testing
- Initial release with Wi-Fi and Ethernet support

## nmrs-gui (Application) Highlights

### 1.1.0

- Binary name fix for `.desktop` files and Nix

### 0.5.0-beta

- Ethernet support
- UI freeze fixes
- WPA-EAP certificate path support

### 0.4.0-beta

- Five themes: Catppuccin, Dracula, Gruvbox, Nord, Tokyo Night
- `--version` flag
- Crate rename to `nmrs-gui`

### 0.3.0-beta

- System default light/dark toggle

### 0.2.0-beta

- Default CSS file creation
- Nix dependencies
- Connection success feedback

### 0.1.0-beta

- Initial GTK4 GUI
- Basic and advanced network detail pages
- Refresh functionality
- Desktop entry and AUR support
