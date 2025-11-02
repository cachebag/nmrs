![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)
# nmrs
#### Wayland-native frontend for NetworkManager. Provides a GTK4 UI and a D-Bus proxy core, built in Rust.
<img width="752" height="802" alt="image" src="https://github.com/user-attachments/assets/3494e88e-5cdd-4848-9fd7-b85b9a5ea2ef" />

# 

**For tiling window managers (Hyprland, Sway, i3, etc.)**

 ```
 windowrulev2 = float, class:^(org\.netrs\.ui)$
 ```
 Adjust class if your compositor reports a different one via `hyprctl clients`.
