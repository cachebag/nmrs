# Connection Manager

This example implements a basic connection manager that provides an interactive CLI for managing Wi-Fi, Ethernet, and VPN connections.

## Features

- List and scan networks
- Connect and disconnect Wi-Fi
- Manage VPN connections
- List devices and saved profiles
- Interactive menu-driven interface

## Code

```rust
use nmrs::{NetworkManager, WifiSecurity, ConnectionError};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;

    loop {
        println!("\n=== nmrs Connection Manager ===");
        println!("1. Scan networks");
        println!("2. List visible networks");
        println!("3. Connect to Wi-Fi");
        println!("4. Disconnect Wi-Fi");
        println!("5. Current connection");
        println!("6. List devices");
        println!("7. List saved connections");
        println!("8. Forget a connection");
        println!("9. List VPN connections");
        println!("0. Exit");
        print!("\nChoice: ");
        io::stdout().flush().ok();

        let choice = read_line();
        match choice.trim() {
            "1" => scan(&nm).await,
            "2" => list_networks(&nm).await,
            "3" => connect_wifi(&nm).await,
            "4" => disconnect(&nm).await,
            "5" => current(&nm).await,
            "6" => devices(&nm).await,
            "7" => saved(&nm).await,
            "8" => forget(&nm).await,
            "9" => vpns(&nm).await,
            "0" => break,
            _ => println!("Invalid choice"),
        }
    }

    Ok(())
}

async fn scan(nm: &NetworkManager) {
    println!("Scanning...");
    match nm.scan_networks(None).await {
        Ok(_) => println!("Scan complete"),
        Err(e) => eprintln!("Scan failed: {}", e),
    }
}

async fn list_networks(nm: &NetworkManager) {
    match nm.list_networks(None).await {
        Ok(networks) => {
            println!("\n{:<5} {:<30} {:>6} {:>10}",
                "#", "SSID", "Signal", "Security");
            println!("{}", "-".repeat(55));
            for (i, net) in networks.iter().enumerate() {
                let sec = if net.is_eap { "EAP" }
                    else if net.is_psk { "PSK" }
                    else { "Open" };
                println!("{:<5} {:<30} {:>5}% {:>10}",
                    i + 1, net.ssid, net.strength.unwrap_or(0), sec);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn connect_wifi(nm: &NetworkManager) {
    print!("SSID: ");
    io::stdout().flush().ok();
    let ssid = read_line();
    let ssid = ssid.trim();

    print!("Password (empty for open): ");
    io::stdout().flush().ok();
    let password = read_line();
    let password = password.trim();

    let security = if password.is_empty() {
        WifiSecurity::Open
    } else {
        WifiSecurity::WpaPsk { psk: password.into() }
    };

    println!("Connecting to '{}'...", ssid);
    match nm.connect(ssid, None, security).await {
        Ok(_) => println!("Connected!"),
        Err(ConnectionError::AuthFailed) => eprintln!("Wrong password"),
        Err(ConnectionError::NotFound) => eprintln!("Network not found"),
        Err(ConnectionError::Timeout) => eprintln!("Connection timed out"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn disconnect(nm: &NetworkManager) {
    match nm.disconnect(None).await {
        Ok(_) => println!("Disconnected"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn current(nm: &NetworkManager) {
    match nm.current_network().await {
        Ok(Some(net)) => {
            println!("Connected to: {} ({}%)",
                net.ssid, net.strength.unwrap_or(0));
        }
        Ok(None) => println!("Not connected"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn devices(nm: &NetworkManager) {
    match nm.list_devices().await {
        Ok(devices) => {
            for dev in &devices {
                println!("{:<10} {:<12} {:<15} {}",
                    dev.interface,
                    format!("{}", dev.device_type),
                    format!("{}", dev.state),
                    dev.identity.current_mac,
                );
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn saved(nm: &NetworkManager) {
    match nm.list_saved_connections().await {
        Ok(connections) => {
            for conn in &connections {
                println!("  {} ({})", conn.id, conn.connection_type);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn forget(nm: &NetworkManager) {
    print!("Connection name to forget: ");
    io::stdout().flush().ok();
    let name = read_line();
    let name = name.trim();

    match nm.forget(name).await {
        Ok(_) => println!("Forgot '{}'", name),
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn vpns(nm: &NetworkManager) {
    match nm.list_vpn_connections().await {
        Ok(vpns) => {
            if vpns.is_empty() {
                println!("No VPN connections");
            }
            for vpn in &vpns {
                println!("  {} ({:?}) — {:?}",
                    vpn.name, vpn.vpn_type, vpn.state);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    input
}
```

## Running

```bash
cargo run --example connection_manager
```

## Enhancements

- **VPN connect/disconnect:** Add menu options for VPN operations
- **Bluetooth:** Add Bluetooth device listing and connection
- **Network details:** Show `NetworkInfo` for selected networks
- **Color output:** Use a crate like `colored` for terminal formatting
- **Persistent config:** Store preferred networks in a config file
