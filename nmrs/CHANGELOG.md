# Changelog

All notable changes to the `nmrs` crate will be documented in this file.

## [Unreleased]
### Added
- Builder pattern for `VpnCredentials` and `EapOptions` ([#188](https://github.com/cachebag/nmrs/issues/188))
- Bluetooth device support ([#198](https://github.com/cachebag/nmrs/pull/198))
- Input validation before any D-Bus operations ([#173](https://github.com/cachebag/nmrs/pull/173))
- CI: adjust workflow to auto-update nix hashes on PRs ([#182](https://github.com/cachebag/nmrs/pull/182))
- More helpful methods to `network_manager` facade ([#190](https://github.com/cachebag/nmrs/pull/190)) 
- Explicitly clean up signal streams to ensure unsubscription ([#197](https://github.com/cachebag/nmrs/pull/197))

### Fixed
- Better error message for empty passkeys ([#198](https://github.com/cachebag/nmrs/pull/198))
- Race condition in signal subscription ([#191](https://github.com/cachebag/nmrs/pull/191))

### Changed
- Various enums and structs marked non-exhaustive ([#198](https://github.com/cachebag/nmrs/pull/198))
- Expose `NMWiredProxy` and propogate speed through + write in field and display for BT device type ([#198](https://github.com/cachebag/nmrs/pull/198))

## [1.3.5] - 2026-01-13
### Changed
- Add `Debug` derive to `NetworkManager` ([#171](https://github.com/cachebag/nmrs/pull/171))

## [1.3.0] - 2026-01-12
### Changed
- Dedupe DBus proxy construction across connection logic ([#165](https://github.com/cachebag/nmrs/pull/165))
- Added contextual logging throughout VPN, connection, and device operations to preserve error context and improve debugging capabilities ([#168](https://github.com/cachebag/nmrs/pull/168))

### Fixed
- VPN operations no longer silently swallow D-Bus errors - now log warnings when proxy creation or method calls fail ([#168](https://github.com/cachebag/nmrs/pull/168))
- Connection cleanup operations (disconnect, deactivate, delete) now log failures instead of ignoring them ([#168](https://github.com/cachebag/nmrs/pull/168))
- VPN error mapping now attempts to extract actual connection state reasons instead of defaulting to `Unknown` ([#168](http://github.com/cachebag/nmrs/pull/168))
- MAC address retrieval errors are now logged with appropriate context ([#168](https://github.com/cachebag/nmrs/pull/168))
- Access point property retrieval failures are now logged for better diagnostics ([#168](https://github.com/cachebag/nmrs/pull/168))

## [1.2.0] - 2026-01-05
### Added
- Docker image for reproducing testing/dev environment ([#159](https://github.com/cachebag/nmrs/pull/159))

### Fixed
- Change `decode_ssid_or_empty` to return a borrowed slice instead of `String` ([#154](https://github.com/cachebag/nmrs/pull/154))

### Changed
- Condense device finding logic under one helper: `find_device_by_type` ([#158](https://github.com/cachebag/nmrs/pull/158))

## [1.1.0] - 2025-12-19

### Fixed
- Native WireGuard profile structure ([#135](https://github.com/cachebag/nmrs/issues/135))

### Added
- Added WireGuard connection example to docs ([#137](https://github.com/cachebag/nmrs/pull/137))

## [1.0.1] - 2025-12-15

### Changed
- Update docs for various structs, enums and functions ([#132](https://github.com/cachebag/nmrs/pull/132))

## [1.0.0] - 2025-12-15

### Added
- Full WireGuard VPN support ([#92](https://github.com/cachebag/nmrs/issues/92))

## [0.5.0-beta] - 2025-12-15

### Changed
- Refactored connection monitoring from polling to event-driven D-Bus signals for faster response times and lower CPU usage ([#46](https://github.com/cachebag/nmrs/issues/46))
- Replaced `tokio` with `futures-timer` for runtime-agnostic async support (fixes GTK/glib compatibility)

### Added
- `ActiveConnectionState` and `ConnectionStateReason` enums for detailed connection status tracking ([#46](https://github.com/cachebag/nmrs/issues/46))
- `monitor_network_changes()` API for real-time network list updates via D-Bus signals
- `NetworkManager` is now `Clone`
- Full support for Ethernet devices ([#88](https://github.com/cachebag/nmrs/issues/88))

### Fixed
- `forget()` now verifies device is disconnected before deleting saved connections ([#124](https://github.com/cachebag/nmrs/issues/124))
- `list_networks()` preserves security flags when deduplicating APs ([#123](https://github.com/cachebag/nmrs/issues/123))
- Fixed race condition in signal subscription where rapid state changes could be missed

## [0.4.0-beta] - 2025-12-11

### Breaking Changes
- Expanded `ConnectionError` enum with new variants (`AuthFailed`, `SupplicantConfigFailed`, `SupplicantTimeout`, `DhcpFailed`, `Timeout`, `Stuck`, `NoWifiDevice`, `WifiNotReady`, `NoSavedConnection`, `Failed(StateReason)`) - exhaustive matches will need a wildcard ([#82](https://github.com/cachebag/nmrs/issues/82))
- Return types changed from `zbus::Result<T>` to `Result<T, ConnectionError>` for structured error handling
- Renamed crate from `nmrs-core` to `nmrs`

### Added
- `StateReason` enum and `reason_to_error()` for mapping NetworkManager failure codes to typed errors ([#82](https://github.com/cachebag/nmrs/issues/82), [#85](https://github.com/cachebag/nmrs/issues/85))
- Comprehensive documentation across all modules ([#82](https://github.com/cachebag/nmrs/issues/82))
- Logging support via `log` crate facade ([#87](https://github.com/cachebag/nmrs/issues/87))

### Changed
- Decomposed `connect()` into smaller helper functions ([#81](https://github.com/cachebag/nmrs/issues/81))
- Extracted disconnect + wait logic to unified helper ([#79](https://github.com/cachebag/nmrs/issues/79))
- Unified state polling logic ([#80](https://github.com/cachebag/nmrs/issues/80))
- Eliminated network lookup duplication via shared helper function ([#83](https://github.com/cachebag/nmrs/issues/83))
- Replaced `eprintln!` with structured logging (`debug!`, `info!`, `warn!`, `error!`) ([#87](https://github.com/cachebag/nmrs/issues/87))

### Fixed
- Auth error mapping now properly distinguishes supplicant failures, DHCP errors, and timeouts ([#82](https://github.com/cachebag/nmrs/issues/82), [#85](https://github.com/cachebag/nmrs/issues/85), [#116](https://github.com/cachebag/nmrs/issues/116))
- `bitrate` property now fetches real connection speeds ([#110](https://github.com/cachebag/nmrs/issues/110))

## [0.3.0-beta] - 2025-12-08

### Added
- CI: Automated release workflow ([#105](https://github.com/cachebag/nmrs/pull/105))

## [0.2.0-beta] - 2025-12-03

### Added
- CI: Nix derivation test ([#57](https://github.com/cachebag/nmrs/pull/57))
- Prevent multiple instances from running by introducing a file lock ([#65](https://github.com/cachebag/nmrs/pull/65))
- CI+tests: Cross platform builds, API testing, unit testing and integration testing ([#95](https://github.com/cachebag/nmrs/pull/96))
- Minor refactors (see issue #77) - ([#91](https://github.com/cachebag/nmrs/pull/91))

## [0.1.1-beta] - 2025-11-21

### Added
- Added GNOME/GTK dependencies to `flake.nix` for NixOS development ([#53](https://github.com/cachebag/nmrs/pull/53))

## [0.1.0-beta] - 2025-11-20

### Added
- Initial BETA release of nmrs core library
- WPA/WPA2 network connection support
- EAP connections (initial support)
- Ability to forget previously saved networks
- Authentication-failure handling
- DBus proxy that subscribes directly to NetworkManager signals
- Nix flake for reproducible development environment ([#47](https://github.com/cachebag/nmrs/pull/47))
- Initial model/builder tests ([#48](https://github.com/cachebag/nmrs/pull/48))

### Fixed
- Network connection failure states with better error handling ([#52](https://github.com/cachebag/nmrs/pull/52))
- Network deduplication
- DBus API mismatches ([#49](https://github.com/cachebag/nmrs/pull/49))
- Saved connections handling

### Known Issues
- EAP connections default to no certificates (advanced certificate management coming in future releases)

[1.2.0]: https://github.com/cachebag/nmrs/compare/nmrs-v1.1.0...nmrs-v1.2.0
[1.3.0]: https://github.com/cachebag/nmrs/compare/nmrs-v1.2.0...nmrs-v1.3.0
[1.3.5]: https://github.com/cachebag/nmrs/compare/nmrs-v1.2.0...nmrs-v1.3.5
[Unreleased]: https://github.com/cachebag/nmrs/compare/nmrs-v1.3.5...HEAD
[1.1.0]: https://github.com/cachebag/nmrs/compare/nmrs-v1.0.1...nmrs-v1.1.0
[1.0.1]: https://github.com/cachebag/nmrs/compare/nmrs-v1.0.0...nmrs-v1.0.1
[1.0.0]: https://github.com/cachebag/nmrs/compare/v0.5.0-beta...nmrs-v1.0.0
[0.5.0-beta]: https://github.com/cachebag/nmrs/compare/v0.4.0-beta...v0.5.0-beta
[0.4.0-beta]: https://github.com/cachebag/nmrs/compare/v0.3.0-beta...v0.4.0-beta
[0.3.0-beta]: https://github.com/cachebag/nmrs/compare/v0.2.0-beta...v0.3.0-beta
[0.2.0-beta]: https://github.com/cachebag/nmrs/compare/v0.1.1-beta...v0.2.0-beta
[0.1.1-beta]: https://github.com/cachebag/nmrs/compare/v0.1.0-beta...v0.1.1-beta
[0.1.0-beta]: https://github.com/cachebag/nmrs/releases/tag/v0.1.0-beta

