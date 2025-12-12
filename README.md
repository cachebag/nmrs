[![Version](https://img.shields.io/github/v/release/cachebag/nmrs?include_prereleases&label=version&color=blue)](https://github.com/cachebag/nmrs/releases) <br>
[![CI](https://github.com/cachebag/nmrs/actions/workflows/ci.yml/badge.svg)](https://github.com/cachebag/nmrs/actions/workflows/ci.yml) <br>
[![Nix](https://github.com/cachebag/nmrs/actions/workflows/nix.yml/badge.svg)](https://github.com/cachebag/nmrs/actions/workflows/nix.yml) <br>
[![License](https://img.shields.io/github/license/cachebag/nmrs?color=red)](LICENSE) <br>

<h1 align="center">nmrs 🦀</h1>

<div align="center">
  <h3>Wayland-native frontend for NetworkManager. Provides a GTK4 UI and a D-Bus proxy core, built in Rust.</h3>
</div>

<!-- Top image -->
<p align="center">
  <img src="https://github.com/user-attachments/assets/276b448d-8a7d-4b66-9318-160b2c966571" width="100%">
</p>

<!-- Bottom row of two images using table (GitHub safe) -->
<table align="center">
<tr>
<td align="center">
<img src="https://github.com/user-attachments/assets/3e0a898f-fec6-4cec-9c22-fec819695fb2" height="420">
</td>
<td align="center">
<img src="https://github.com/user-attachments/assets/c51f40ae-f1e5-4c39-a583-bdc82c980f53" height="420">
</td>
</tr>
</table>

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
    "format-wifi":📡 "{essid}",
    "format-ethernet": "",
    "on-click": "nmrs"
  },
```

**For tiling window managers to avoid automatic tiling (Hyprland, Sway, i3, etc.)**

 ```
 windowrulev2 = float, class:^(org\.nmrs\.ui)$
 ```
 Adjust class if your compositor reports a different one via `hyprctl clients`.

#

## Styling
`nmrs` produces a default style in your configuration directory (e.g. `~/.config/nmrs/style.css`)

You can override this by editing that file.

Example:
```css
/* Global overrides */
* {
    font-family: "Inter", "Sans";
    color: #ebdbb2; /* Gruvbox light text */
}

window, .network-page {
    background: #1d2021; /* Gruvbox dark background */
}

/* Replace all labels with a distinct color */
label {
    color: #d5c4a1 !important; /* Gruvbox faded text */
}
```

See `nmrs-gui/src/style.css` for any custom widget labels I currently use.

#

This project is still in development. If you would like to contribute, please read the [contribution guide](./CONTRIBUTING.md).

## License

This project is licensed under the MIT License.  
See the [LICENSE](./LICENSE) file for details.

