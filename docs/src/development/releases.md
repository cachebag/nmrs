# Release Process

This page documents the release process for nmrs and nmrs-gui.

## Versioning

nmrs follows [Semantic Versioning](https://semver.org/):

- **Major** (X.0.0) — breaking API changes
- **Minor** (0.X.0) — new features, backward-compatible
- **Patch** (0.0.X) — bug fixes, backward-compatible

The library (`nmrs`) and GUI (`nmrs-gui`) are versioned independently.

## Current Versions

| Crate | Version |
|-------|---------|
| `nmrs` | 2.2.0 |
| `nmrs-gui` | 1.1.0 |

## Changelogs

Each crate maintains its own changelog:

- [`nmrs/CHANGELOG.md`](https://github.com/cachebag/nmrs/blob/master/nmrs/CHANGELOG.md) — Core library
- [`nmrs-gui/CHANGELOG.md`](https://github.com/cachebag/nmrs/blob/master/nmrs-gui/CHANGELOG.md) — GUI application

## Release Checklist

1. Update version in `Cargo.toml`
2. Update the changelog
3. Run `cargo test`
4. Run `cargo clippy`
5. Run `cargo fmt --check`
6. Build documentation (`mdbook build` in `docs/`)
7. Create a git tag: `git tag v2.2.0`
8. Push the tag: `git push origin v2.2.0`
9. Publish to crates.io: `cargo publish -p nmrs`

## Distribution Channels

| Channel | Package |
|---------|---------|
| [crates.io](https://crates.io/crates/nmrs) | `nmrs` library |
| [AUR](https://aur.archlinux.org/packages/nmrs) | `nmrs` (GUI binary) |
| [Nix](https://github.com/cachebag/nmrs/blob/master/flake.nix) | Nix flake |

## API Stability

- All public types are `#[non_exhaustive]` — new fields/variants can be added in minor releases
- Existing API signatures are preserved across minor releases
- Deprecated items are documented and kept for at least one minor release

## Next Steps

- [Contributing](./contributing.md) – how to contribute
- [Changelog](../appendix/changelog.md) – full version history
