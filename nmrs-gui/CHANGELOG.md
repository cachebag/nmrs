# Changelog

All notable changes to the `nmrs-gui` crate will be documented in this file.

## [Unreleased]

## [1.1.0] - 2025-12-19

### Fixed
- Corrected binary name for `.desktop` file + `postInstall` hook for Nix flake ([#146](https://github.com/cachebag/nmrs/pull/146))

## [0.5.0-beta] - 2025-12-15

### Added
- Full support for Ethernet devices ([#88](https://github.com/cachebag/nmrs/issues/88))

### Fixed
- Fixed UI freeze when connecting/forgetting networks
- Supply option to provide cert paths for WPA-EAP connections ([#56](https://github.com/cachebag/nmrs/issues/56))

## [0.4.0-beta] - 2025-12-11

### Breaking Changes
- Renamed crate from `nmrs-ui` to `nmrs-gui`

### Added
- Pre-defined themes (Catppuccin, Dracula, Gruvbox, Nord, Tokyo) ([#106](https://github.com/cachebag/nmrs/issues/106))
- `--version` flag with build hash extraction ([#108](https://github.com/cachebag/nmrs/issues/108))

### Fixed
- Re-aligned refresh button ([#111](https://github.com/cachebag/nmrs/issues/111))
- Show connection status when connecting with saved credentials ([#61](https://github.com/cachebag/nmrs/issues/61))

## [0.3.0-beta] - 2025-12-08

### Fixed
- Fixed UI not freezing on connections ([#101](https://github.com/cachebag/nmrs/pull/101))
- Fixed separate `ScrolledWindow` for each stack child ([#103](https://github.com/cachebag/nmrs/pull/103))
- Dropped deps that aren't needed for now ([#104](https://github.com/cachebag/nmrs/pull/104))

### Added
- Expose system default theme toggle (light/dark) ([#102](https://github.com/cachebag/nmrs/pull/102))

## [0.2.0-beta] - 2025-12-03

### Added
- Write `.css` file for user by default ([#58](https://github.com/cachebag/nmrs/pull/58))
- Config: Nix installation deps ([#60](https://github.com/cachebag/nmrs/pull/60))
- Visual indication on successful connection ([#64](https://github.com/cachebag/nmrs/pull/64))
- Refactored `network.rs` and `network_page.rs` to follow best practices ([#66](https://github.com/cachebag/nmrs/pull/90))

## [0.1.1-beta] - 2025-11-21

### Added
- Exposed layout to allow users to place a `style.css` in `~/.config/nmrs/` for custom styling ([#55](https://github.com/cachebag/nmrs/pull/55))
- Styling section with example CSS for customization in README

## [0.1.0-beta] - 2025-11-20

### Added
- Initial BETA release of nmrs GUI
- GTK4-based user interface
- Basic and advanced network information pages
- Persistent saved-connection state tracking
- Refresh button for manual network scanning ([#51](https://github.com/cachebag/nmrs/pull/51))
- `.desktop` file for launcher integration
- AUR package support (via `yay` and `paru`)
- Initial smoke tests ([#48](https://github.com/cachebag/nmrs/pull/48))
- Issue templates for bug reports and feature requests
- Contributing guidelines (CONTRIBUTING.md)
- Installation support for Arch Linux, Debian/Ubuntu, and from source

### Fixed
- UI enhancements for connected devices
- Password authentication failure handling
- `refresh_networks` helper for proper scanning ([#50](https://github.com/cachebag/nmrs/pull/50))
- Adjusted polling to accurately display networks on launch
- `forget_btn` thread locks
- Context correction for network settings

### Documentation
- Initial README with installation and usage instructions

[Unreleased]: https://github.com/cachebag/nmrs/compare/gui-v1.1.0...HEAD
[1.1.0]: https://github.com/cachebag/nmrs/compare/gui-v0.5.0-beta...gui-v1.1.0
[0.5.0-beta]: https://github.com/cachebag/nmrs/compare/v0.4.0-beta...gui-v0.5.0-beta
[0.4.0-beta]: https://github.com/cachebag/nmrs/compare/v0.3.0-beta...v0.4.0-beta
[0.3.0-beta]: https://github.com/cachebag/nmrs/compare/v0.2.0-beta...v0.3.0-beta
[0.2.0-beta]: https://github.com/cachebag/nmrs/compare/v0.1.1-beta...v0.2.0-beta
[0.1.1-beta]: https://github.com/cachebag/nmrs/compare/v0.1.0-beta...v0.1.1-beta
[0.1.0-beta]: https://github.com/cachebag/nmrs/releases/tag/v0.1.0-beta

