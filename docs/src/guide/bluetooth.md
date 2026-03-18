# Bluetooth

nmrs supports Bluetooth network connections through NetworkManager's Bluetooth integration with BlueZ. This covers Bluetooth PAN (Personal Area Network) and DUN (Dial-Up Networking) profiles.

## Prerequisites

- BlueZ must be running (the Linux Bluetooth stack)
- The Bluetooth device must be **paired** using `bluetoothctl` or another pairing tool before nmrs can connect
- The Bluetooth adapter must be powered on

## Listing Bluetooth Devices

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let devices = nm.list_bluetooth_devices().await?;
    for device in &devices {
        println!("{}", device);
    }

    Ok(())
}
```

The `BluetoothDevice` struct provides:

| Field | Type | Description |
|-------|------|-------------|
| `bdaddr` | `String` | Bluetooth MAC address |
| `name` | `Option<String>` | Device name from BlueZ |
| `alias` | `Option<String>` | User-friendly alias |
| `bt_caps` | `u32` | Bluetooth capability flags |
| `state` | `DeviceState` | Current connection state |

The `Display` implementation shows devices as `alias (role) [MAC]`.

## Connecting

Connecting requires a device name and a `BluetoothIdentity`:

```rust
use nmrs::{NetworkManager, models::{BluetoothIdentity, BluetoothNetworkRole}};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let identity = BluetoothIdentity::new(
        "C8:1F:E8:F0:51:57".into(),
        BluetoothNetworkRole::PanU,
    )?;

    nm.connect_bluetooth("My Phone", &identity).await?;
    println!("Bluetooth connected!");

    Ok(())
}
```

### Network Roles

| Role | Description |
|------|-------------|
| `BluetoothNetworkRole::PanU` | Personal Area Network User — most common for phone tethering |
| `BluetoothNetworkRole::Dun` | Dial-Up Networking — for modem-style connections |

### BluetoothIdentity Validation

`BluetoothIdentity::new()` validates the Bluetooth MAC address format. It returns a `ConnectionError` if the address is invalid.

## Connecting to the First Available Device

A practical pattern is to list devices and connect to the first one:

```rust
use nmrs::{NetworkManager, models::BluetoothIdentity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    let devices = nm.list_bluetooth_devices().await?;
    if devices.is_empty() {
        println!("No Bluetooth devices found");
        println!("Make sure the device is paired with bluetoothctl");
        return Ok(());
    }

    let device = &devices[0];
    println!("Connecting to: {}", device);

    let identity = BluetoothIdentity::new(
        device.bdaddr.clone(),
        device.bt_caps.into(),
    )?;

    let name = device.alias.as_deref()
        .or(device.name.as_deref())
        .unwrap_or("Bluetooth Device");

    nm.connect_bluetooth(name, &identity).await?;
    println!("Connected!");

    Ok(())
}
```

## Forgetting a Bluetooth Connection

Remove a saved Bluetooth connection profile:

```rust
let nm = NetworkManager::new().await?;

nm.forget_bluetooth("My Phone").await?;
```

If the device is currently connected, it will be disconnected first before the profile is deleted.

## Errors

| Error | Meaning |
|-------|---------|
| `ConnectionError::NoBluetoothDevice` | No Bluetooth adapter found |
| `ConnectionError::InvalidAddress` | Invalid Bluetooth MAC address format |
| `ConnectionError::Timeout` | Connection took too long |
| `ConnectionError::NoSavedConnection` | No matching profile found (for forget) |

## Pairing Devices

nmrs does not handle Bluetooth pairing — that's the responsibility of BlueZ. Use `bluetoothctl` to pair devices before connecting with nmrs:

```bash
# Start bluetoothctl
bluetoothctl

# Scan for devices
scan on

# Pair with a device
pair C8:1F:E8:F0:51:57

# Trust the device (allows auto-reconnection)
trust C8:1F:E8:F0:51:57

# Exit
exit
```

After pairing, the device will appear in `list_bluetooth_devices()`.

## Next Steps

- [Device Management](./devices.md) – list all network devices including Bluetooth
- [Connection Profiles](./profiles.md) – manage saved connections
- [Error Handling](./error-handling.md) – handle Bluetooth-specific errors
