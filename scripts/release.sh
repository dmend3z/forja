#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.2.0
#
# Bumps version in all package files, commits, tags, and pushes.
# The v* tag triggers the GitHub Actions release workflow which:
#   1. Builds Rust binaries for all platforms
#   2. Publishes platform packages to npm
#   3. Publishes forja-cli to npm
#   4. Creates a GitHub Release
#   5. Updates the Homebrew tap

VERSION="${1:-}"

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh <version>"
  echo "Example: ./scripts/release.sh 0.2.0"
  exit 1
fi

# Strip leading 'v' if provided
VERSION="${VERSION#v}"

# Validate semver format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be semver (e.g., 0.2.0)"
  exit 1
fi

TAG="v${VERSION}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Check for clean working tree (allow staged changes)
if [ -n "$(git -C "$ROOT" status --porcelain)" ]; then
  echo "Error: working tree is not clean. Commit or stash changes first."
  exit 1
fi

# Check tag doesn't already exist
if git -C "$ROOT" rev-parse "$TAG" &>/dev/null; then
  echo "Error: tag $TAG already exists"
  exit 1
fi

# Ensure we're on main
CURRENT_BRANCH="$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)"
if [ "$CURRENT_BRANCH" != "main" ]; then
  echo "Error: releases must be run from the main branch (currently on '$CURRENT_BRANCH')"
  exit 1
fi

echo "Releasing forja $TAG"
echo ""

# Bump Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" "$ROOT/Cargo.toml"

# Bump npm/cli/package.json (version + optionalDependencies)
for pkg in cli darwin-arm64 darwin-x64 linux-x64 linux-arm64; do
  sed -i '' "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" "$ROOT/npm/$pkg/package.json"
done
for dep in darwin-arm64 darwin-x64 linux-x64 linux-arm64; do
  sed -i '' "s/\"forja-${dep}\": \".*\"/\"forja-${dep}\": \"${VERSION}\"/" "$ROOT/npm/cli/package.json"
done

# Update Cargo.lock
cargo check --manifest-path "$ROOT/Cargo.toml" --quiet

echo "Bumped files:"
echo "  Cargo.toml -> $VERSION"
echo "  npm/cli/package.json -> $VERSION"
echo "  npm/darwin-arm64/package.json -> $VERSION"
echo "  npm/darwin-x64/package.json -> $VERSION"
echo "  npm/linux-x64/package.json -> $VERSION"
echo "  npm/linux-arm64/package.json -> $VERSION"
echo ""

# Commit, tag, push
git -C "$ROOT" add \
  Cargo.toml Cargo.lock \
  npm/cli/package.json \
  npm/darwin-arm64/package.json \
  npm/darwin-x64/package.json \
  npm/linux-x64/package.json \
  npm/linux-arm64/package.json

git -C "$ROOT" commit -m "chore: release ${TAG}"
git -C "$ROOT" tag "$TAG"
git -C "$ROOT" push origin main --tags

echo ""
echo "Pushed $TAG — GitHub Actions will handle the rest."
echo "Watch: https://github.com/dmend3z/forja/actions"
