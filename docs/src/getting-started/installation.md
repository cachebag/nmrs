# Installation

This guide covers installation for the **nmrs library**.

### Using Cargo

The easiest way to add nmrs to your project:

```bash
cargo add nmrs
```

Or manually add to your `Cargo.toml`:

```toml
[dependencies]
nmrs = "2.0.0"
```

### From Source

Clone and build from source:

```bash
git clone https://github.com/cachebag/nmrs.git
cd nmrs/nmrs
cargo build --release
```

### Verify Installation

Create a simple test to verify nmrs is working:

```rust
use nmrs::NetworkManager;

#[tokio::main]
async fn main() -> nmrs::Result<()> {
    let nm = NetworkManager::new().await?;
    println!("nmrs is working!");
    Ok(())
}
```

## System Requirements

- **Operating System**: Linux (any modern distribution)
- **Rust**: 1.90.0 or later
- **NetworkManager**: Version 1.0 or later, running and accessible via D-Bus
- **D-Bus**: System bus must be available

## Permissions

nmrs requires permission to manage network connections. On most systems, this is handled by PolicyKit. Ensure your user is in the appropriate groups:

```bash
# Check if you're in the network group
groups

# Add yourself to the network group if needed (requires logout/login)
sudo usermod -aG network $USER
```

## Verify NetworkManager

Ensure NetworkManager is running:

```bash
systemctl status NetworkManager
```

If it's not running:

```bash
sudo systemctl start NetworkManager
sudo systemctl enable NetworkManager  # Start on boot
```

## Next Steps

- Continue to the [Quick Start](./quick-start.md) guide
- Having issues? Check [Troubleshooting](../appendix/troubleshooting.md)
