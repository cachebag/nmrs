#!/usr/bin/env bash
# Installation script for git hooks

set -e

HOOK_DIR=".git/hooks"
HOOK_FILE="$HOOK_DIR/pre-commit"
SOURCE_HOOK="scripts/pre-commit"

echo "Installing git hooks for nmrs..."

# Check if .git directory exists
if [ ! -d ".git" ]; then
  echo "Error: Not in a git repository"
  exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$HOOK_DIR"

# Check if source hook exists
if [ ! -f "$SOURCE_HOOK" ]; then
  echo "Error: Hook file not found at $SOURCE_HOOK"
  echo "Make sure you're in the root of the nmrs repository"
  exit 1
fi

# Backup existing hook if it exists
if [ -f "$HOOK_FILE" ]; then
  echo "Backing up existing pre-commit hook to pre-commit.backup"
  mv "$HOOK_FILE" "$HOOK_FILE.backup"
fi

# Copy hook
cp "$SOURCE_HOOK" "$HOOK_FILE"

# Make executable
chmod +x "$HOOK_FILE"

echo "Pre-commit hook installed successfully!"
echo ""
echo "The hook will run automatically on 'git commit'"
echo "To bypass the hook, use: git commit --no-verify"
echo ""
echo "Dependencies:"
echo "  - nixpkgs-fmt: for Nix formatting"
echo "  - determinate-nixd: for auto-fixing Nix hashes"
echo ""
echo "Install with:"
echo "  nix profile install nixpkgs#nixpkgs-fmt"
echo "  nix profile install nixpkgs#determinate-nixd"
