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
