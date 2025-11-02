![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)
# nmrs
#### Wayland-native frontend for NetworkManager. Provides a GTK4 UI and a D-Bus proxy core, built in Rust.
<img width="603" height="718" alt="image" src="https://github.com/user-attachments/assets/67f424b2-4c48-4d87-8ec2-7ae2db090048" />

# 

**For tiling window managers (Hyprland, Sway, i3, etc.)**

 ```
 windowrulev2 = float, class:^(org\.netrs\.ui)$
 ```
 Adjust class if your compositor reports a different one via `hyprctl clients`.
