# Custom Timeouts

nmrs uses timeouts to prevent operations from hanging indefinitely. You can customize these timeouts for different network environments.

## Default Timeouts

| Timeout | Default | Purpose |
|---------|---------|---------|
| `connection_timeout` | 30 seconds | How long to wait for a connection to activate |
| `disconnect_timeout` | 10 seconds | How long to wait for a device to disconnect |

## Creating Custom Timeouts

Use `TimeoutConfig` with the builder pattern:

```rust
use nmrs::{NetworkManager, TimeoutConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let config = TimeoutConfig::new()
        .with_connection_timeout(Duration::from_secs(60))
        .with_disconnect_timeout(Duration::from_secs(20));

    let nm = NetworkManager::with_config(config).await?;

    println!("Connection timeout: {:?}", nm.timeout_config().connection_timeout);
    println!("Disconnect timeout: {:?}", nm.timeout_config().disconnect_timeout);

    Ok(())
}
```

## When to Increase Timeouts

### Enterprise Wi-Fi (WPA-EAP)

802.1X authentication involves multiple round trips to a RADIUS server and can take significantly longer than WPA-PSK:

```rust
let config = TimeoutConfig::new()
    .with_connection_timeout(Duration::from_secs(60));

let nm = NetworkManager::with_config(config).await?;
```

### Slow DHCP Servers

Some networks have slow or overloaded DHCP servers:

```rust
let config = TimeoutConfig::new()
    .with_connection_timeout(Duration::from_secs(45));
```

### VPN Connections

VPN connections through distant servers may need extra time:

```rust
let config = TimeoutConfig::new()
    .with_connection_timeout(Duration::from_secs(45));
```

## When to Decrease Timeouts

For fast-fail scenarios where you want quick feedback:

```rust
let config = TimeoutConfig::new()
    .with_connection_timeout(Duration::from_secs(10))
    .with_disconnect_timeout(Duration::from_secs(5));

let nm = NetworkManager::with_config(config).await?;
```

## Reading Current Configuration

```rust
let nm = NetworkManager::new().await?;
let config = nm.timeout_config();

println!("Connection timeout: {:?}", config.connection_timeout);
println!("Disconnect timeout: {:?}", config.disconnect_timeout);
```

## Timeout Errors

When a timeout is exceeded, nmrs returns `ConnectionError::Timeout`:

```rust
use nmrs::{NetworkManager, WifiSecurity, ConnectionError, TimeoutConfig};
use std::time::Duration;

let config = TimeoutConfig::new()
    .with_connection_timeout(Duration::from_secs(10));

let nm = NetworkManager::with_config(config).await?;

match nm.connect("SlowNetwork", WifiSecurity::Open).await {
    Ok(_) => println!("Connected!"),
    Err(ConnectionError::Timeout) => {
        eprintln!("Connection timed out — try a longer timeout");
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## What Timeouts Affect

Timeouts apply to all operations that wait for NetworkManager state transitions:

- `connect()` — Wi-Fi connection activation
- `connect_wired()` — Ethernet connection activation
- `connect_bluetooth()` — Bluetooth connection activation
- `connect_vpn()` — VPN connection activation
- `disconnect()` — Wi-Fi disconnection

The `disconnect_timeout` applies to the waiting period after requesting disconnection.

## Next Steps

- [Connection Options](./connection-options.md) – configure autoconnect behavior
- [Error Handling](../guide/error-handling.md) – handle timeout errors
