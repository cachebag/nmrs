# Real-Time Monitoring

nmrs uses D-Bus signals to provide real-time notifications when network state changes. This is more efficient than polling — your callback fires only when something actually changes.

## Network Change Monitoring

Subscribe to network changes (access points appearing or disappearing, or signal
strength changing):

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // This runs indefinitely — spawn it as a background task
    nm.monitor_network_changes(|| {
        println!("Network list changed!");
    }).await?;

    Ok(())
}
```

`monitor_network_changes()` subscribes to D-Bus signals for access point additions, removals, and signal strength updates on all Wi-Fi devices. The callback fires whenever the visible network list or signal data changes.

## Device State Monitoring

Subscribe to device state changes (connected, disconnected, cable plugged in, etc.):

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    nm.monitor_device_changes(|| {
        println!("Device state changed!");
    }).await?;

    Ok(())
}
```

`monitor_device_changes()` subscribes to state change signals on all network devices — both wired and wireless.

## Running Monitors as Background Tasks

Both monitoring functions run indefinitely. In a real application, spawn them as background tasks:

### With Tokio

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    // Spawn network monitor
    let nm_clone = nm.clone();
    tokio::spawn(async move {
        if let Err(e) = nm_clone.monitor_network_changes(|| {
            println!("Networks changed");
        }).await {
            eprintln!("Network monitor error: {}", e);
        }
    });

    // Spawn device monitor
    let nm_clone = nm.clone();
    tokio::spawn(async move {
        if let Err(e) = nm_clone.monitor_device_changes(|| {
            println!("Device state changed");
        }).await {
            eprintln!("Device monitor error: {}", e);
        }
    });

    // Your main application logic here
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
```

### With GTK/GLib (for GUI applications)

```rust
use nmrs::NetworkManager;

// Inside a GTK application
let nm = NetworkManager::new().await?;

glib::MainContext::default().spawn_local({
    let nm = nm.clone();
    async move {
        let _ = nm.monitor_network_changes(|| {
            println!("Networks changed — refresh the UI!");
        }).await;
    }
});

glib::MainContext::default().spawn_local({
    let nm = nm.clone();
    async move {
        let _ = nm.monitor_device_changes(|| {
            println!("Device changed — update status!");
        }).await;
    }
});
```

## Thread Safety

`NetworkManager` is `Clone` and can be safely shared across async tasks. Each clone shares the same underlying D-Bus connection, making it lightweight to pass into multiple monitoring tasks.

## Practical Pattern: Refresh on Change

A common pattern is to refresh your application state whenever a change is detected:

```rust
use nmrs::NetworkManager;
use std::sync::Arc;
use tokio::sync::Notify;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    let notify = Arc::new(Notify::new());

    // Monitor for changes
    let notify_clone = notify.clone();
    let nm_clone = nm.clone();
    tokio::spawn(async move {
        let _ = nm_clone.monitor_network_changes(move || {
            notify_clone.notify_one();
        }).await;
    });

    // React to changes
    loop {
        notify.notified().await;

        let networks = nm.list_networks().await?;
        println!("Updated: {} networks visible", networks.len());
    }
}
```

## What Triggers Each Monitor

| Monitor | Triggers |
|---------|----------|
| `monitor_network_changes` | Access point added, access point removed, signal strength change |
| `monitor_device_changes` | Device state change (connected, disconnected, etc.), cable plug/unplug |

## Next Steps

- [Device Management](./devices.md) – understand device states
- [WiFi Management](./wifi.md) – scan and connect to networks
- [Error Handling](./error-handling.md) – handle monitoring errors
