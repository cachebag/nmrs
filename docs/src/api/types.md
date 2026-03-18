# Core Types

This page lists the primary types exported by nmrs. For complete API documentation, see [docs.rs/nmrs](https://docs.rs/nmrs).

## NetworkManager

The main entry point for all operations.

```rust
use nmrs::NetworkManager;

let nm = NetworkManager::new().await?;
let nm = NetworkManager::with_config(config).await?;
```

- `Clone` — clones share the same D-Bus connection
- `Send + Sync` — safe to share across tasks
- See [NetworkManager API](./network-manager.md) for all methods

## Result Type

```rust
pub type Result<T> = std::result::Result<T, ConnectionError>;
```

All public methods return `nmrs::Result<T>`.

## Wi-Fi Types

| Type | Description |
|------|-------------|
| `Network` | A discovered Wi-Fi network (SSID, signal, security flags) |
| `NetworkInfo` | Detailed network information (channel, speed, bars) |
| `WifiSecurity` | Authentication type: `Open`, `WpaPsk`, `WpaEap` |
| `EapOptions` | Enterprise Wi-Fi (802.1X) configuration |
| `EapOptionsBuilder` | Builder for `EapOptions` |
| `EapMethod` | Outer EAP method: `Peap`, `Ttls` |
| `Phase2` | Inner auth method: `Mschapv2`, `Pap` |

## Device Types

| Type | Description |
|------|-------------|
| `Device` | A network device (interface, type, state, MAC) |
| `DeviceIdentity` | Device MAC addresses (permanent and current) |
| `DeviceType` | Device kind: `Wifi`, `Ethernet`, `Bluetooth`, `WifiP2P`, `Loopback`, `Other(u32)` |
| `DeviceState` | Operational state: `Disconnected`, `Activated`, `Failed`, etc. |

## VPN Types

| Type | Description |
|------|-------------|
| `VpnType` | VPN protocol: `WireGuard` |
| `VpnCredentials` | Full VPN configuration for connecting |
| `VpnCredentialsBuilder` | Builder for `VpnCredentials` |
| `WireGuardPeer` | WireGuard peer configuration |
| `VpnConnection` | A saved/active VPN connection |
| `VpnConnectionInfo` | Detailed VPN info (IP, DNS, gateway) |

## Bluetooth Types

| Type | Description |
|------|-------------|
| `BluetoothDevice` | A Bluetooth device with BlueZ info |
| `BluetoothIdentity` | Bluetooth MAC + network role for connecting |
| `BluetoothNetworkRole` | Role: `PanU`, `Dun` |

## Configuration Types

| Type | Description |
|------|-------------|
| `TimeoutConfig` | Connection/disconnection timeouts |
| `ConnectionOptions` | Autoconnect, priority, retry settings |

## Error Types

| Type | Description |
|------|-------------|
| `ConnectionError` | All possible error variants |
| `StateReason` | Device state reason codes |
| `ConnectionStateReason` | Activation/deactivation reason codes |
| `ActiveConnectionState` | Connection lifecycle states |

## Builder Types

| Type | Description |
|------|-------------|
| `ConnectionBuilder` | Base connection settings builder |
| `WifiConnectionBuilder` | Wi-Fi connection builder |
| `WireGuardBuilder` | WireGuard VPN builder |
| `IpConfig` | IP address with CIDR prefix |
| `Route` | Static route configuration |
| `WifiBand` | Wi-Fi band: `Bg` (2.4 GHz), `A` (5 GHz) |
| `WifiMode` | Wi-Fi mode: `Infrastructure`, `Adhoc`, `Ap` |

## Re-exports

nmrs re-exports commonly used types at the crate root for convenience:

```rust
use nmrs::{
    NetworkManager,
    WifiSecurity, EapOptions, EapMethod, Phase2,
    VpnCredentials, VpnType, WireGuardPeer,
    TimeoutConfig, ConnectionOptions,
    ConnectionError, DeviceType, DeviceState,
};
```

Less commonly used types are available through the `models` and `builders` modules:

```rust
use nmrs::models::{BluetoothIdentity, BluetoothNetworkRole, BluetoothDevice};
use nmrs::builders::{ConnectionBuilder, WireGuardBuilder, IpConfig, Route};
```
