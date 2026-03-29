#!/bin/bash
# Release script for hammertalk
# Usage: ./release.sh 1.0.1

set -e

VERSION="$1"

if [[ -z "$VERSION" ]]; then
    echo "Usage: ./release.sh <version>"
    echo "Example: ./release.sh 1.0.1"
    exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format X.Y.Z (e.g., 1.0.1)"
    exit 1
fi

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}==>${NC} $1"; }
success() { echo -e "${GREEN}==>${NC} $1"; }

# Check for uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    echo "Error: Uncommitted changes. Commit or stash them first."
    exit 1
fi

# Check we're on master
BRANCH=$(git branch --show-current)
if [[ "$BRANCH" != "master" ]]; then
    echo "Error: Not on master branch (currently on $BRANCH)"
    exit 1
fi

info "Releasing v$VERSION..."

# Update Cargo.toml
info "Updating Cargo.toml..."
sed -i'' -e "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Build to update Cargo.lock
cargo build --release

# Commit and tag
info "Creating commit and tag..."
git add Cargo.toml Cargo.lock
git commit -m "Release v$VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"

# Push
info "Pushing to GitHub..."
git push
git push --tags

# Create GitHub release
info "Creating GitHub release..."
gh release create "v$VERSION" \
    ./target/release/hammertalk \
    ./hammertalk-ctl \
    ./download-model.sh \
    --title "v$VERSION" \
    --generate-notes

success "GitHub release created"

# Update AUR hammertalk-bin
info "Updating AUR hammertalk-bin..."

AUR_DIR=$(mktemp -d)
trap "rm -rf $AUR_DIR" EXIT

git clone ssh://aur@aur.archlinux.org/hammertalk-bin.git "$AUR_DIR/hammertalk-bin"
cp aur-bin/* "$AUR_DIR/hammertalk-bin/"

# Update version in PKGBUILD
sed -i'' -e "s/^pkgver=.*/pkgver=$VERSION/" "$AUR_DIR/hammertalk-bin/PKGBUILD"
sed -i'' -e "s/^pkgrel=.*/pkgrel=1/" "$AUR_DIR/hammertalk-bin/PKGBUILD"

# Generate .SRCINFO (without makepkg, so this works on non-Arch systems)
cd "$AUR_DIR/hammertalk-bin"
cat > .SRCINFO <<SRCINFO
pkgbase = hammertalk-bin
	pkgdesc = Push-to-talk transcription daemon for Wayland and macOS
	pkgver = $VERSION
	pkgrel = 1
	url = https://github.com/hiasinho/hammertalk
	install = hammertalk-bin.install
	arch = x86_64
	license = MIT
	depends = ydotool
	depends = gcc-libs
	optdepends = pipewire: audio capture
	optdepends = pulseaudio: audio capture (alternative)
	provides = hammertalk
	conflicts = hammertalk
	conflicts = hammertalk-git
	source = hammertalk-${VERSION}::https://github.com/hiasinho/hammertalk/releases/download/v${VERSION}/hammertalk
	source = hammertalk-ctl-${VERSION}::https://github.com/hiasinho/hammertalk/releases/download/v${VERSION}/hammertalk-ctl
	source = download-model.sh-${VERSION}::https://github.com/hiasinho/hammertalk/releases/download/v${VERSION}/download-model.sh
	sha256sums = SKIP
	sha256sums = SKIP
	sha256sums = SKIP

pkgname = hammertalk-bin
SRCINFO

# Commit and push
git add -A
git commit -m "Update to v$VERSION"
git push

success "AUR hammertalk-bin updated"

# Update Homebrew tap
info "Updating Homebrew tap..."

TAP_DIR=$(mktemp -d)
trap "rm -rf $AUR_DIR $TAP_DIR" EXIT

git clone https://github.com/hiasinho/homebrew-tap.git "$TAP_DIR/homebrew-tap"

# Download the source tarball and compute SHA256
TARBALL_URL="https://github.com/hiasinho/hammertalk/archive/refs/tags/v${VERSION}.tar.gz"
TARBALL_PATH=$(mktemp)
curl -fSL "$TARBALL_URL" -o "$TARBALL_PATH"
SHA256=$(shasum -a 256 "$TARBALL_PATH" | awk '{print $1}')
rm -f "$TARBALL_PATH"

# Update formula
cd "$TAP_DIR/homebrew-tap"
sed -i'' -e "s|url \"https://github.com/hiasinho/hammertalk/archive/refs/tags/v.*\.tar\.gz\"|url \"$TARBALL_URL\"|" Formula/hammertalk.rb
sed -i'' -e "s/sha256 \".*\"/sha256 \"$SHA256\"/" Formula/hammertalk.rb

git add -A
git commit -m "Update hammertalk to v$VERSION"
git push

success "Homebrew tap updated"

echo ""
success "Release v$VERSION complete!"
echo ""
echo "  GitHub:   https://github.com/hiasinho/hammertalk/releases/tag/v$VERSION"
echo "  AUR:      https://aur.archlinux.org/packages/hammertalk-bin"
echo "  Homebrew: brew upgrade hammertalk"
