#!/bin/bash
# Hammertalk remote installer
# Usage: curl -fsSL https://raw.githubusercontent.com/USER/hammertalk/main/install-remote.sh | sh
#
# This script:
# 1. Checks for required dependencies
# 2. Installs Rust via rustup if needed
# 3. Clones and builds hammertalk
# 4. Downloads the Moonshine model
# 5. Installs binary and systemd service

set -e

REPO_URL="https://github.com/hiasinho/hammertalk"
INSTALL_DIR="$HOME/.local/bin"
SERVICE_DIR="$HOME/.config/systemd/user"
MODEL_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hammertalk/models/moonshine-tiny"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}==>${NC} $1"; }
success() { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}==> WARNING:${NC} $1"; }
error() { echo -e "${RED}==> ERROR:${NC} $1" >&2; }

# Check if command exists
has() { command -v "$1" &>/dev/null; }

echo ""
echo "  ╦ ╦┌─┐┌┬┐┌┬┐┌─┐┬─┐┌┬┐┌─┐┬  ┬┌─"
echo "  ╠═╣├─┤││││││├┤ ├┬┘ │ ├─┤│  ├┴┐"
echo "  ╩ ╩┴ ┴┴ ┴┴ ┴└─┘┴└─ ┴ ┴ ┴┴─┘┴ ┴"
echo "  Push-to-talk transcription daemon"
echo ""

# Check for required tools
info "Checking dependencies..."

missing_deps=()

if ! has curl && ! has wget; then
    error "curl or wget required"
    exit 1
fi

if ! has git; then
    missing_deps+=("git")
fi

if ! has ydotool; then
    missing_deps+=("ydotool")
fi

# Check for audio (PipeWire or PulseAudio)
if ! has pactl && ! has pw-cli; then
    warn "Neither PipeWire nor PulseAudio detected - audio capture may not work"
fi

if [[ ${#missing_deps[@]} -gt 0 ]]; then
    warn "Missing dependencies: ${missing_deps[*]}"
    echo "    Install them with your package manager:"
    echo "      Arch:   pacman -S ${missing_deps[*]}"
    echo "      Ubuntu: apt install ${missing_deps[*]}"
    echo "      Fedora: dnf install ${missing_deps[*]}"
    echo ""
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check for Rust
if ! has cargo; then
    info "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    success "Rust installed"
else
    success "Rust found: $(rustc --version)"
fi

# Clone repository
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

info "Cloning repository..."
git clone --depth 1 "$REPO_URL" "$TEMP_DIR/hammertalk"
cd "$TEMP_DIR/hammertalk"

# Build
info "Building release binary (this may take a few minutes)..."
cargo build --release
success "Build complete"

# Install binaries
info "Installing to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
cp target/release/hammertalk "$INSTALL_DIR/"
cp hammertalk-ctl "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/hammertalk" "$INSTALL_DIR/hammertalk-ctl"
success "Binaries installed"

# Install systemd service
info "Installing systemd service..."
mkdir -p "$SERVICE_DIR"
cp hammertalk.service "$SERVICE_DIR/"
systemctl --user daemon-reload
success "Systemd service installed"

# Download model
info "Downloading Moonshine model (~106MB)..."
mkdir -p "$MODEL_DIR"
cd "$MODEL_DIR"

BASE_URL="https://huggingface.co/UsefulSensors/moonshine/resolve/main/onnx/merged/tiny"
TOKENIZER_URL="https://huggingface.co/UsefulSensors/moonshine-tiny/resolve/main/tokenizer.json"

for file in encoder_model.onnx decoder_model_merged.onnx; do
    if [[ ! -f "$file" ]]; then
        echo "    Downloading $file..."
        curl -fL "$BASE_URL/float/$file" -o "$file"
    fi
done

if [[ ! -f "tokenizer.json" ]]; then
    echo "    Downloading tokenizer.json..."
    curl -fL "$TOKENIZER_URL" -o "tokenizer.json"
fi
success "Model downloaded"

# Verify ydotoold is running
if has ydotool; then
    if ! pgrep -x ydotoold &>/dev/null; then
        warn "ydotoold is not running. Start it with: systemctl --user start ydotool"
    fi
fi

# Done!
echo ""
success "Installation complete!"
echo ""
echo "  Start the service:"
echo "    systemctl --user start hammertalk"
echo ""
echo "  Enable auto-start:"
echo "    systemctl --user enable hammertalk"
echo ""
echo "  Add to Sway config (~/.config/sway/config):"
echo "    bindsym --no-repeat \$mod+t exec ~/.local/bin/hammertalk-ctl start"
echo "    bindsym --release \$mod+t exec ~/.local/bin/hammertalk-ctl stop"
echo ""
echo "  Hold \$mod+t, speak, release. Text appears at cursor."
echo ""
