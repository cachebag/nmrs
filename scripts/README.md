# Scripts

This directory contains utility scripts for the infra of `nmrs`.

## `bump_version.py`

Prepares a release by updating version numbers and changelog.

### Usage

```bash
python3 scripts/bump_version.py <version> <release_type> --crate <crate>
```

### Arguments

- `version`: The version number in semver format (e.g., `1.2.0`)
- `release_type`: Either `beta` or `stable`
- `--crate`: Required. Either `nmrs` or `nmrs-gui`

### Examples

```bash
# Prepare nmrs 1.2.0 stable release
python3 scripts/bump_version.py 1.2.0 stable --crate nmrs

# Prepare nmrs-gui 0.6.0 beta release
python3 scripts/bump_version.py 0.6.0 beta --crate nmrs-gui
```

### What it does

1. Updates `version` in the crate's `Cargo.toml`
2. Updates the crate's `CHANGELOG.md`:
   - Moves `[Unreleased]` section to new version section with current date
   - Adds new empty `[Unreleased]` section
   - Updates comparison links
3. For `nmrs` releases only:
   - Updates `pkgver` in `PKGBUILD`
   - Updates `version` in `package.nix`

### Notes

- Run this script on `dev` branch before creating a PR to master
- PKGBUILD checksums need to be updated after the GitHub release is created (tarball must exist)
- package.nix `cargoHash` may need manual update

---

## `extract_release_notes.py`

Extracts release notes for a specific version from a crate's CHANGELOG.md.

### Usage

```bash
python3 scripts/extract_release_notes.py <version> <release_type> --crate <crate> [output_file]
```

### Arguments

- `version`: The version number (e.g., `1.2.0`)
- `release_type`: Either `beta` or `stable`
- `--crate`: Required. Either `nmrs` or `nmrs-gui`
- `output_file`: Optional output file path (defaults to stdout)

### Examples

```bash
# Print release notes to stdout
python3 scripts/extract_release_notes.py 1.2.0 stable --crate nmrs

# Write to file (used by release workflow)
python3 scripts/extract_release_notes.py 1.2.0 stable --crate nmrs RELEASE_NOTES.md
```

---

## Post-Release: Updating Checksums

After a GitHub release is created, the tarball exists and checksums can be updated:

### PKGBUILD

```bash
# On Arch Linux
cd /path/to/nmrs
updpkgsums
makepkg --printsrcinfo > .SRCINFO
```

### package.nix

```bash
# Set cargoHash to empty string temporarily
# Then run nix-build and copy the correct hash from the error
nix-build default.nix
```

---

## Releasing Both Crates

If you need to release both `nmrs` and `nmrs-gui`:

1. Prepare both in separate commits on `dev`:
   ```bash
   python3 scripts/bump_version.py 1.2.0 stable --crate nmrs
   git commit -am "chore(nmrs): prepare 1.2.0 release"
   
   python3 scripts/bump_version.py 1.2.0 stable --crate nmrs-gui
   git commit -am "chore(nmrs-gui): prepare 1.2.0 release"
   ```

2. Open PR to master

3. After merge, create both tags:
   ```bash
   git checkout master && git pull
   git tag nmrs-v1.2.0
   git tag gui-v1.2.0
   git push origin nmrs-v1.2.0 gui-v1.2.0
   ```

4. Both releases will be created automatically
