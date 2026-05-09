# mmrs

Rust bindings for [ModemManager](https://modemmanager.org/) over D-Bus.

> **Status:** Early development. Tracking issue [#398](https://github.com/networkmanager-rs/nmrs/issues/398).

## Requirements

- Linux with ModemManager running
- Rust 1.90.0+

## Contributing

Contributions welcome — see the [contributing guide](../docs/src/development/contributing.md). In short:

- Follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) (`type(#issue): description`)
- Run `cargo +nightly fmt`, `cargo clippy -- -D warnings`, and `cargo test -p mmrs` before submitting
- Attach a relevant [issue](https://github.com/cachebag/nmrs/issues) when possible
- All tests must pass before merge

## License

Licensed under either of [MIT](../LICENSE-MIT) or [Apache-2.0](../LICENSE-APACHE), at your option.
