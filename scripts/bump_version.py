#!/usr/bin/env python3
"""
Version bumping script for nmrs.

This script updates version numbers for a specific crate:
- Cargo.toml (nmrs or nmrs-gui)
- CHANGELOG.md (per-crate, in the crate's directory)
- PKGBUILD and package.nix (nmrs only)

Usage:
    python3 scripts/bump_version.py <version> <release_type> --crate <crate
"""

import re
import sys
from datetime import datetime
from pathlib import Path
from typing import Optional


def update_cargo_toml(file_path: Path, version: str) -> bool:
    """Update version in a Cargo.toml file."""
    try:
        content = file_path.read_text()
        pattern = r'^version\s*=\s*"[^"]*"'
        replacement = f'version = "{version}"'
        
        new_content = re.sub(pattern, replacement, content, count=1, flags=re.MULTILINE)
        
        if new_content != content:
            file_path.write_text(new_content)
            print(f"✓ Updated {file_path}")
            return True
        else:
            print(f"⚠ No changes needed in {file_path}")
            return False
    except Exception as e:
        print(f"✗ Error updating {file_path}: {e}")
        return False


def update_pkgbuild(file_path: Path, version: str, release_type: str) -> bool:
    """Update PKGBUILD with new version (checksums must be updated after release)."""
    try:
        content = file_path.read_text()
        
        # Update pkgver
        content = re.sub(r'^pkgver=.*', f'pkgver={version}', content, flags=re.MULTILINE)
        
        # Update source URL tag suffix
        if release_type == "stable":
            tag_suffix = ""
        else:
            tag_suffix = f"-{release_type}"
        
        # Update the tag in source URL: v$pkgver-beta -> v{version}{tag_suffix}
        content = re.sub(
            r'v\$pkgver(-beta)?',
            f'v$pkgver{tag_suffix}',
            content
        )
        
        # Update the extracted directory name
        content = re.sub(
            r'\$pkgname-\$pkgver(-beta)?',
            f'${{pkgname}}-${{pkgver}}{tag_suffix}',
            content
        )
        
        file_path.write_text(content)
        print(f"✓ Updated {file_path}")
        print(f"  ⚠ Note: SHA256 checksums need to be updated after the GitHub release is created")
        print(f"  ⚠ Run: python3 scripts/update_checksums.py {version} {release_type}")
        return True
    except Exception as e:
        print(f"✗ Error updating {file_path}: {e}")
        return False


def update_package_nix(file_path: Path, version: str, release_type: str) -> bool:
    """Update package.nix with new version (cargoHash must be updated separately)."""
    try:
        content = file_path.read_text()
        
        if release_type == "stable":
            version_str = version
        else:
            version_str = f"{version}-{release_type}"
        
        pattern = r'version\s*=\s*"[^"]*";'
        replacement = f'version = "{version_str}";'
        
        new_content = re.sub(pattern, replacement, content)
        
        if new_content != content:
            file_path.write_text(new_content)
            print(f"✓ Updated {file_path}")
            print(f"  ⚠ Note: cargoHash may need to be updated")
            print(f"  ⚠ Set cargoHash = \"\"; and run nix-build to get the correct hash")
            return True
        else:
            print(f"⚠ No changes needed in {file_path}")
            return False
    except Exception as e:
        print(f"✗ Error updating {file_path}: {e}")
        return False


def update_changelog(file_path: Path, version: str, release_type: str, crate: str) -> bool:
    """Update per-crate CHANGELOG.md: move Unreleased to new version section."""
    try:
        content = file_path.read_text()
        today = datetime.now().strftime("%Y-%m-%d")
        
        # Find the Unreleased section
        unreleased_pattern = r'## \[Unreleased\](.*?)(?=## \[|\Z)'
        match = re.search(unreleased_pattern, content, re.DOTALL)
        
        if not match:
            print(f"⚠ No [Unreleased] section found in {file_path}")
            return False
        
        unreleased_content = match.group(1).strip()
        
        if not unreleased_content:
            print(f"⚠ [Unreleased] section is empty in {file_path}")
            unreleased_content = "\n\n(No changes documented)"
        
        # Format version header
        if release_type == "stable":
            version_header = f"## [{version}] - {today}"
            version_tag = version
        else:
            version_header = f"## [{version}-{release_type}] - {today}"
            version_tag = f"{version}-{release_type}"
        
        new_version_section = f"{version_header}\n{unreleased_content}\n\n"
        new_unreleased_section = "## [Unreleased]\n\n"
        
        # Replace the Unreleased section with new version + fresh Unreleased
        new_content = re.sub(
            unreleased_pattern,
            new_unreleased_section + new_version_section,
            content,
            flags=re.DOTALL
        )
        
        # Determine git tag format
        if crate == "nmrs-gui":
            git_tag = f"gui-v{version_tag}"
        else:
            git_tag = f"nmrs-v{version_tag}"
        
        # Update the [Unreleased] comparison link
        unreleased_link_pattern = r'\[Unreleased\]:\s*https://github\.com/[^/]+/[^/]+/compare/[^\s]+\.\.\.HEAD'
        unreleased_link_replacement = f'[Unreleased]: https://github.com/cachebag/nmrs/compare/{git_tag}...HEAD'
        new_content = re.sub(unreleased_link_pattern, unreleased_link_replacement, new_content, flags=re.IGNORECASE)
        
        # Find the previous version tag to create comparison link
        existing_links = re.findall(
            r'\[([^\]]+)\]:\s*https://github\.com/[^/]+/[^/]+/compare/([^\s]+)\.\.\.([^\s]+)',
            new_content
        )
        
        prev_tag = None
        for link_text, _, curr_tag in existing_links:
            if link_text.lower() != 'unreleased':
                prev_tag = curr_tag.strip()
                break
        
        if not prev_tag:
            # Fallback to a reasonable default
            prev_tag = "v0.1.0-beta"
        
        # Create the new version link
        link_label = version if release_type == "stable" else version_tag
        new_version_link = f'[{link_label}]: https://github.com/cachebag/nmrs/compare/{prev_tag}...{git_tag}\n'
        
        # Insert before the Unreleased link
        new_content = re.sub(
            r'(\[Unreleased\]:)',
            new_version_link + r'\1',
            new_content
        )
        
        file_path.write_text(new_content)
        print(f"✓ Updated {file_path}")
        return True
    except Exception as e:
        print(f"✗ Error updating {file_path}: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    """Main entry point."""
    if len(sys.argv) < 4 or '--crate' not in sys.argv:
        print("Usage: bump_version.py <version> <release_type> --crate <crate>")
        print()
        print("Arguments:")
        print("  version       Version number (e.g., 1.2.0)")
        print("  release_type  'beta' or 'stable'")
        print("  --crate       Required: 'nmrs' or 'nmrs-gui'")
        print()
        print("Examples:")
        print("  python3 scripts/bump_version.py 1.2.0 stable --crate nmrs")
        print("  python3 scripts/bump_version.py 0.6.0 beta --crate nmrs-gui")
        print()
        print("This script should be run on the dev branch before creating a PR to master.")
        sys.exit(1)
    
    version = sys.argv[1]
    release_type = sys.argv[2]
    
    # Parse --crate argument
    crate: Optional[str] = None
    if '--crate' in sys.argv:
        crate_idx = sys.argv.index('--crate')
        if crate_idx + 1 < len(sys.argv):
            crate = sys.argv[crate_idx + 1]
    
    # Validate inputs
    if not re.match(r'^\d+\.\d+\.\d+$', version):
        print(f"✗ Invalid version format: {version}")
        print("Expected format: X.Y.Z (e.g., 1.2.0)")
        sys.exit(1)
    
    if release_type not in ['beta', 'stable']:
        print(f"✗ Invalid release type: {release_type}")
        print("Expected: 'beta' or 'stable'")
        sys.exit(1)
    
    if crate not in ['nmrs', 'nmrs-gui']:
        print(f"✗ Invalid or missing crate: {crate}")
        print("Expected: 'nmrs' or 'nmrs-gui'")
        sys.exit(1)
    
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    
    print(f"Preparing {crate} release: {version}-{release_type}")
    print("=" * 50)
    
    success = True
    
    # Update Cargo.toml for the specified crate
    cargo_toml_path = project_root / crate / 'Cargo.toml'
    if not cargo_toml_path.exists():
        print(f"✗ File not found: {cargo_toml_path}")
        success = False
    else:
        if not update_cargo_toml(cargo_toml_path, version):
            success = False
    
    # Update per-crate CHANGELOG.md
    changelog_path = project_root / crate / 'CHANGELOG.md'
    if not changelog_path.exists():
        print(f"✗ File not found: {changelog_path}")
        print(f"  Create {crate}/CHANGELOG.md with an [Unreleased] section first")
        success = False
    else:
        if not update_changelog(changelog_path, version, release_type, crate):
            success = False
    
    # For nmrs releases, also update PKGBUILD and package.nix
    if crate == "nmrs":
        pkgbuild_path = project_root / 'PKGBUILD'
        if pkgbuild_path.exists():
            if not update_pkgbuild(pkgbuild_path, version, release_type):
                success = False
        else:
            print(f"⚠ PKGBUILD not found, skipping")
        
        package_nix_path = project_root / 'package.nix'
        if package_nix_path.exists():
            if not update_package_nix(package_nix_path, version, release_type):
                success = False
        else:
            print(f"⚠ package.nix not found, skipping")
    
    print("=" * 50)
    
    if success:
        # Determine the tag that will be created
        if release_type == "stable":
            version_tag = version
        else:
            version_tag = f"{version}-{release_type}"
        
        if crate == "nmrs-gui":
            git_tag = f"gui-v{version_tag}"
        else:
            git_tag = f"nmrs-v{version_tag}"
        
        print(f"✓ Successfully prepared {crate} release {version}-{release_type}")
        print()
        print("Next steps:")
        print(f"  1. Review the changes: git diff")
        print(f"  2. Commit: git commit -am 'chore({crate}): prepare {version_tag} release'")
        print(f"  3. Push and open PR to master")
        print(f"  4. After merge, create tag: git tag {git_tag} && git push origin {git_tag}")
        print(f"  5. The release workflow will create the GitHub Release automatically")
        if crate == "nmrs":
            print()
            print("For nmrs releases, after the GitHub Release is created:")
            print(f"  6. Update PKGBUILD checksums (tarball now exists)")
            print(f"  7. Update package.nix cargoHash if needed")
    else:
        print("✗ Some errors occurred during version bumping")
        sys.exit(1)


if __name__ == '__main__':
    main()
