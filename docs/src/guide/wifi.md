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
    let networks = nm.list_networks().await?;
    
    // Connect to WPA-PSK network
    nm.connect("MyWiFi", WifiSecurity::WpaPsk {
        psk: "password".into()
    }).await?;
    
    // Get current connection
    if let Some(ssid) = nm.current_ssid().await {
        println!("Connected to: {}", ssid);
    }
    
    // Disconnect
    nm.disconnect().await?;
    
    Ok(())
}
```

## Security Types

nmrs supports all major WiFi security protocols:

### Open Networks

No authentication required:

```rust
nm.connect("FreeWiFi", WifiSecurity::Open).await?;
```

### WPA-PSK (Personal)

Password-based authentication:

```rust
nm.connect("HomeWiFi", WifiSecurity::WpaPsk {
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

nm.connect("CorpWiFi", WifiSecurity::WpaEap {
    opts: eap_opts
}).await?;
```

## Network Information

The `Network` struct contains detailed information about discovered networks:

```rust
pub struct Network {
    pub ssid: String,              // Network name
    pub strength: Option<u8>,      // Signal strength (0-100)
    pub security: WifiSecurity,    // Security type
    pub frequency: Option<u32>,    // Frequency in MHz
    pub hwaddress: Option<String>, // BSSID/MAC address
}
```

Example usage:

```rust
let networks = nm.list_networks().await?;

for net in networks {
    println!("SSID: {}", net.ssid);
    
    if let Some(strength) = net.strength {
        println!("  Signal: {}%", strength);
        
        if strength > 70 {
            println!("  Quality: Excellent");
        } else if strength > 50 {
            println!("  Quality: Good");
        } else {
            println!("  Quality: Weak");
        }
    }
    
    if let Some(freq) = net.frequency {
        let band = if freq > 5000 { "5GHz" } else { "2.4GHz" };
        println!("  Band: {}", band);
    }
}
```

## Connection Options

Customize connection behavior with `ConnectionOptions`:

```rust
use nmrs::{NetworkManager, WifiSecurity, ConnectionOptions};

let opts = ConnectionOptions::new(true)  // autoconnect
    .with_priority(10)                   // higher = preferred
    .with_ipv4_method("auto")            // DHCP
    .with_dns(vec!["1.1.1.1".into(), "8.8.8.8".into()]);

// Note: Advanced connection options require using builders directly
// See the Advanced Topics section for details
```

## WiFi Radio Control

Enable or disable WiFi hardware:

```rust
// Disable WiFi (airplane mode)
nm.set_wifi_enabled(false).await?;

// Enable WiFi
nm.set_wifi_enabled(true).await?;

// Check WiFi status
let enabled = nm.is_wifi_enabled().await?;
println!("WiFi is {}", if enabled { "enabled" } else { "disabled" });
```

## Network Scanning

Trigger a fresh scan:

```rust
// Request a scan (may take a few seconds)
nm.request_scan().await?;

// Wait a moment for scan to complete
tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

// Get updated results
let networks = nm.list_networks().await?;
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
if let Some(info) = nm.current_network_info().await? {
    println!("SSID: {}", info.ssid);
    println!("IP: {:?}", info.ip4_address);
    println!("Gateway: {:?}", info.gateway);
    println!("DNS: {:?}", info.dns);
}
```

## Error Handling

WiFi operations can fail for various reasons. Handle them gracefully:

```rust
use nmrs::ConnectionError;

match nm.connect("Network", WifiSecurity::WpaPsk {
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
    
    Err(ConnectionError::NoSecrets) => {
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

## Best Practices

### 1. Cache the NetworkManager Instance

```rust
// Good - reuse the same instance
let nm = NetworkManager::new().await?;
nm.list_networks().await?;
nm.connect("WiFi", WifiSecurity::Open).await?;

// Avoid - creating multiple instances
let nm1 = NetworkManager::new().await?;
nm1.list_networks().await?;
let nm2 = NetworkManager::new().await?; // Unnecessary
nm2.connect("WiFi", WifiSecurity::Open).await?;
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
match timeout(Duration::from_secs(30), nm.connect("WiFi", security)).await {
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
