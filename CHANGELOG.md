# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Write `.css` file for user by default ([#58](https://github.com/cachebag/nmrs/pull/58))
- CI: Nix derivation test ([#57](https://github.com/cachebag/nmrs/pull/57))
- Config: Nix installation deps ([#60](https://github.com/cachebag/nmrs/pull/60))

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

[unreleased]: https://github.com/cachebag/nmrs/compare/v0.1.1-beta...HEAD
[0.1.1-beta]: https://github.com/cachebag/nmrs/compare/v0.1.0-beta...v0.1.1-beta
[0.1.0-beta]: https://github.com/cachebag/nmrs/releases/tag/v0.1.0-beta
