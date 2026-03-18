# Error Types

nmrs uses a single error enum, `ConnectionError`, for all operations. It implements `std::error::Error`, `Display`, and `Debug`.

## ConnectionError

```rust
#[non_exhaustive]
pub enum ConnectionError {
    // D-Bus errors
    Dbus(zbus::Error),
    DbusOperation { context: String, source: zbus::Error },

    // Network not found
    NotFound,

    // Authentication
    AuthFailed,
    MissingPassword,
    SupplicantConfigFailed,
    SupplicantTimeout,

    // Connection lifecycle
    DhcpFailed,
    Timeout,
    Stuck(String),

    // Device errors
    NoWifiDevice,
    NoWiredDevice,
    WifiNotReady,
    NoBluetoothDevice,
    NoSavedConnection,

    // Device/activation failures with reason codes
    DeviceFailed(StateReason),
    ActivationFailed(ConnectionStateReason),

    // VPN errors
    NoVpnConnection,
    VpnFailed(String),
    InvalidPrivateKey(String),
    InvalidPublicKey(String),
    InvalidAddress(String),
    InvalidGateway(String),
    InvalidPeers(String),

    // Other
    InvalidUtf8(Utf8Error),
}
```

## Error Categories

### User-Facing Errors

These indicate issues the user can fix:

| Error | User Action |
|-------|------------|
| `NotFound` | Move closer to the network or check SSID spelling |
| `AuthFailed` | Check password or credentials |
| `MissingPassword` | Provide a non-empty password |
| `Timeout` | Retry or increase timeout |
| `DhcpFailed` | Check network infrastructure |
| `NoWifiDevice` | Check that a Wi-Fi adapter is installed |
| `NoWiredDevice` | Check that an Ethernet adapter exists |

### Validation Errors

These indicate invalid input to nmrs:

| Error | Fix |
|-------|-----|
| `InvalidPrivateKey` | Check WireGuard key format (base64, ~44 chars) |
| `InvalidPublicKey` | Check peer public key format |
| `InvalidAddress` | Use CIDR notation (e.g., `10.0.0.2/24`) |
| `InvalidGateway` | Use `host:port` format |
| `InvalidPeers` | Add at least one peer with allowed IPs |

### System Errors

These indicate infrastructure issues:

| Error | Investigation |
|-------|--------------|
| `Dbus` | Is NetworkManager running? Is D-Bus accessible? |
| `DbusOperation` | Check `context` for what operation failed |
| `SupplicantConfigFailed` | Check wpa_supplicant configuration |
| `SupplicantTimeout` | Check RADIUS server connectivity |
| `WifiNotReady` | Wi-Fi device still initializing |
| `Stuck` | NetworkManager in unexpected state |
| `DeviceFailed` | Check the `StateReason` for details |
| `ActivationFailed` | Check the `ConnectionStateReason` for details |

## StateReason

Low-level device state reason codes from NetworkManager. Used in `DeviceFailed`:

Common values include reasons like "supplicant disconnect", "DHCP failure", "firmware missing", "carrier dropped", and many others. These map directly to NetworkManager's `NM_DEVICE_STATE_REASON_*` constants.

## ConnectionStateReason

Activation/deactivation reason codes. Used in `ActivationFailed`:

Common values include reasons like "user disconnected", "carrier dropped", "connection removed", "dependency failed", and others. These map to NetworkManager's `NM_ACTIVE_CONNECTION_STATE_REASON_*` constants.

## ActiveConnectionState

The lifecycle state of an active connection:

```rust
pub enum ActiveConnectionState {
    Unknown,
    Activating,
    Activated,
    Deactivating,
    Deactivated,
    Other(u32),
}
```

## Error Handling Patterns

### Simple Propagation

```rust
async fn connect() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    nm.connect("MyWiFi", WifiSecurity::Open).await?;
    Ok(())
}
```

### Specific Error Handling

```rust
match nm.connect("MyWiFi", security).await {
    Ok(_) => println!("Connected"),
    Err(ConnectionError::AuthFailed) => eprintln!("Wrong password"),
    Err(ConnectionError::NotFound) => eprintln!("Network not found"),
    Err(e) => eprintln!("Error: {}", e),
}
```

### With anyhow

```rust
use anyhow::{Context, Result};

async fn connect() -> Result<()> {
    let nm = NetworkManager::new().await
        .context("Failed to connect to NetworkManager")?;
    nm.connect("MyWiFi", WifiSecurity::Open).await
        .context("Failed to connect to MyWiFi")?;
    Ok(())
}
```

## Full API Reference

See [docs.rs/nmrs](https://docs.rs/nmrs) for complete error documentation.
