# Waybar Integration

[Waybar](https://github.com/Alexays/Waybar) is a popular status bar for Wayland compositors. You can configure it to launch nmrs-gui when clicking the network module.

## Basic Configuration

Add to your Waybar config (`~/.config/waybar/config`):

```json
"network": {
    "on-click": "nmrs"
}
```

This launches nmrs-gui when you click the network module in Waybar.

## Full Network Module Example

```json
"network": {
    "format-wifi": "{icon} {essid}",
    "format-ethernet": "󰈀 {ifname}",
    "format-disconnected": "󰤭 Disconnected",
    "format-icons": ["󰤯", "󰤟", "󰤢", "󰤥", "󰤨"],
    "tooltip-format-wifi": "{essid} ({signalStrength}%)\n{ipaddr}/{cidr}",
    "tooltip-format-ethernet": "{ifname}\n{ipaddr}/{cidr}",
    "on-click": "nmrs",
    "interval": 5
}
```

## Keybinding Integration

You can also bind nmrs-gui to a keyboard shortcut.

### Hyprland

```
bind = $mainMod, N, exec, nmrs
windowrule = float, class:org.nmrs.ui
windowrule = size 400 600, class:org.nmrs.ui
windowrule = center, class:org.nmrs.ui
```

### Sway

```
bindsym $mod+n exec nmrs
for_window [app_id="org.nmrs.ui"] floating enable
for_window [app_id="org.nmrs.ui"] resize set width 400 height 600
```

### i3

```
bindsym $mod+n exec nmrs
for_window [class="org.nmrs.ui"] floating enable
```

## Single Instance Behavior

nmrs-gui enforces single-instance mode. If you click the Waybar module while nmrs-gui is already open, the second instance will exit immediately and the existing window remains. This means you can safely bind the launch command to a click handler without worrying about duplicate windows.

## Next Steps

- [Configuration](./configuration.md) — customize the interface
- [Themes](./themes.md) — change the visual theme
