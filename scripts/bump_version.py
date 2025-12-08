#!/usr/bin/env python3
"""
Version bumping script for nmrs.

This script updates version numbers across all relevant files:
- Cargo.toml files (nmrs-core, nmrs-ui)
- PKGBUILD
- package.nix
- CHANGELOG.md (moves Unreleased section to new version)
"""

import hashlib
import re
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path
from typing import Optional
from urllib.request import urlretrieve


def update_cargo_toml(file_path: Path, version: str) -> bool:
    """Update version in a Cargo.toml file."""
    try:
        content = file_path.read_text()
        pattern = r'^version\s*=\s*"[^"]*"'
        replacement = f'version = "{version}"'
        
        new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
        
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


def calculate_sha256(file_path: Path) -> str:
    """Calculate SHA256 hash of a file."""
    sha256_hash = hashlib.sha256()
    with open(file_path, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()


def update_pkgbuild_checksums(file_path: Path, version: str, release_type: str) -> bool:
    """Update SHA256 checksums in PKGBUILD by downloading the tarball."""
    try:
        content = file_path.read_text()
        
        # Find the tarball URL in source array
        # Format: "$pkgname-$pkgver.tar.gz::https://github.com/cachebag/nmrs/archive/v$pkgver-beta.tar.gz"
        tarball_url = f"https://github.com/cachebag/nmrs/archive/v{version}-{release_type}.tar.gz"
        
        print(f"  Downloading tarball from {tarball_url}...")
        
        # Download tarball to temp file
        tmp_path = None
        try:
            tmp_path = Path(tempfile.mktemp(suffix='.tar.gz'))
            urlretrieve(tarball_url, tmp_path)
            tarball_hash = calculate_sha256(tmp_path)
            print(f"  Calculated SHA256: {tarball_hash}")
        except Exception as e:
            print(f"  ⚠ Could not download tarball: {e}")
            print(f"  ⚠ Tarball may not exist yet (will be created on GitHub release)")
            print(f"  ⚠ Checksums will need to be updated after the release tag is created")
            return False
        finally:
            if tmp_path and tmp_path.exists():
                tmp_path.unlink()
        
        # Update sha256sums array - replace only the first hash (tarball)
        # Pattern: sha256sums=('hash1' 'hash2')
        # We need to replace hash1 but keep hash2
        sha256_pattern = r"(sha256sums=\(')([^']+)(')"
        match = re.search(sha256_pattern, content)
        
        if match:
            # Replace only the first hash, keep the rest of the line
            new_content = re.sub(
                sha256_pattern,
                f"\\g<1>{tarball_hash}\\g<3>",
                content,
                count=1
            )
            file_path.write_text(new_content)
            print(f"  ✓ Updated PKGBUILD sha256sums")
            return True
        else:
            print(f"  ⚠ Could not find sha256sums in PKGBUILD")
            return False
            
    except Exception as e:
        print(f"  ✗ Error updating PKGBUILD checksums: {e}")
        import traceback
        traceback.print_exc()
        return False


def update_pkgbuild(file_path: Path, version: str, release_type: str) -> bool:
    """Update PKGBUILD with new version."""
    try:
        content = file_path.read_text()
        
        content = re.sub(r'^pkgver=.*', f'pkgver={version}', content, flags=re.MULTILINE)
        
        # Update source URL - replace v$pkgver-beta with v{version}-{release_type}
        content = re.sub(
            r'v\$pkgver-beta',
            f'v{version}-{release_type}',
            content
        )

        content = re.sub(
            r'\$pkgname-\$pkgver-beta',
            f'${{pkgname}}-{version}-{release_type}',
            content
        )
        
        file_path.write_text(content)
        print(f"✓ Updated {file_path}")
        
        # Now update checksums
        update_pkgbuild_checksums(file_path, version, release_type)
        
        return True
    except Exception as e:
        print(f"✗ Error updating {file_path}: {e}")
        return False


def update_package_nix_cargohash(file_path: Path) -> bool:
    """Update cargoHash in package.nix by attempting a build with empty hash."""
    try:
        script_dir = Path(__file__).parent
        project_root = script_dir.parent
        
        print(f"  Attempting to calculate cargoHash...")
        
        # Read current content
        content = file_path.read_text()
        original_content = content
        
        # Temporarily set cargoHash to empty string
        content = re.sub(
            r'cargoHash\s*=\s*"[^"]*";',
            'cargoHash = "";',
            content
        )
        file_path.write_text(content)
        
        try:
            # Try to build - nix will fail but tell us the correct hash
            result = subprocess.run(
                ['nix-build', '--no-out-link', str(project_root / 'default.nix')],
                capture_output=True,
                text=True,
                timeout=300,
                cwd=str(project_root)
            )
            
            # Extract hash from error message
            # Nix error format: "got:    sha256-..."
            if result.returncode != 0:
                # Look for hash in stderr
                hash_match = re.search(r'got:\s+sha256-([A-Za-z0-9+/=]+)', result.stderr)
                if hash_match:
                    correct_hash = f"sha256-{hash_match.group(1)}"
                    # Restore original, then update with correct hash
                    file_path.write_text(original_content)
                    return update_cargohash_in_file(file_path, correct_hash)
                else:
                    # Check stdout too
                    hash_match = re.search(r'sha256-([A-Za-z0-9+/=]+)', result.stdout)
                    if hash_match:
                        correct_hash = f"sha256-{hash_match.group(1)}"
                        file_path.write_text(original_content)
                        return update_cargohash_in_file(file_path, correct_hash)
            
            # If build succeeded (unlikely with empty hash), restore original
            file_path.write_text(original_content)
            
        except subprocess.TimeoutExpired:
            file_path.write_text(original_content)
            print(f"  ⚠ Build timed out")
        except FileNotFoundError:
            file_path.write_text(original_content)
            print(f"  ⚠ nix-build not found (nix may not be installed)")
        except subprocess.SubprocessError as e:
            file_path.write_text(original_content)
            # Error is expected - we're looking for the hash in the error message
            pass
        
        # If we get here, we couldn't extract the hash
        print(f"  ⚠ Could not automatically calculate cargoHash")
        print(f"  ⚠ You may need to update it manually:")
        print(f"  ⚠   1. Set cargoHash = \"\"; in package.nix")
        print(f"  ⚠   2. Run: nix-build default.nix")
        print(f"  ⚠   3. Copy the hash from the error message (look for 'got: sha256-...')")
        return False
        
    except Exception as e:
        # Make sure we restore original content on any error
        try:
            if 'original_content' in locals():
                file_path.write_text(original_content)
        except:
            pass
        print(f"  ✗ Error updating cargoHash: {e}")
        return False


def update_cargohash_in_file(file_path: Path, correct_hash: str) -> bool:
    """Update the cargoHash value in package.nix."""
    try:
        content = file_path.read_text()
        pattern = r'cargoHash\s*=\s*"[^"]*";'
        replacement = f'cargoHash = "{correct_hash}";'
        
        new_content = re.sub(pattern, replacement, content)
        file_path.write_text(new_content)
        print(f"  ✓ Updated cargoHash to {correct_hash}")
        return True
    except Exception as e:
        print(f"  ✗ Error updating cargoHash in file: {e}")
        return False


def update_package_nix(file_path: Path, version: str, release_type: str) -> bool:
    """Update package.nix with new version."""
    try:
        content = file_path.read_text()
        
        pattern = r'version\s*=\s*"[^"]*";'
        replacement = f'version = "{version}-{release_type}";'
        
        new_content = re.sub(pattern, replacement, content)
        
        if new_content != content:
            file_path.write_text(new_content)
            print(f"✓ Updated {file_path}")
            
            # Try to update cargoHash (may fail if nix tools not available)
            update_package_nix_cargohash(file_path)
            
            return True
        else:
            print(f"⚠ No changes needed in {file_path}")
            return False
    except Exception as e:
        print(f"✗ Error updating {file_path}: {e}")
        return False


def update_changelog(file_path: Path, version: str, release_type: str) -> bool:
    """Update CHANGELOG.md: move Unreleased to new version section."""
    try:
        content = file_path.read_text()
        
        today = datetime.now().strftime("%Y-%m-%d")
        
        unreleased_pattern = r'## \[Unreleased\](.*?)(?=## \[|\Z)'
        match = re.search(unreleased_pattern, content, re.DOTALL)
        
        if not match:
            print("⚠ No [Unreleased] section found in CHANGELOG.md")
            return False
        
        unreleased_content = match.group(1).strip()
        
        # If Unreleased section is empty, warn but continue
        if not unreleased_content or unreleased_content == "":
            print("⚠ [Unreleased] section is empty")
            unreleased_content = "\n\n(No changes documented)"
                
        new_version_section = f"## [{version}-{release_type}] - {today}\n{unreleased_content}\n\n"
        
        # Replace Unreleased with new version section and add new Unreleased
        new_unreleased_section = "## [Unreleased]\n\n"
        
        # Replace the Unreleased section
        new_content = re.sub(
            unreleased_pattern,
            new_unreleased_section + new_version_section,
            content,
            flags=re.DOTALL
        )
        
        # Update the comparison links at the bottom
        # Find the [unreleased] link and update it
        unreleased_link_pattern = r'\[unreleased\]:\s*https://github\.com/[^/]+/[^/]+/compare/v([^\.]+\.[^\.]+\.[^-]+-[^\.]+)\.\.\.HEAD'
        unreleased_link_replacement = f'[unreleased]: https://github.com/cachebag/nmrs/compare/v{version}-{release_type}...HEAD'
        new_content = re.sub(unreleased_link_pattern, unreleased_link_replacement, new_content, flags=re.IGNORECASE)
        
        # Add new version link before the unreleased link
        # Find the first version link to determine the previous version
        version_link_pattern = r'\[([^\]]+)\]:\s*https://github\.com/[^/]+/[^/]+/compare/v([^\.]+\.[^\.]+\.[^-]+-[^\.]+)\.\.\.v([^\.]+\.[^\.]+\.[^-]+-[^\.]+)'
        first_match = re.search(version_link_pattern, new_content)
        
        if first_match:
            prev_version = first_match.group(3)
            new_version_link = f'[{version}-{release_type}]: https://github.com/cachebag/nmrs/compare/v{prev_version}...v{version}-{release_type}\n'
            # Insert before the unreleased link
            new_content = re.sub(
                r'(\[unreleased\]:)',
                new_version_link + r'\1',
                new_content,
                flags=re.IGNORECASE
            )
        else:
            # Fallback: add link at the end of links section
            new_version_link = f'[{version}-{release_type}]: https://github.com/cachebag/nmrs/compare/v0.2.0-beta...v{version}-{release_type}\n'
            new_content = re.sub(
                r'(\[unreleased\]:)',
                new_version_link + r'\1',
                new_content,
                flags=re.IGNORECASE
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
    if len(sys.argv) < 3:
        print("Usage: bump_version.py <version> <release_type> [--update-checksums-only]")
        print("Example: bump_version.py 0.3.0 beta")
        print("         bump_version.py 0.3.0 beta --update-checksums-only")
        sys.exit(1)
    
    version = sys.argv[1]
    release_type = sys.argv[2]
    update_checksums_only = '--update-checksums-only' in sys.argv
    
    if not re.match(r'^\d+\.\d+\.\d+$', version):
        print(f"✗ Invalid version format: {version}")
        print("Expected format: X.Y.Z (e.g., 0.3.0)")
        sys.exit(1)
    
    if release_type not in ['beta', 'stable']:
        print(f"✗ Invalid release type: {release_type}")
        print("Expected: 'beta' or 'stable'")
        sys.exit(1)
    
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    
    if update_checksums_only:
        print(f"Updating checksums for {version}-{release_type}")
        print("=" * 50)
        success = True
        
        # Only update checksums
        pkgbuild_path = project_root / 'nmrs' / 'PKGBUILD'
        if pkgbuild_path.exists():
            update_pkgbuild_checksums(pkgbuild_path, version, release_type)
        else:
            print(f"✗ File not found: {pkgbuild_path}")
            success = False
        
        package_nix_path = project_root / 'package.nix'
        if package_nix_path.exists():
            update_package_nix_cargohash(package_nix_path)
        else:
            print(f"✗ File not found: {package_nix_path}")
            success = False
    else:
        print(f"Bumping version to {version}-{release_type}")
        print("=" * 50)
        
        success = True
        
        # Update Cargo.toml files
        for cargo_toml in ['nmrs-core/Cargo.toml', 'nmrs-ui/Cargo.toml']:
            path = project_root / cargo_toml
            if not path.exists():
                print(f"✗ File not found: {path}")
                success = False
            else:
                if not update_cargo_toml(path, version):
                    success = False
        
        # Update PKGBUILD
        pkgbuild_path = project_root / 'nmrs' / 'PKGBUILD'
        if not pkgbuild_path.exists():
            print(f"✗ File not found: {pkgbuild_path}")
            success = False
        else:
            if not update_pkgbuild(pkgbuild_path, version, release_type):
                success = False
        
        # Update package.nix
        package_nix_path = project_root / 'package.nix'
        if not package_nix_path.exists():
            print(f"✗ File not found: {package_nix_path}")
            success = False
        else:
            if not update_package_nix(package_nix_path, version, release_type):
                success = False
        
        # Update CHANGELOG.md
        changelog_path = project_root / 'CHANGELOG.md'
        if not changelog_path.exists():
            print(f"✗ File not found: {changelog_path}")
            success = False
        else:
            if not update_changelog(changelog_path, version, release_type):
                success = False
    
    print("=" * 50)
    if success:
        print(f"✓ Successfully bumped version to {version}-{release_type}")
        print("\nNote: SHA256 checksums have been automatically updated where possible.")
        print("If checksums couldn't be calculated automatically:")
        print("  - PKGBUILD: Tarball may not exist yet (will be created on GitHub release)")
        print("  - package.nix: cargoHash may need manual update if nix tools unavailable")
        print("\nNext steps:")
        print("  1. Review the changes")
        print("  2. Verify checksums are correct")
        print("  3. Commit and tag the release")
    else:
        print("✗ Some errors occurred during version bumping")
        sys.exit(1)


if __name__ == '__main__':
    main()

