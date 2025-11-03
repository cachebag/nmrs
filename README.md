![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)
# nmrs 🦀
<div align="center">
  <h3>Wayland-native frontend for NetworkManager. Provides a GTK4 UI and a D-Bus proxy core, built in Rust.</h3>
</div>
<p align="center">
  <img width="472" height="598" alt="image" src="https://github.com/user-attachments/assets/c2a46227-df88-4e9e-b3c9-f4c259399785" />
</p>

# 

**For tiling window managers (Hyprland, Sway, i3, etc.)**

 ```
 windowrulev2 = float, class:^(org\.netrs\.ui)$
 ```
 Adjust class if your compositor reports a different one via `hyprctl clients`.

 #

 This project is still in development. If you would like to contribute, please read the contribution guide. Here's a quick setup guide to get you started:

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
