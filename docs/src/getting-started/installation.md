# Installation

This guide covers installation for both the **nmrs library** (for developers) and **nmrs-gui** (for end users).

## nmrs Library

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

## nmrs-gui Application

### Arch Linux (AUR)

For Arch Linux users, install from the AUR:

```bash
yay -S nmrs
# or
paru -S nmrs
```

### Nix/NixOS

Install via Nix:

```bash
nix-shell -p nmrs
```

Or add to your NixOS configuration:

```nix
environment.systemPackages = with pkgs; [
  nmrs
];
```

### From Source (GUI)

Requirements:
- Rust 1.85.1 or later
- GTK4 development libraries
- libadwaita

```bash
# Install dependencies (Arch Linux)
sudo pacman -S gtk4 libadwaita

# Install dependencies (Ubuntu/Debian)
sudo apt install libgtk-4-dev libadwaita-1-dev

# Install dependencies (Fedora)
sudo dnf install gtk4-devel libadwaita-devel

# Build and install
git clone https://github.com/cachebag/nmrs.git
cd nmrs
cargo build --release -p nmrs-gui
sudo cp target/release/nmrs-gui /usr/local/bin/nmrs
```

## System Requirements

### For the Library (nmrs)
- **Operating System**: Linux (any modern distribution)
- **Rust**: 1.78.0 or later
- **NetworkManager**: Version 1.0 or later, running and accessible via D-Bus
- **D-Bus**: System bus must be available

### For the GUI (nmrs-gui)
All of the above, plus:
- **Rust**: 1.85.1 or later
- **GTK4**: Version 4.0 or later
- **libadwaita**: For modern GNOME styling
- **Wayland** or **X11**: Display server

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

- **Library Users**: Continue to the [Quick Start](./quick-start.md) guide
- **GUI Users**: See the [GUI Configuration](../gui/configuration.md) guide
- **Having Issues?**: Check [Troubleshooting](../appendix/troubleshooting.md)
