# Themes

nmrs-gui ships with five themes, each with light and dark variants. Themes are selected from the dropdown in the header bar.

## Available Themes

### Gruvbox

A retro groove color scheme with warm colors. Inspired by the popular Vim color scheme.

### Nord

An arctic, north-bluish color palette. Clean and minimal with cool blue tones.

### Dracula

A dark theme with vibrant colors. Popular across many editors and terminal emulators.

### Catppuccin

A soothing pastel theme with four flavors. nmrs uses the Mocha (dark) and Latte (light) variants.

### Tokyo Night

Inspired by the lights of Tokyo at night. A clean dark theme with vibrant accents.

## Light/Dark Mode

Each theme has both light and dark variants. Toggle between them using the light/dark button in the header bar. The toggle saves your preference to `~/.config/nmrs/theme`.

You can also use the system default by setting `light` or `dark` in the theme file, which uses the GTK4 default appearance.

## Theme CSS Variables

All themes override the same set of CSS variables:

| Variable | Purpose |
|----------|---------|
| `--bg-primary` | Main background color |
| `--bg-secondary` | Secondary/card background |
| `--bg-tertiary` | Tertiary/hover background |
| `--text-primary` | Main text color |
| `--text-secondary` | Secondary text color |
| `--text-tertiary` | Muted text color |
| `--border-color` | Default border color |
| `--border-color-hover` | Hover state border color |
| `--accent-color` | Primary accent (links, active items) |
| `--success-color` | Success indicators (connected) |
| `--warning-color` | Warning indicators (weak signal) |
| `--error-color` | Error indicators (disconnected) |

## Creating a Custom Theme

You can create your own theme by overriding CSS variables in `~/.config/nmrs/style.css`:

```css
:root {
    --bg-primary: #0d1117;
    --bg-secondary: #161b22;
    --bg-tertiary: #21262d;
    --text-primary: #e6edf3;
    --text-secondary: #8b949e;
    --text-tertiary: #484f58;
    --border-color: #30363d;
    --border-color-hover: #484f58;
    --accent-color: #58a6ff;
    --success-color: #3fb950;
    --warning-color: #d29922;
    --error-color: #f85149;
}
```

Since user CSS loads after the selected theme, your overrides will always take effect.

## Theme Storage

- Theme selection: `~/.config/nmrs/theme` (plain text, e.g., `nord`)
- User CSS: `~/.config/nmrs/style.css`

## Next Steps

- [Configuration](./configuration.md) — full configuration reference
- [Waybar Integration](./waybar.md) — launch from your status bar
