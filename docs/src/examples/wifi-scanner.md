# Basic WiFi Scanner

This example demonstrates building a simple but complete WiFi network scanner using nmrs.

## Features

- Lists all available WiFi networks
- Shows signal strength with visual indicators
- Displays security types
- Filters by signal strength
- Auto-refreshes every few seconds

## Complete Code

```rust
use nmrs::{NetworkManager, models::Network};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    // Initialize NetworkManager
    let nm = NetworkManager::new().await?;
    
    println!("WiFi Network Scanner");
    println!("===================\n");
    
    // Scan loop
    loop {
        // Clear screen (Unix/Linux)
        print!("\x1B[2J\x1B[1;1H");
        
        // Get networks
        let mut networks = nm.list_networks().await?;
        
        // Sort by signal strength (strongest first)
        networks.sort_by(|a, b| {
            b.strength.unwrap_or(0).cmp(&a.strength.unwrap_or(0))
        });
        
        // Display header
        println!("WiFi Network Scanner - {} networks found\n", networks.len());
        println!("{:<30} {:>10} {:>8} {:<20}", 
                 "SSID", "Signal", "Band", "Security");
        println!("{}", "-".repeat(70));
        
        // Display each network
        for net in networks {
            print_network(&net);
        }
        
        println!("\n{}", "-".repeat(70));
        println!("Press Ctrl+C to exit");
        
        // Wait before next scan
        sleep(Duration::from_secs(5)).await;
    }
}

fn print_network(net: &Network) {
    let signal = net.strength.unwrap_or(0);
    let signal_bar = signal_strength_bar(signal);
    
    let band = match net.frequency {
        Some(freq) if freq > 5000 => "5GHz",
        Some(_) => "2.4GHz",
        None => "Unknown",
    };
    
    let security = match &net.security {
        nmrs::WifiSecurity::Open => "Open",
        nmrs::WifiSecurity::WpaPsk { .. } => "WPA-PSK",
        nmrs::WifiSecurity::WpaEap { .. } => "WPA-EAP",
    };
    
    println!("{:<30} {:>3}% {} {:>8} {:<20}",
             truncate_ssid(&net.ssid, 30),
             signal,
             signal_bar,
             band,
             security
    );
}

fn signal_strength_bar(strength: u8) -> String {
    let bars = match strength {
        80..=100 => "▂▄▆█",
        60..=79  => "▂▄▆▁",
        40..=59  => "▂▄▁▁",
        20..=39  => "▂▁▁▁",
        _        => "▁▁▁▁",
    };
    
    let color = match strength {
        70..=100 => "\x1b[32m", // Green
        40..=69  => "\x1b[33m", // Yellow
        _        => "\x1b[31m", // Red
    };
    
    format!("{}{}\x1b[0m", color, bars)
}

fn truncate_ssid(ssid: &str, max_len: usize) -> String {
    if ssid.len() <= max_len {
        ssid.to_string()
    } else {
        format!("{}...", &ssid[..max_len - 3])
    }
}
```

## Running the Example

Add to your `Cargo.toml`:

```toml
[dependencies]
nmrs = "2.0.0"
tokio = { version = "1", features = ["full"] }
```

Run with:

```bash
cargo run
```

## Sample Output

```
WiFi Network Scanner - 8 networks found

SSID                           Signal     Band Security            
----------------------------------------------------------------------
MyHomeNetwork                   92% ▂▄▆█  5GHz WPA-PSK            
CoffeeShop_Guest                78% ▂▄▆▁  2.4GHz Open              
Neighbor-5G                     65% ▂▄▆▁  5GHz WPA-PSK            
Corporate_WiFi                  58% ▂▄▁▁  5GHz WPA-EAP            
Guest_Network                   45% ▂▄▁▁  2.4GHz Open              
FarAwayNetwork                  22% ▂▁▁▁  2.4GHz WPA-PSK            

----------------------------------------------------------------------
Press Ctrl+C to exit
```

## Enhancements

### Filter by Signal Strength

```rust
// Only show networks with signal > 30%
let networks: Vec<_> = networks
    .into_iter()
    .filter(|n| n.strength.unwrap_or(0) > 30)
    .collect();
```

### Group by Frequency Band

```rust
let mut networks_2_4ghz = Vec::new();
let mut networks_5ghz = Vec::new();

for net in networks {
    match net.frequency {
        Some(freq) if freq > 5000 => networks_5ghz.push(net),
        Some(_) => networks_2_4ghz.push(net),
        None => {}
    }
}

println!("\n5GHz Networks:");
for net in networks_5ghz {
    print_network(&net);
}

println!("\n2.4GHz Networks:");
for net in networks_2_4ghz {
    print_network(&net);
}
```

### Add Connection Capability

```rust
use std::io::{self, Write};

// After displaying networks
print!("\nEnter number to connect (or 0 to skip): ");
io::stdout().flush()?;

let mut input = String::new();
io::stdin().read_line(&mut input)?;

if let Ok(choice) = input.trim().parse::<usize>() {
    if choice > 0 && choice <= networks.len() {
        let selected = &networks[choice - 1];
        
        // Get password if needed
        match &selected.security {
            nmrs::WifiSecurity::Open => {
                nm.connect(&selected.ssid, nmrs::WifiSecurity::Open).await?;
                println!("Connected to {}", selected.ssid);
            }
            _ => {
                print!("Enter password: ");
                io::stdout().flush()?;
                let mut password = String::new();
                io::stdin().read_line(&mut password)?;
                
                nm.connect(&selected.ssid, nmrs::WifiSecurity::WpaPsk {
                    psk: password.trim().to_string()
                }).await?;
                println!("Connected to {}", selected.ssid);
            }
        }
    }
}
```

### Export to JSON

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct NetworkExport {
    ssid: String,
    signal: u8,
    frequency: Option<u32>,
    security: String,
}

// Convert networks to exportable format
let exports: Vec<NetworkExport> = networks.iter().map(|n| {
    NetworkExport {
        ssid: n.ssid.clone(),
        signal: n.strength.unwrap_or(0),
        frequency: n.frequency,
        security: format!("{:?}", n.security),
    }
}).collect();

// Write to file
let json = serde_json::to_string_pretty(&exports)?;
std::fs::write("networks.json", json)?;
println!("Exported to networks.json");
```

## Related Examples

- [WiFi Auto-Connect](./wifi-auto-connect.md)
- [Network Monitor Dashboard](./network-monitor.md)
- [Connection Manager](./connection-manager.md)

## See Also

- [WiFi Management Guide](../guide/wifi.md)
- [Scanning Networks](../guide/wifi-scanning.md)
- [API Reference](../api/network-manager.md)
