# Changelog

All notable changes to the `mmrs` crate will be documented in this file.

## [Unreleased]

### Added

- High-level `ModemManager` entry point with modem enumeration, primary-modem
  connection helpers, SIM PIN operations, signal queries, and per-modem
  `ModemScope` support (#402).
- `ConnectionStatus` snapshot model for `ModemManager::status` and
  `ModemScope::status` (#402).
- Public model types for the ModemManager domain under `mmrs::models`:
  `Modem`, `ModemState`, `AccessTechnology`, `Sim`, `SimLockState`,
  `Bearer`, `BearerConfig`, `BearerStats`, `Ip4Config`, `IpType`,
  `ModemError`, and the `Result` alias. All public structs and enums are
  `#[non_exhaustive]`; `BearerConfig` ships with `with_*` builder methods.
