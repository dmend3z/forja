#!/usr/bin/env bash
set -euo pipefail

# Reads version from Cargo.toml and updates all npm package.json files.

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION=$(grep '^version' "$REPO_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

if [ -z "$VERSION" ]; then
  echo "Error: could not read version from Cargo.toml"
  exit 1
fi

echo "Syncing npm packages to version $VERSION"

PACKAGES=(
  "$REPO_ROOT/npm/cli/package.json"
  "$REPO_ROOT/npm/darwin-arm64/package.json"
  "$REPO_ROOT/npm/darwin-x64/package.json"
  "$REPO_ROOT/npm/linux-x64/package.json"
  "$REPO_ROOT/npm/linux-arm64/package.json"
)

for pkg in "${PACKAGES[@]}"; do
  if [ ! -f "$pkg" ]; then
    echo "Warning: $pkg not found, skipping"
    continue
  fi

  # Update the package's own version
  sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$pkg"

  # Update optionalDependencies versions (only in cli/package.json)
  sed -i '' "s/\"forja-darwin-arm64\": \"[^\"]*\"/\"forja-darwin-arm64\": \"$VERSION\"/" "$pkg"
  sed -i '' "s/\"forja-darwin-x64\": \"[^\"]*\"/\"forja-darwin-x64\": \"$VERSION\"/" "$pkg"
  sed -i '' "s/\"forja-linux-x64\": \"[^\"]*\"/\"forja-linux-x64\": \"$VERSION\"/" "$pkg"
  sed -i '' "s/\"forja-linux-arm64\": \"[^\"]*\"/\"forja-linux-arm64\": \"$VERSION\"/" "$pkg"

  echo "  Updated $pkg"
done

echo "Done. All packages set to $VERSION"
