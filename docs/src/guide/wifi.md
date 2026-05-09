# WiFi Management

nmrs provides comprehensive WiFi management capabilities through the `NetworkManager` API. This chapter covers all WiFi-related operations.

## Overview

WiFi management in nmrs includes:

- **Network Discovery** - Scan for available access points
- **Connection Management** - Connect, disconnect, and monitor connections
- **Security Support** - Open, WPA-PSK, WPA-EAP/Enterprise
- **Signal Monitoring** - Real-time signal strength updates
- **Profile Management** - Save and manage connection profiles
- **Advanced Features** - Hidden networks, custom DNS, static IP

## Quick Reference

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // Scan for networks
    let networks = nm.list_networks(None).await?;
    
    // Connect to WPA-PSK network
    nm.connect("MyWiFi", None, WifiSecurity::WpaPsk {
        psk: "password".into()
    }).await?;
    
    // Get current connection
    if let Some(ssid) = nm.current_ssid().await {
        println!("Connected to: {}", ssid);
    }
    
    // Disconnect
    nm.disconnect(None).await?;
    
    Ok(())
}
```

## Security Types

nmrs supports all major WiFi security protocols:

### Open Networks

No authentication required:

```rust
nm.connect("FreeWiFi", None, WifiSecurity::Open).await?;
```

### WPA-PSK (Personal)

Password-based authentication:

```rust
nm.connect("HomeWiFi", None, WifiSecurity::WpaPsk {
    psk: "your_password".into()
}).await?;
```

### WPA-EAP (Enterprise)

802.1X authentication with various methods:

```rust
use nmrs::{WifiSecurity, EapOptions, EapMethod, Phase2};

let eap_opts = EapOptions::new("user@company.com", "password")
    .with_method(EapMethod::Peap)
    .with_phase2(Phase2::Mschapv2)
    .with_domain_suffix_match("company.com");

nm.connect("CorpWiFi", None, WifiSecurity::WpaEap {
    opts: eap_opts
}).await?;
```

## Network Information

The `Network` struct contains detailed information about discovered networks
(see the [Models reference](../api/models.md#network) for the full layout):

```rust
pub struct Network {
    pub device: String,                // owning Wi-Fi interface (e.g. "wlan0")
    pub ssid: String,                  // network name
    pub bssid: Option<String>,         // BSSID of the strongest AP
    pub strength: Option<u8>,          // signal strength (0–100)
    pub frequency: Option<u32>,        // MHz
    pub secured: bool,                 // requires authentication
    pub is_psk: bool,                  // WPA-PSK
    pub is_eap: bool,                  // WPA-EAP / 802.1X
    pub is_hotspot: bool,
    pub bssids: Vec<String>,           // all merged BSSIDs (strongest first)
    pub is_active: bool,
    pub known: bool,                   // a saved profile exists for this SSID
    pub security_features: SecurityFeatures,
    // ...
}
```

Example usage:

```rust
let networks = nm.list_networks(None).await?;

for net in networks {
    println!("SSID: {}", net.ssid);

    if let Some(strength) = net.strength {
        let quality = match strength {
            70..=100 => "Excellent",
            50..=69 => "Good",
            _ => "Weak",
        };
        println!("  Signal: {}% ({})", strength, quality);
    }

    if let Some(freq) = net.frequency {
        let band = if freq > 5000 { "5GHz" } else { "2.4GHz" };
        println!("  Band: {}", band);
    }

    let kind = if net.is_eap {
        "WPA-EAP"
    } else if net.is_psk {
        "WPA-PSK"
    } else if net.secured {
        "Other (secured)"
    } else {
        "Open"
    };
    println!("  Security: {}", kind);
}
```

## Connection Options

`ConnectionOptions` controls the high-level behavior of profiles created by
[`NetworkManager`](../api/network-manager.md):

```rust
use nmrs::ConnectionOptions;

let opts = ConnectionOptions::new(true)   // autoconnect
    .with_priority(10)                    // higher = preferred
    .with_retries(3);                     // 0 means never retry, None = unlimited
```

This struct intentionally only covers the connection-management knobs
NetworkManager exposes per-profile. To configure DHCP method, manual IP
addresses, custom DNS servers, or static routes you need a builder — see
the [`ConnectionBuilder`](../api/builders.md#connectionbuilder) reference,
or use [`WifiConnectionBuilder`](../api/builders.md#wificonnectionbuilder)
for Wi-Fi-specific defaults.

## WiFi Radio Control

Enable or disable WiFi hardware:

```rust
// Disable WiFi (airplane mode)
nm.set_wireless_enabled(false).await?;

// Enable WiFi
nm.set_wireless_enabled(true).await?;

// Check WiFi status
let state = nm.wifi_state().await?;
println!("WiFi is {}", if state.enabled { "enabled" } else { "disabled" });
println!("Hardware switch is {}", if state.hardware_enabled { "on" } else { "off" });
```

## Network Scanning

Trigger a fresh scan:

```rust
// Request a scan (may take a few seconds)
nm.scan_networks(None).await?;

// Wait a moment for scan to complete
tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

// Get updated results
let networks = nm.list_networks(None).await?;
```

## Detecting Connection State

Check your current WiFi status:

```rust
// Get current SSID
if let Some(ssid) = nm.current_ssid().await {
    println!("Connected to: {}", ssid);
} else {
    println!("Not connected");
}

// Get detailed network info
if let Some(network) = nm.current_network().await? {
    println!("SSID: {}", network.ssid);
    println!("Signal: {}%", network.strength.unwrap_or(0));
}
```

## Error Handling

WiFi operations can fail for various reasons. Handle them gracefully:

```rust
use nmrs::ConnectionError;

match nm.connect("Network", None, WifiSecurity::WpaPsk {
    psk: "pass".into()
}).await {
    Ok(_) => println!("Connected!"),
    
    Err(ConnectionError::AuthFailed) => {
        eprintln!("Wrong password");
    }
    
    Err(ConnectionError::NotFound) => {
        eprintln!("Network not found - out of range?");
    }
    
    Err(ConnectionError::Timeout) => {
        eprintln!("Connection timed out");
    }
    
    Err(ConnectionError::DhcpFailed) => {
        eprintln!("Failed to get IP address");
    }
    
    Err(ConnectionError::MissingPassword) => {
        eprintln!("Missing password or credentials");
    }

    Err(e) => eprintln!("Error: {}", e),
}
```

## Real-Time Updates

Monitor WiFi networks in real-time:

```rust
use std::sync::Arc;

let nm = Arc::new(NetworkManager::new().await?);
let nm_clone = nm.clone();

nm.monitor_network_changes(move || {
    println!("Network list changed!");
    // In a GUI app, you'd trigger a UI refresh here
}).await?;

// Monitor device state (connection/disconnection)
nm.monitor_device_changes(|| {
    println!("Device state changed!");
}).await?;
```

## Related Guides

- [Scanning Networks](./wifi-scanning.md) - Detailed scanning guide
- [Connecting to Networks](./wifi-connecting.md) - Connection details
- [WPA-PSK Networks](./wifi-wpa-psk.md) - Password-protected WiFi
- [WPA-EAP (Enterprise)](./wifi-enterprise.md) - Enterprise WiFi
- [Hidden Networks](./wifi-hidden.md) - Connecting to hidden SSIDs
- [Error Handling](./error-handling.md) - Comprehensive error guide
- [Per-Device Scoping](./wifi-per-device.md) - Multi-radio, per-interface operations

## Best Practices

### 1. Cache the NetworkManager Instance

```rust
// Good - reuse the same instance
let nm = NetworkManager::new().await?;
nm.list_networks(None).await?;
nm.connect("WiFi", None, WifiSecurity::Open).await?;

// Avoid - creating multiple instances
let nm1 = NetworkManager::new().await?;
nm1.list_networks(None).await?;
let nm2 = NetworkManager::new().await?; // Unnecessary
nm2.connect("WiFi", None, WifiSecurity::Open).await?;
```

### 2. Handle Signal Strength

```rust
// Always check for None
if let Some(strength) = network.strength {
    println!("Signal: {}%", strength);
} else {
    println!("Signal: Unknown");
}
```

### 3. Use Timeouts

```rust
use tokio::time::{timeout, Duration};

// Wrap operations in timeouts
match timeout(Duration::from_secs(30), nm.connect("WiFi", None, security)).await {
    Ok(Ok(_)) => println!("Connected"),
    Ok(Err(e)) => eprintln!("Connection failed: {}", e),
    Err(_) => eprintln!("Operation timed out"),
}
```

### 4. Monitor for Disconnections

```rust
// Keep monitoring in the background
tokio::spawn(async move {
    loop {
        if nm.current_ssid().await.is_none() {
            eprintln!("Disconnected!");
            // Attempt reconnection logic
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
});
```

## Next Steps

- Learn about [VPN Connections](./vpn.md)
- Explore [Device Management](./devices.md)
- See complete [Examples](../examples/wifi-scanner.md)
