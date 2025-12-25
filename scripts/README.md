# Scripts

Utility scripts for managing nmrs releases.

## `bump_version.py`

Prepares a release by updating version numbers and changelog.

### Usage

```bash
python3 scripts/bump_version.py <version> <release_type> --crate <crate>
```

### Arguments

- `version`: Version number in semver format (e.g., `1.2.0`)
- `release_type`: Either `beta` or `stable`
- `--crate`: Either `nmrs` or `nmrs-gui`

### Examples

```bash
# Prepare nmrs library 1.2.0 stable release
python3 scripts/bump_version.py 1.2.0 stable --crate nmrs

# Prepare nmrs-gui 1.1.0 stable release
python3 scripts/bump_version.py 1.1.0 stable --crate nmrs-gui
```

### What it does

1. Updates `version` in the crate's `Cargo.toml`
2. Updates the crate's `CHANGELOG.md` (moves Unreleased section to new version)

## Releasing

### nmrs (library)

```bash
# 1. Bump version and update changelog
python3 scripts/bump_version.py 1.2.0 stable --crate nmrs

# 2. Review and commit
git diff
git commit -am "chore(nmrs): prepare 1.2.0 release"

# 3. Push to master and tag
git push origin master
git tag nmrs-v1.2.0
git push origin nmrs-v1.2.0

# CI automatically publishes to crates.io
```

### nmrs-gui (binary)

```bash
# 1. Bump version and update changelog
python3 scripts/bump_version.py 1.1.0 stable --crate nmrs-gui

# 2. Review and commit
git diff
git commit -am "chore(nmrs-gui): prepare 1.1.0 release"

# 3. Push to master and tag
git push origin master
git tag gui-v1.1.0
git push origin gui-v1.1.0

# CI automatically creates GitHub release with binary

# 4. Manually update AUR in the nmrs-aur/ directory
cd nmrs-aur/
# Update PKGBUILD version and source URL
updpkgsums
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to 1.1.0"
git push
```
