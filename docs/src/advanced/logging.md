# Logging and Debugging

nmrs uses the [`log`](https://docs.rs/log) crate for structured logging. You can enable logging to see what nmrs is doing internally, which is invaluable for debugging connection issues.

## Enabling Logging

nmrs produces log messages but doesn't configure a logger — that's up to your application. The simplest option is `env_logger`:

```toml
[dependencies]
nmrs = "2.2"
env_logger = "0.11"
log = "0.4"
```

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    env_logger::init();

    let nm = NetworkManager::new().await?;
    nm.scan_networks(None).await?;

    Ok(())
}
```

Run with:

```bash
RUST_LOG=nmrs=debug cargo run
```

## Log Levels

| Level | Content |
|-------|---------|
| `error` | Connection failures, D-Bus errors |
| `warn` | Unexpected states, fallback behavior |
| `info` | Connection events, state transitions |
| `debug` | D-Bus method calls, scan results, settings |
| `trace` | Raw D-Bus messages, detailed internal state |

### Level Examples

```bash
# Only errors
RUST_LOG=nmrs=error cargo run

# Info and above
RUST_LOG=nmrs=info cargo run

# Full debug output
RUST_LOG=nmrs=debug cargo run

# Everything including D-Bus internals
RUST_LOG=nmrs=trace cargo run

# Debug nmrs + info for zbus
RUST_LOG=nmrs=debug,zbus=info cargo run
```

## Debugging Connection Issues

When a connection fails, enable debug logging to see the full sequence:

```bash
RUST_LOG=nmrs=debug cargo run
```

This will show:
- Which device was selected
- Whether a saved connection was found
- The settings dictionary sent to NetworkManager
- State transitions during activation
- The specific error or reason for failure

## Debugging D-Bus Issues

If you suspect a D-Bus communication problem, enable trace logging for both nmrs and zbus:

```bash
RUST_LOG=nmrs=trace,zbus=debug cargo run
```

You can also use system tools:

```bash
# Monitor NetworkManager D-Bus traffic
sudo dbus-monitor --system "interface='org.freedesktop.NetworkManager'"

# Check NetworkManager journal logs
journalctl -u NetworkManager -f

# Check wpa_supplicant logs (for Wi-Fi auth issues)
journalctl -u wpa_supplicant -f
```

## Using with Other Loggers

nmrs works with any logger that implements the `log` facade:

### tracing (with compatibility layer)

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-log = "0.2"
```

```rust
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("nmrs=debug")
        .init();
    // ...
}
```

### simplelog

```toml
[dependencies]
simplelog = "0.12"
```

```rust
use simplelog::*;

fn main() {
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    ).unwrap();
    // ...
}
```

## Next Steps

- [D-Bus Architecture](./dbus.md) – understand the communication layer
- [Troubleshooting](../appendix/troubleshooting.md) – common issues and fixes
