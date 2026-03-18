# GUI Configuration

nmrs-gui stores its configuration in `~/.config/nmrs/`.

## Configuration Files

| File | Purpose |
|------|---------|
| `~/.config/nmrs/theme` | Selected theme name |
| `~/.config/nmrs/style.css` | Custom CSS overrides |

## Theme Selection

The active theme is stored in `~/.config/nmrs/theme` as a plain text string:

- `gruvbox` — Gruvbox theme
- `nord` — Nord theme
- `dracula` — Dracula theme
- `catppuccin` — Catppuccin theme
- `tokyo` — Tokyo Night theme
- `light` — System default, light mode
- `dark` — System default, dark mode

You can change the theme through the GUI using the dropdown in the header bar, or by editing the file directly.

## Custom CSS

nmrs-gui creates a default `style.css` at `~/.config/nmrs/style.css` on first launch. Edit this file to customize the interface.

### CSS Loading Order

1. **Bundled stylesheet** — base styles at application priority
2. **Selected theme** — theme overrides at user priority
3. **User stylesheet** — `~/.config/nmrs/style.css` at user priority (highest)

Since user CSS loads last, it always takes precedence over themes.

### CSS Variables

Themes define CSS variables that you can override:

```css
/* Override theme colors */
:root {
    --bg-primary: #1a1b26;
    --bg-secondary: #24283b;
    --bg-tertiary: #414868;
    --text-primary: #c0caf5;
    --text-secondary: #a9b1d6;
    --text-tertiary: #565f89;
    --border-color: #3b4261;
    --border-color-hover: #545c7e;
    --accent-color: #7aa2f7;
    --success-color: #9ece6a;
    --warning-color: #e0af68;
    --error-color: #f7768e;
}
```

### Example Customizations

**Larger font:**

```css
window {
    font-size: 16px;
}
```

**Custom accent color:**

```css
:root {
    --accent-color: #ff6b6b;
}
```

**Rounded corners:**

```css
.network-row {
    border-radius: 8px;
}
```

## Tiling Window Manager Configuration

### Hyprland

Add to `~/.config/hypr/hyprland.conf`:

```
windowrule = float, class:org.nmrs.ui
windowrule = size 400 600, class:org.nmrs.ui
windowrule = center, class:org.nmrs.ui
```

### Sway

Add to `~/.config/sway/config`:

```
for_window [app_id="org.nmrs.ui"] floating enable
for_window [app_id="org.nmrs.ui"] resize set width 400 height 600
```

### i3

Add to `~/.config/i3/config`:

```
for_window [class="org.nmrs.ui"] floating enable
for_window [class="org.nmrs.ui"] resize set 400 600
```

## Signal Strength Indicators

nmrs-gui uses CSS classes for signal strength:

| Class | Signal Range |
|-------|-------------|
| `network-good` | Strong signal |
| `network-okay` | Medium signal |
| `network-poor` | Weak signal |

You can style these in your CSS:

```css
.network-good { color: var(--success-color); }
.network-okay { color: var(--warning-color); }
.network-poor { color: var(--error-color); }
```

## Application ID

The GTK application ID is `org.nmrs.ui`. Use this for window rules and desktop integration.

## Single Instance

nmrs-gui uses a file lock to ensure only one instance runs at a time. If you try to launch a second instance, it will exit silently.

## Next Steps

- [Themes](./themes.md) — explore available themes
- [Waybar Integration](./waybar.md) — launch from your status bar
