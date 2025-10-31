# nmrs

Wayland-native Rust frontend for NetworkManager. Provides a GTK4 UI and a D-Bus proxy core, built in Rust.

## Note for tiling window managers (Hyprland, Sway, i3, etc.)
If the window opens tiled by default, add a rule to float it manually. For example, in Hyprland:

 ```
 windowrulev2 = float, class:^(org\.netrs\.ui)$
 ```
 Adjust class if your compositor reports a different one via `hyprctl clients`.
