# Error Handling

nmrs uses a single error type, `ConnectionError`, for all operations. Each variant describes a specific failure mode, making it straightforward to handle errors precisely.

## The Result Type

nmrs re-exports a `Result` type alias:

```rust
pub type Result<T> = std::result::Result<T, ConnectionError>;
```

All public API methods return `nmrs::Result<T>`.

## ConnectionError Variants

### Network & Wi-Fi Errors

| Variant | Description |
|---------|-------------|
| `NotFound` | Network not visible during scan |
| `AuthFailed` | Wrong password or rejected credentials |
| `MissingPassword` | Empty password provided |
| `NoWifiDevice` | No Wi-Fi adapter found |
| `WifiNotReady` | Wi-Fi device not ready in time |
| `NoWiredDevice` | No Ethernet adapter found |
| `DhcpFailed` | Failed to obtain an IP address via DHCP |
| `Timeout` | Operation timed out waiting for activation |
| `Stuck(String)` | Connection stuck in an unexpected state |

### Authentication Errors

| Variant | Description |
|---------|-------------|
| `SupplicantConfigFailed` | wpa_supplicant configuration error |
| `SupplicantTimeout` | wpa_supplicant timed out during auth |

### VPN Errors

| Variant | Description |
|---------|-------------|
| `NoVpnConnection` | VPN not found or not active |
| `VpnFailed(String)` | VPN connection failed with details |
| `InvalidPrivateKey(String)` | Bad WireGuard private key |
| `InvalidPublicKey(String)` | Bad WireGuard public key |
| `InvalidAddress(String)` | Bad IP address or CIDR notation |
| `InvalidGateway(String)` | Bad gateway format (host:port) |
| `InvalidPeers(String)` | Invalid peer configuration |

### Bluetooth Errors

| Variant | Description |
|---------|-------------|
| `NoBluetoothDevice` | No Bluetooth adapter found |

### Profile Errors

| Variant | Description |
|---------|-------------|
| `NoSavedConnection` | No saved profile for the requested network |

### Low-Level Errors

| Variant | Description |
|---------|-------------|
| `Dbus(zbus::Error)` | D-Bus communication error |
| `DbusOperation { context, source }` | D-Bus error with context |
| `DeviceFailed(StateReason)` | Device failure with NM reason code |
| `ActivationFailed(ConnectionStateReason)` | Activation failure with reason |
| `InvalidUtf8(Utf8Error)` | Invalid UTF-8 in SSID |

## Basic Error Handling

Use the `?` operator for simple propagation:

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    nm.connect("MyWiFi", WifiSecurity::Open).await?;
    Ok(())
}
```

## Pattern Matching

Handle specific errors differently:

```rust
use nmrs::{NetworkManager, WifiSecurity, ConnectionError};

let nm = NetworkManager::new().await?;

match nm.connect("MyWiFi", WifiSecurity::WpaPsk {
    psk: "password".into(),
}).await {
    Ok(_) => println!("Connected!"),
    Err(ConnectionError::NotFound) => {
        eprintln!("Network not in range");
    }
    Err(ConnectionError::AuthFailed) => {
        eprintln!("Wrong password");
    }
    Err(ConnectionError::Timeout) => {
        eprintln!("Connection timed out");
    }
    Err(ConnectionError::DhcpFailed) => {
        eprintln!("Connected to AP but DHCP failed");
    }
    Err(ConnectionError::NoWifiDevice) => {
        eprintln!("No Wi-Fi adapter found");
    }
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

## Retry Logic

Implement retries for transient failures:

```rust
use nmrs::{NetworkManager, WifiSecurity, ConnectionError};

let nm = NetworkManager::new().await?;

for attempt in 1..=3 {
    match nm.connect("MyWiFi", WifiSecurity::WpaPsk {
        psk: "password".into(),
    }).await {
        Ok(_) => {
            println!("Connected on attempt {}", attempt);
            break;
        }
        Err(ConnectionError::Timeout) if attempt < 3 => {
            eprintln!("Attempt {} timed out, retrying...", attempt);
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

## VPN Error Handling

```rust
use nmrs::{NetworkManager, ConnectionError};

let nm = NetworkManager::new().await?;

match nm.get_vpn_info("MyVPN").await {
    Ok(info) => println!("VPN IP: {:?}", info.ip4_address),
    Err(ConnectionError::NoVpnConnection) => {
        eprintln!("VPN is not active");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Converting to Other Error Types

`ConnectionError` implements `std::error::Error` and `Display`, so it works with error handling crates like `anyhow`:

```rust
use anyhow::Result;
use nmrs::NetworkManager;

async fn connect() -> Result<()> {
    let nm = NetworkManager::new().await?;
    nm.connect("MyWiFi", nmrs::WifiSecurity::Open).await?;
    Ok(())
}
```

## Non-Exhaustive

`ConnectionError` is marked `#[non_exhaustive]`, which means new variants may be added in future versions without a breaking change. Always include a wildcard arm in match expressions:

```rust
match result {
    Err(ConnectionError::AuthFailed) => { /* ... */ }
    Err(ConnectionError::NotFound) => { /* ... */ }
    Err(e) => { /* catch-all for current and future variants */ }
    Ok(_) => {}
}
```

## Next Steps

- [WiFi Management](./wifi.md) – Wi-Fi-specific operations
- [VPN Management](./vpn-management.md) – VPN-specific errors
- [Custom Timeouts](../advanced/timeouts.md) – prevent timeout errors
