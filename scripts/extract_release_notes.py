#!/usr/bin/env python3
"""
Extract release notes for a specific version from CHANGELOG.md.

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
        version_tag = f"[{version}]"
        pattern = rf'## \[{re.escape(version)}\](.*?)(?=## \[|\Z)'
        release_name = f"{version}"
    else:
        version_tag = f"[{version}-{release_type}]"
        pattern = rf'## \[{re.escape(version)}-{re.escape(release_type)}\](.*?)(?=## \[|\Z)'
        release_name = f"{version}-{release_type}"
    
    match = re.search(pattern, content, re.DOTALL)
    
    if not match:
        return f"# Release {release_name}\n\nNo release notes found."
    
    notes = match.group(1).strip()
    
    # Format as markdown
    return f"# Release {release_name}\n\n{notes}"


def main():
    """Main entry point."""
    if len(sys.argv) < 3:
        print("Usage: extract_release_notes.py <version> <release_type> [output_file]")
        print("Example: extract_release_notes.py 0.3.0 beta")
        sys.exit(1)
    
    version = sys.argv[1]
    release_type = sys.argv[2]
    output_file = sys.argv[3] if len(sys.argv) > 3 else None
    
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    changelog_path = project_root / 'CHANGELOG.md'
    
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

