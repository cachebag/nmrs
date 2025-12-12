# nmrs

[![Crates.io](https://img.shields.io/crates/v/nmrs)](https://crates.io/crates/nmrs)
[![Documentation](https://docs.rs/nmrs/badge.svg)](https://docs.rs/nmrs)
[![License](https://img.shields.io/crates/l/nmrs)](LICENSE)

Rust bindings for NetworkManager via D-Bus.

## Overview

`nmrs` provides a high-level, async API for managing Wi-Fi connections on Linux systems. It abstracts the complexity of D-Bus communication with NetworkManager, offering typed error handling and an ergonomic interface.

## Features

- **Network Operations**: Connect to WPA-PSK, WPA-EAP, and open networks
- **Discovery**: Scan for and list available access points with signal strength
- **Profile Management**: Query, create, and delete saved connection profiles
- **Status Queries**: Get current connection state, SSID, and detailed network information
- **Typed Errors**: Structured error types mapping NetworkManager state reason codes
- **Fully Async**: Built on `tokio` with `async/await` support

## Installation

```toml
[dependencies]
nmrs = "0.4"
```

## Quick Start

```rust
use nmrs::{NetworkManager, WifiSecurity};

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    
    // List networks
    let networks = nm.list_networks().await?;
    for net in networks {
        println!("{} ({}%)", net.ssid, net.strength.unwrap_or(0));
    }
    
    // Connect
    nm.connect("MyNetwork", WifiSecurity::WpaPsk {
        psk: "password".into()
    }).await?;
    
    Ok(())
}
```

## Error Handling

All operations return `Result<T, ConnectionError>` with specific error variants:

```rust
use nmrs::ConnectionError;

match nm.connect(ssid, creds).await {
    Ok(_) => println!("Connected"),
    Err(ConnectionError::AuthFailed) => eprintln!("Wrong password"),
    Err(ConnectionError::NotFound) => eprintln!("Network not in range"),
    Err(ConnectionError::Timeout) => eprintln!("Connection timed out"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Logging

This crate uses the [`log`](https://docs.rs/log) facade. Enable logging with:

```rust
env_logger::init();
```

Then run with `RUST_LOG=nmrs=debug` to see detailed logs.

## Documentation

Full API documentation is available at [docs.rs/nmrs](https://docs.rs/nmrs).

## Requirements

- Linux with NetworkManager
- D-Bus system bus access

## License

MIT
