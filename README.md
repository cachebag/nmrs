![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)
# nmrs

Wayland-native Rust frontend for NetworkManager. Provides a GTK4 UI and a D-Bus proxy core, built in Rust.

# 

**For tiling window managers (Hyprland, Sway, i3, etc.)**

 ```
 windowrulev2 = float, class:^(org\.netrs\.ui)$
 ```
 Adjust class if your compositor reports a different one via `hyprctl clients`.
