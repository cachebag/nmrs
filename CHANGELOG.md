# Changelog
## [Unreleased]

### Fixed
- Fixed UI not freezing on connections ([#101](https://github.com/cachebag/nmrs/pull/101))
- Fixed seperate `ScrolledWindow` for each stack child ([#103](https://github.com/cachebag/nmrs/pull/103))
- Dropped deps that aren't needed for now ([#104](https://github.com/cachebag/nmrs/pull/104))

### Added
- Expose system default theme toggle (light/dark) ([#102](https://github.com/cachebag/nmrs/pull/102))
- CI: Automated release workflow ([#105](https://github.com/cachebag/nmrs/pull/105)) 

## [0.2.0-beta] - 2025-12-03

### Added
- Write `.css` file for user by default ([#58](https://github.com/cachebag/nmrs/pull/58))
- CI: Nix derivation test ([#57](https://github.com/cachebag/nmrs/pull/57))
- Config: Nix installation deps ([#60](https://github.com/cachebag/nmrs/pull/60))
- UI: Visual indication on successful connection ([#64](https://github.com/cachebag/nmrs/pull/64))
- Core: prevent multiple instances of `nmrs` from running by introducing a file lock ([#65](https://github.com/cachebag/nmrs/pull/65))
- UI(Refactor): refactored `network.rs` and `network_page.rs` to follow best practices and enhancement general functionality and perf ([#66](https://github.com/cachebag/nmrs/pull/90)) 
- CI+tests: Cross platform builds, API testing, unit testing and integration testing ([#95](https://github.com/cachebag/nmrs/pull/96))
- Core: Minor refactors (see issue #77) - ([#91](https://github.com/cachebag/nmrs/pull/91))

## [0.1.1-beta] - 2025-11-21

### Added
- Exposed layout to allow users to place a `style.css` in `~/.config/nmrs/` for custom styling ([#55](https://github.com/cachebag/nmrs/pull/55))
- Styling section with example CSS for customization in README
- Added GNOME/GTK dependencies to `flake.nix` for NixOS development ([#53](https://github.com/cachebag/nmrs/pull/53))

## [0.1.0-beta] - 2025-11-20

### Added
- Initial BETA release of nmrs - A Wayland Native NetworkManager Frontend
- WPA/WPA2 network connections
- EAP connections (initial support; currently defaults to no certificates)
- Ability to forget previously saved networks
- Authentication-failure handling with clear rejection feedback
- Basic and advanced network information pages
- Persistent saved-connection state tracking
- Refresh button for manual network scanning ([#51](https://github.com/cachebag/nmrs/pull/51))
- GTK4-based user interface (`nmrs-ui`)
- DBus proxy core (`nmrs-core`) that subscribes directly to NetworkManager signals
- `.desktop` file for launcher integration
- AUR package support (via `yay` and `paru`)
- Nix flake for reproducible development environment ([#47](https://github.com/cachebag/nmrs/pull/47))
- Initial smoke tests and model/builder tests ([#48](https://github.com/cachebag/nmrs/pull/48))
- Issue templates for bug reports and feature requests
- Contributing guidelines (CONTRIBUTING.md)
- Installation support for Arch Linux, Debian/Ubuntu, and from source

### Fixed
- Network connection failure states with better error handling ([#52](https://github.com/cachebag/nmrs/pull/52))
- Improved network deduplication
- UI enhancements for connected devices
- Password authentication failure handling
- `refresh_networks` helper for proper scanning ([#50](https://github.com/cachebag/nmrs/pull/50))
- Adjusted polling to accurately display networks on launch
- DBus API mismatches ([#49](https://github.com/cachebag/nmrs/pull/49))
- Saved connections handling
- `forget_btn` thread locks
- Context correction for network settings
- Re-introduced EAP support

### Documentation
- Initial README with installation and usage instructions
- Setup guide with contribution link
- Updated references for project structure

### Known Issues
- EAP connections default to no certificates (advanced certificate management coming in future releases)
- VPN connections planned for near future

[unreleased]: https://github.com/cachebag/nmrs/compare/v0.2.0-beta...HEAD
[0.2.0-beta]: https://github.com/cachebag/nmrs/compare/v0.1.1-beta...v0.2.0-beta
[0.1.1-beta]: https://github.com/cachebag/nmrs/compare/v0.1.0-beta...v0.1.1-beta
[0.1.0-beta]: https://github.com/cachebag/nmrs/releases/tag/v0.1.0-beta