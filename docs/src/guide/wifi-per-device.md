# Per-Device Wi-Fi Scoping

Many machines have more than one Wi-Fi radio — a built-in card plus a USB dongle, a laptop in a dock with a secondary adapter, or an IoT gateway with dual radios on different bands. By default, nmrs routes every Wi-Fi operation through whichever device NetworkManager returns first. That works on single-radio systems, but on multi-radio setups you need to control *which* adapter scans, connects, or gets disabled.

nmrs 3.0 introduces per-device scoping so you can target a specific interface by name.

## Listing Wi-Fi Devices

Start by discovering the available radios:

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let devices = nm.list_wifi_devices().await?;
    for dev in &devices {
        println!("{} ({})", dev.interface, dev.mac);
        println!("  State: {:?}", dev.state);
        if let Some(ssid) = &dev.active_ssid {
            println!("  Connected to: {}", ssid);
        }
    }

    Ok(())
}
```

Each `WifiDevice` contains:

| Field | Type | Description |
|-------|------|-------------|
| `interface` | `String` | Interface name (`wlan0`, `wlp2s0`, …) |
| `mac` | `String` | Hardware MAC address |
| `state` | `DeviceState` | Current operational state |
| `active_ssid` | `Option<String>` | SSID of the active connection, if any |

You can also look up a single device directly:

```rust
let dev = nm.wifi_device_by_interface("wlan1").await?;
println!("{} is {:?}", dev.interface, dev.state);
```

## The WifiScope Pattern

The most ergonomic way to work with a specific radio is `WifiScope`. Call `nm.wifi("wlan1")` to get a scope pinned to that interface, then chain operations without repeating the interface name:

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let scope = nm.wifi("wlan1");

    scope.scan().await?;
    let networks = scope.list_networks().await?;
    for net in &networks {
        println!("{} ({}%)", net.ssid, net.strength.unwrap_or(0));
    }

    scope.connect("HomeWiFi", WifiSecurity::WpaPsk {
        psk: "hunter2".into(),
    }).await?;

    Ok(())
}
```

`WifiScope` delegates to `NetworkManager` under the hood but locks every call to a single interface. The available methods are:

| Method | Description |
|--------|-------------|
| `scope.interface()` | Returns the interface name this scope is pinned to |
| `scope.scan()` | Trigger a scan on this device |
| `scope.list_networks()` | List networks visible to this device |
| `scope.list_access_points()` | List raw access points (including duplicates per BSSID) |
| `scope.connect(ssid, creds)` | Connect through this device |
| `scope.connect_to_bssid(ssid, bssid, creds)` | Connect to a specific BSSID through this device |
| `scope.disconnect()` | Disconnect this device |
| `scope.set_enabled(bool)` | Enable or disable this device |
| `scope.forget(ssid)` | Remove a saved connection from this device |

Because the interface is already captured, none of these methods take an interface parameter.

### BSSID targeting

When the same SSID is broadcast by multiple access points, use `connect_to_bssid` to force a specific one:

```rust
let scope = nm.wifi("wlan0");

let aps = scope.list_access_points().await?;
if let Some(best) = aps.iter().max_by_key(|ap| ap.strength.unwrap_or(0)) {
    scope.connect_to_bssid(
        &best.ssid,
        &best.hwaddress.as_deref().unwrap_or_default(),
        WifiSecurity::WpaPsk { psk: "password".into() },
    ).await?;
}
```

## Per-Interface vs Global Operations

nmrs distinguishes between operations that target one device and operations that affect the entire Wi-Fi subsystem.

| Operation | Per-device | Global |
|-----------|-----------|--------|
| Enable/disable radio | `nm.set_wifi_enabled("wlan1", true)` | `nm.set_wireless_enabled(false)` |
| Scan | `nm.scan_networks(Some("wlan1"))` | `nm.scan_networks(None)` (scans all) |
| List networks | `nm.list_networks(Some("wlan1"))` | `nm.list_networks(None)` (merges all) |
| Connect | `nm.connect("ssid", Some("wlan1"), creds)` | `nm.connect("ssid", None, creds)` |
| Disconnect | `nm.disconnect(Some("wlan1"))` | `nm.disconnect(None)` (all devices) |

When you pass `None`, nmrs falls back to the original behavior: pick the first Wi-Fi device for single-device operations, or aggregate across all devices for scans and listings.

## Per-Device Enable/Disable

There are two distinct toggles:

- **`set_wireless_enabled(bool)`** flips NetworkManager's global `WirelessEnabled` property. This affects *every* Wi-Fi radio on the system — equivalent to airplane-mode for Wi-Fi.
- **`set_wifi_enabled(interface, bool)`** targets a single radio. It sets `Autoconnect = false` and disconnects the device (to disable) or re-enables autoconnect (to enable). The rest of the system's Wi-Fi radios are unaffected.

```rust
let nm = NetworkManager::new().await?;

// Disable only the USB dongle
nm.set_wifi_enabled("wlan1", false).await?;

// The built-in radio stays online
let dev = nm.wifi_device_by_interface("wlan0").await?;
assert_ne!(dev.state, nmrs::DeviceState::Unavailable);
```

Using `WifiScope`:

```rust
let scope = nm.wifi("wlan1");
scope.set_enabled(false).await?;
```

> **Note:** `set_wifi_enabled` is *not* the same as the global `set_wireless_enabled`. The global toggle controls the NM-level `WirelessEnabled` property (equivalent to `nmcli radio wifi off`), while per-device disable works through the device's autoconnect and disconnect mechanism.

## Direct Method Approach

If you don't want a `WifiScope`, every Wi-Fi method on `NetworkManager` accepts an optional interface name. Pass `None` for single-radio behavior or `Some("wlan1")` to target a device:

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Scan on a specific interface
    nm.scan_networks(Some("wlan1")).await?;

    // List networks from a specific interface
    let networks = nm.list_networks(Some("wlan1")).await?;

    // Connect through a specific interface
    nm.connect("OfficeWiFi", Some("wlan1"), WifiSecurity::WpaPsk {
        psk: "secret".into(),
    }).await?;

    // Disconnect a specific interface
    nm.disconnect(Some("wlan1")).await?;

    // Or use None to get the default (first device) behavior
    nm.scan_networks(None).await?;
    nm.connect("HomeWiFi", None, WifiSecurity::Open).await?;

    Ok(())
}
```

## Error Handling

Two error variants are specific to per-device scoping:

```rust
use nmrs::ConnectionError;

let nm = NetworkManager::new().await?;

match nm.wifi_device_by_interface("wlan99").await {
    Ok(dev) => println!("Found: {}", dev.interface),
    Err(ConnectionError::WifiInterfaceNotFound { interface }) => {
        eprintln!("No Wi-Fi device named '{}'", interface);
    }
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

| Variant | Meaning |
|---------|---------|
| `WifiInterfaceNotFound { interface }` | No network device with that name exists |
| `NotAWifiDevice { interface }` | The interface exists but is not a Wi-Fi device (e.g., `eth0`) |

`NotAWifiDevice` fires when you pass a valid interface name that belongs to an Ethernet or Bluetooth adapter:

```rust
match nm.wifi_device_by_interface("eth0").await {
    Err(ConnectionError::NotAWifiDevice { interface }) => {
        eprintln!("'{}' is not a Wi-Fi interface", interface);
    }
    _ => {}
}
```

## Next Steps

- [WiFi Management](./wifi.md) — general Wi-Fi operations (scanning, security types, connection options)
- [Device Management](./devices.md) — listing and inspecting all device types
- [Real-Time Monitoring](./monitoring.md) — subscribe to device state changes
- [Error Handling](./error-handling.md) — full error variant reference
