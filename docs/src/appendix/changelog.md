# Changelog

See the full changelog on GitHub: [**nmrs** CHANGELOG](https://github.com/cachebag/nmrs/blob/master/nmrs/CHANGELOG.md)

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
