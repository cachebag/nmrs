# NetworkManager API

The `NetworkManager` struct is the primary entry point for all nmrs operations. It manages a D-Bus connection to the NetworkManager daemon.

## Construction

```rust
use nmrs::{NetworkManager, TimeoutConfig};
use std::time::Duration;

// Default timeouts (30s connect, 10s disconnect)
let nm = NetworkManager::new().await?;

// Custom timeouts
let config = TimeoutConfig::new()
    .with_connection_timeout(Duration::from_secs(60))
    .with_disconnect_timeout(Duration::from_secs(20));
let nm = NetworkManager::with_config(config).await?;

// Read current config
let config = nm.timeout_config();
```

## Wi-Fi Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `scan_networks()` | `Result<()>` | Trigger active Wi-Fi scan |
| `list_networks()` | `Result<Vec<Network>>` | List visible networks |
| `connect(ssid, security)` | `Result<()>` | Connect to a Wi-Fi network |
| `disconnect()` | `Result<()>` | Disconnect from current network |
| `current_network()` | `Result<Option<Network>>` | Get current Wi-Fi network |
| `current_ssid()` | `Option<String>` | Get current SSID |
| `current_connection_info()` | `Option<(String, Option<u32>)>` | Get SSID + frequency |
| `is_connected(ssid)` | `Result<bool>` | Check if connected to a specific network |
| `show_details(network)` | `Result<NetworkInfo>` | Get detailed network info |

## Ethernet Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `connect_wired()` | `Result<()>` | Connect first available Ethernet device |

## VPN Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `connect_vpn(creds)` | `Result<()>` | Connect to a VPN |
| `disconnect_vpn(name)` | `Result<()>` | Disconnect a VPN by name |
| `list_vpn_connections()` | `Result<Vec<VpnConnection>>` | List all saved VPNs |
| `forget_vpn(name)` | `Result<()>` | Delete a saved VPN profile |
| `get_vpn_info(name)` | `Result<VpnConnectionInfo>` | Get active VPN details |

## Bluetooth Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `list_bluetooth_devices()` | `Result<Vec<BluetoothDevice>>` | List Bluetooth devices |
| `connect_bluetooth(name, identity)` | `Result<()>` | Connect to a Bluetooth device |
| `forget_bluetooth(name)` | `Result<()>` | Delete a Bluetooth profile |

## Device Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `list_devices()` | `Result<Vec<Device>>` | List all network devices |
| `list_wireless_devices()` | `Result<Vec<Device>>` | List Wi-Fi devices |
| `list_wired_devices()` | `Result<Vec<Device>>` | List Ethernet devices |
| `get_device_by_interface(name)` | `Result<OwnedObjectPath>` | Find device by interface name |
| `is_connecting()` | `Result<bool>` | Check if any device is connecting |

## Wi-Fi Control Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `wifi_enabled()` | `Result<bool>` | Check if Wi-Fi is enabled |
| `set_wifi_enabled(bool)` | `Result<()>` | Enable/disable Wi-Fi |
| `wifi_hardware_enabled()` | `Result<bool>` | Check hardware radio state (rfkill) |
| `wait_for_wifi_ready()` | `Result<()>` | Wait for Wi-Fi device to become ready |

## Connection Profile Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `list_saved_connections()` | `Result<Vec<String>>` | List all saved profiles |
| `has_saved_connection(ssid)` | `Result<bool>` | Check if a profile exists |
| `get_saved_connection_path(ssid)` | `Result<Option<OwnedObjectPath>>` | Get profile D-Bus path |
| `forget(ssid)` | `Result<()>` | Delete a Wi-Fi profile |

## Monitoring Methods

| Method | Returns | Description |
|--------|---------|-------------|
| `monitor_network_changes(callback)` | `Result<()>` | Watch for AP changes (runs forever) |
| `monitor_device_changes(callback)` | `Result<()>` | Watch for device state changes (runs forever) |

## Thread Safety

`NetworkManager` is `Clone`, `Send`, and `Sync`. Clones share the same D-Bus connection.

**Important:** Concurrent connection operations (calling `connect()` from multiple tasks) are not supported. Use `is_connecting()` to guard against this.

## Full API Reference

For complete documentation with all method signatures, see [docs.rs/nmrs](https://docs.rs/nmrs).
