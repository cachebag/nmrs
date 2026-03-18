# GUI Installation

## Arch Linux (AUR)

The easiest way to install on Arch Linux:

```bash
yay -S nmrs
# or
paru -S nmrs
```

## Nix/NixOS

### Nix Shell

```bash
nix-shell -p nmrs
```

### NixOS Configuration

Add to your NixOS configuration:

```nix
environment.systemPackages = with pkgs; [
  nmrs
];
```

### Nix Flake

```bash
nix run github:cachebag/nmrs
```

## From Source

### Dependencies

Install build dependencies for your distribution:

**Arch Linux:**

```bash
sudo pacman -S gtk4 libadwaita rust
```

**Ubuntu/Debian:**

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential
```

**Fedora:**

```bash
sudo dnf install gtk4-devel libadwaita-devel rust cargo
```

### Build

```bash
git clone https://github.com/cachebag/nmrs.git
cd nmrs
cargo build --release -p nmrs-gui
```

### Install

```bash
sudo cp target/release/nmrs-gui /usr/local/bin/nmrs
```

## Verification

After installation, launch nmrs-gui:

```bash
nmrs
```

The window should appear showing available Wi-Fi networks.

## System Requirements

- **Rust:** 1.94.0+ (for building from source)
- **GTK4:** 4.0+
- **NetworkManager:** running and accessible via D-Bus
- **Display:** Wayland or X11
- **Linux:** any modern distribution

## Desktop Entry

If you installed from source, you may want to create a desktop entry:

```ini
[Desktop Entry]
Name=nmrs
Comment=NetworkManager GUI
Exec=nmrs
Icon=network-wireless
Type=Application
Categories=Network;Settings;
```

Save as `~/.local/share/applications/nmrs.desktop`.

## Next Steps

- [Configuration](./configuration.md) — customize the GUI
- [Themes](./themes.md) — change the visual theme
- [Waybar Integration](./waybar.md) — launch from your status bar
