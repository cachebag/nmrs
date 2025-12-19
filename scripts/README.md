# Scripts

This directory contains utility scripts for the infra of `nmrs`.

## `bump_version.py`

Automates version bumping across all relevant files in the project.

### Usage

```bash
python3 scripts/bump_version.py <version> <release_type> [--crate <crate>] [--update-checksums-only]
```

### Arguments

- `version`: The version number in semver format (e.g., `0.3.0`)
- `release_type`: Either `beta` or `stable`
- `--crate`: Optional crate name (`nmrs` or `nmrs-gui`). If not specified, updates both crates (backward compatibility)
- `--update-checksums-only`: Optional flag to only update checksums (useful after release is created)

### Examples

```bash
# Full version bump for both crates (backward compatible)
python3 scripts/bump_version.py 0.3.0 beta

# Version bump for nmrs crate only
python3 scripts/bump_version.py 0.3.0 beta --crate nmrs

# Version bump for nmrs-gui crate only
python3 scripts/bump_version.py 0.3.0 beta --crate nmrs-gui

# Only update checksums (after GitHub release is created)
python3 scripts/bump_version.py 0.3.0 beta --crate nmrs --update-checksums-only
```

### What it does

1. Updates `version` in the specified crate's `Cargo.toml` (or both if `--crate` not specified)
2. Updates `pkgver` and source URLs in `nmrs/PKGBUILD` (only for `nmrs` releases)
3. Updates `version` in `package.nix` (only for `nmrs` releases)
4. Updates `CHANGELOG.md`:
   - Moves `[Unreleased]` section to new version section with current date
   - Adds new empty `[Unreleased]` section
   - Updates comparison links at the bottom with crate-specific tag format:
     - `nmrs` releases: `nmrs-v*` tags for stable, `nmrs-v*-beta` for beta
     - `nmrs-gui` releases: `gui-v*` tags
5. **Automatically updates checksums (only for `nmrs` releases):**
   - **PKGBUILD**: Downloads the GitHub tarball and calculates SHA256 (may fail if tarball doesn't exist yet)
   - **package.nix**: Attempts to calculate `cargoHash` using nix-build (requires nix to be installed)

### Notes

- The script validates version format (must be X.Y.Z)
- **Checksum updates:**
  - PKGBUILD checksums are calculated by downloading the tarball from GitHub
  - If the tarball doesn't exist yet (before release), the script will warn but continue
  - The release workflow automatically updates checksums after the GitHub release is created
  - package.nix `cargoHash` requires nix to be installed and may fail if nix is unavailable
- After running, review the changes before committing

## `extract_release_notes.py`

Extracts release notes for a specific version from CHANGELOG.md.

### Usage

```bash
python3 scripts/extract_release_notes.py <version> <release_type> [--crate <crate>] [output_file]
```

### Arguments

- `version`: The version number (e.g., `0.3.0`)
- `release_type`: Either `beta` or `stable`
- `--crate`: Optional crate name (`nmrs` or `nmrs-gui`) - currently used for documentation only
- `output_file`: Optional output file path (defaults to stdout)

### Examples

```bash
python3 scripts/extract_release_notes.py 0.3.0 beta RELEASE_NOTES.md
python3 scripts/extract_release_notes.py 0.3.0 beta --crate nmrs RELEASE_NOTES.md
python3 scripts/extract_release_notes.py 0.3.0 beta --crate nmrs-gui RELEASE_NOTES.md
```

### What it does

Extracts the release notes section for the specified version from CHANGELOG.md and formats it for use in GitHub releases.
