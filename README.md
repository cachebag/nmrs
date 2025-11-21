![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)
# nmrs 🦀
<div align="center">
  <h3>Wayland-native frontend for NetworkManager. Provides a GTK4 UI and a D-Bus proxy core, built in Rust.</h3>
</div>
<p align="center">
  <img width="1080" height="606" alt="image" src="https://github.com/user-attachments/assets/8e76a3ca-bdfc-4e7d-99ce-c7309e5f7600" />
</p>

# 

## Install
Via `yay`
```bash
yay -S nmrs
```
or `paru`
```bash
paru -S nmrs
```

**Wire into `waybar`**
```config
"network": {
    "tooltip": false,
    "format-wifi": "  {essid}",
    "format-ethernet": "",
    "on-click": "nmrs"
  },
```

**For tiling window managers to avoid automatic tiling (Hyprland, Sway, i3, etc.)**

 ```
 windowrulev2 = float, class:^(org\.netrs\.ui)$
 ```
 Adjust class if your compositor reports a different one via `hyprctl clients`.

#

## Styling
Expose your own styles by placing `style.css` in `~/.config/nmrs/`

Example:
```css
/* Global overrides */
* {
    font-family: "Inter", "Sans";
    color: #073642; /* Solarized dark teal */
}

window, .network-page {
    background: #fdf6e3; /* Solarized base3 */
}

/* Replace all labels with a distinct color */
label {
    color: #586e75 !important;
}
```

See `nmrs-ui/src/style.css` for any custom widget labels I currently use.

#

This project is still in development. If you would like to contribute, please read the [contribution guide](./CONTRIBUTING.md). Here's a quick list of setup steps to get you started:

**Requirements**
* Rust toolchain (`rustup`, `cargo`, `rustc`)
* GTK4 and libadwaita development libraries  

On **Arch Linux**:
```bash
sudo pacman -S gtk4 libadwaita base-devel
```

On Debian/Ubuntu:
```bash
sudo apt install pkg-config libglib2.0-dev libgirepository1.0-dev \
libgdk-pixbuf2.0-dev libpango1.0-dev libcairo2-dev \
libgtk-4-dev libadwaita-1-dev
```

**Clone and Build**
```bash
git clone https://github.com/cachebag/nmrs.git
cd nmrs
cargo build --release
```

**Run**
```bash
cargo run
```
