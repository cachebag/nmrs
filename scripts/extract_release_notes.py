#!/usr/bin/env python3
"""
Extract release notes for a specific version from a crate's CHANGELOG.md.

This script extracts the release notes for a given version and writes them
to a file that can be used for GitHub release notes.
"""

import re
import sys
from pathlib import Path


def extract_release_notes(changelog_path: Path, version: str, release_type: str) -> str:
    """Extract release notes for a specific version."""
    content = changelog_path.read_text()
    
    # Format version tag based on release type
    if release_type == "stable":
        pattern = rf'## \[{re.escape(version)}\](.*?)(?=## \[|\Z)'
        release_name = version
    else:
        pattern = rf'## \[{re.escape(version)}-{re.escape(release_type)}\](.*?)(?=## \[|\Z)'
        release_name = f"{version}-{release_type}"
    
    match = re.search(pattern, content, re.DOTALL)
    
    if not match:
        return f"# Release {release_name}\n\nNo release notes found."
    
    notes = match.group(1).strip()
    
    # Remove the date suffix from the header if it was captured
    notes = re.sub(r'^.*?\n', '', notes, count=0)  # Keep the notes as-is
    
    return f"# Release {release_name}\n\n{notes}"


def main():
    """Main entry point."""
    if len(sys.argv) < 4 or '--crate' not in sys.argv:
        print("Usage: extract_release_notes.py <version> <release_type> --crate <crate> [output_file]")
        print()
        print("Arguments:")
        print("  version       Version number (e.g., 1.2.0)")
        print("  release_type  'beta' or 'stable'")
        print("  --crate       Required: 'nmrs' or 'nmrs-gui'")
        print("  output_file   Optional: file to write notes to (defaults to stdout)")
        print()
        print("Examples:")
        print("  python3 scripts/extract_release_notes.py 1.2.0 stable --crate nmrs")
        print("  python3 scripts/extract_release_notes.py 1.2.0 stable --crate nmrs RELEASE_NOTES.md")
        sys.exit(1)
    
    version = sys.argv[1]
    release_type = sys.argv[2]
    
    # Parse arguments
    crate = None
    output_file = None
    i = 3
    
    while i < len(sys.argv):
        if sys.argv[i] == '--crate':
            if i + 1 < len(sys.argv):
                crate = sys.argv[i + 1]
                if crate not in ['nmrs', 'nmrs-gui']:
                    print(f"✗ Invalid crate: {crate}")
                    print("Expected: 'nmrs' or 'nmrs-gui'")
                    sys.exit(1)
                i += 2
            else:
                print("✗ --crate requires a value")
                sys.exit(1)
        else:
            output_file = sys.argv[i]
            i += 1
    
    if not crate:
        print("✗ --crate is required")
        sys.exit(1)
    
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    
    # Use per-crate changelog
    changelog_path = project_root / crate / 'CHANGELOG.md'
    
    if not changelog_path.exists():
        print(f"✗ CHANGELOG.md not found at {changelog_path}")
        sys.exit(1)
    
    notes = extract_release_notes(changelog_path, version, release_type)
    
    if output_file:
        output_path = Path(output_file)
        output_path.write_text(notes)
        print(f"✓ Release notes written to {output_path}")
    else:
        print(notes)


if __name__ == '__main__':
    main()
