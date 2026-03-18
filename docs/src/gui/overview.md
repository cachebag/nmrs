# GUI Overview

**nmrs-gui** is a Wayland-compatible GTK4 graphical interface for NetworkManager. It provides a modern, lightweight network management UI that integrates well with tiling window managers.

## Features

- **Wi-Fi Management** — scan, connect, disconnect, and view network details
- **Ethernet Support** — view and manage wired connections
- **Enterprise Wi-Fi** — WPA-EAP/802.1X with password, username, and certificate support
- **Multiple Themes** — Gruvbox, Nord, Dracula, Catppuccin, Tokyo Night (light and dark)
- **Custom Styling** — CSS-based customization via `~/.config/nmrs/style.css`
- **Real-Time Updates** — D-Bus signal monitoring for live network and device state
- **Single Instance** — file lock ensures only one instance runs at a time
- **Wayland Native** — first-class Wayland support, also works on X11

## Architecture

```
nmrs-gui
├── Main entry (single-instance lock, GTK Application)
├── CSS loading (bundled → theme → user overrides)
├── Header
│   ├── Wi-Fi label and status
│   ├── Theme selector dropdown
│   ├── Refresh button
│   ├── Light/Dark toggle
│   └── Wi-Fi enable/disable switch
├── Network list view
│   ├── Grouped by SSID + band
│   ├── Signal strength indicators
│   └── Double-click to connect, arrow for details
├── Network details page
│   ├── SSID, status, signal, BSSID
│   ├── Frequency, channel, mode, speed
│   ├── Security type
│   └── Forget button
├── Wired device list view
│   └── Double-click to connect, arrow for details
├── Wired details page
│   ├── Interface, state, type, MAC, driver
│   └── Managed status
├── Connect modal
│   ├── Password field
│   ├── EAP username (for enterprise)
│   ├── CA certificate path
│   └── System CA checkbox
└── D-Bus monitors
    ├── Network changes (debounced refresh)
    └── Device changes (debounced refresh)
```

## Screenshots

nmrs-gui uses a clean, minimal interface that adapts to your chosen theme and color scheme.

## Requirements

- GTK4 4.0+
- Linux with NetworkManager running
- Wayland or X11 display server

## Next Steps

- [Installation](./installation.md) — install nmrs-gui
- [Configuration](./configuration.md) — customize behavior
- [Themes](./themes.md) — choose and customize themes
- [Waybar Integration](./waybar.md) — launch from your status bar
