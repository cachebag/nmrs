# Network Monitor Dashboard

This example creates a real-time network monitoring dashboard that reacts to network and device changes using D-Bus signals.

## Features

- Monitors network list changes in real-time
- Monitors device state changes
- Refreshes network list on changes
- Displays current connection status

## Code

```rust
use nmrs::NetworkManager;
use std::sync::Arc;
use tokio::sync::Notify;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    println!("=== Network Monitor Dashboard ===\n");

    // Print initial state
    print_status(&nm).await;

    // Set up change notifications
    let network_notify = Arc::new(Notify::new());
    let device_notify = Arc::new(Notify::new());

    // Monitor network changes (access points)
    let notify = network_notify.clone();
    let nm_clone = nm.clone();
    tokio::spawn(async move {
        if let Err(e) = nm_clone.monitor_network_changes(move || {
            notify.notify_one();
        }).await {
            eprintln!("Network monitor error: {}", e);
        }
    });

    // Monitor device changes (state transitions)
    let notify = device_notify.clone();
    let nm_clone = nm.clone();
    tokio::spawn(async move {
        if let Err(e) = nm_clone.monitor_device_changes(move || {
            notify.notify_one();
        }).await {
            eprintln!("Device monitor error: {}", e);
        }
    });

    // React to changes
    loop {
        tokio::select! {
            _ = network_notify.notified() => {
                println!("\n--- Network list changed ---");
                print_networks(&nm).await;
            }
            _ = device_notify.notified() => {
                println!("\n--- Device state changed ---");
                print_status(&nm).await;
            }
        }
    }
}

async fn print_status(nm: &NetworkManager) {
    // Current connection
    match nm.current_ssid().await {
        Some(ssid) => println!("Connected to: {}", ssid),
        None => println!("Not connected to Wi-Fi"),
    }

    // Wi-Fi state
    if let Ok(enabled) = nm.wifi_enabled().await {
        println!("Wi-Fi enabled: {}", enabled);
    }

    // Devices
    if let Ok(devices) = nm.list_devices().await {
        println!("\nDevices:");
        for dev in &devices {
            println!("  {} — {} [{}]", dev.interface, dev.device_type, dev.state);
        }
    }

    println!();
}

async fn print_networks(nm: &NetworkManager) {
    if let Ok(networks) = nm.list_networks().await {
        println!("Visible networks ({}):", networks.len());
        for net in &networks {
            let security = if net.is_eap {
                "EAP"
            } else if net.is_psk {
                "PSK"
            } else {
                "Open"
            };
            println!(
                "  {:30} {:>3}%  {}",
                net.ssid,
                net.strength.unwrap_or(0),
                security,
            );
        }
    }
    println!();
}
```

## Running

```bash
cargo run --example network_monitor
```

## Sample Output

```
=== Network Monitor Dashboard ===

Connected to: HomeWiFi
Wi-Fi enabled: true

Devices:
  wlan0 — Wi-Fi [Activated]
  eth0 — Ethernet [Disconnected]
  lo — Loopback [Unmanaged]

--- Network list changed ---
Visible networks (5):
  HomeWiFi                        87%  PSK
  Neighbor5G                      42%  PSK
  CafeGuest                       31%  Open
  OfficeNet                       25%  EAP
  IoT_Network                     15%  PSK

--- Device state changed ---
Connected to: HomeWiFi
Wi-Fi enabled: true

Devices:
  wlan0 — Wi-Fi [Activated]
  eth0 — Ethernet [Activated]
  lo — Loopback [Unmanaged]
```

## Enhancements

- **Debouncing:** D-Bus signals can fire rapidly. Add a debounce timer to avoid refreshing too frequently.
- **Detailed view:** Call `show_details()` on networks for channel, speed, and security info.
- **History:** Keep a log of state transitions with timestamps.
- **Alerts:** Trigger notifications when connection drops or a specific network appears.
