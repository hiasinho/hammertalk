#!/bin/bash
# Install hammertalk on macOS
#
# This script:
# 1. Builds the release binary
# 2. Installs it to ~/.local/bin
# 3. Installs a launchd service (user agent)
# 4. Downloads the default model if missing

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_DIR="$HOME/.local/bin"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_NAME="com.hammertalk.daemon"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}==>${NC} $1"; }
success() { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}==> WARNING:${NC} $1"; }
error() { echo -e "${RED}==> ERROR:${NC} $1" >&2; }

# Check we're on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    error "This script is for macOS only. Use install.sh on Linux."
    exit 1
fi

echo ""
echo "  ╦ ╦┌─┐┌┬┐┌┬┐┌─┐┬─┐┌┬┐┌─┐┬  ┬┌─"
echo "  ╠═╣├─┤││││││├┤ ├┬┘ │ ├─┤│  ├┴┐"
echo "  ╩ ╩┴ ┴┴ ┴┴ ┴└─┘┴└─ ┴ ┴ ┴┴─┘┴ ┴"
echo "  Push-to-talk transcription for macOS"
echo ""

# Build with hotkey support
info "Building release binary (with built-in hotkey support)..."
cd "$SCRIPT_DIR"
cargo build --release --features hotkey
success "Build complete"

# Stop service if running
if launchctl list "$PLIST_NAME" &>/dev/null; then
    info "Stopping running service..."
    launchctl bootout "gui/$(id -u)/$PLIST_NAME" 2>/dev/null || true
fi

# Install binaries
info "Installing to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp target/release/hammertalk "$BIN_DIR/"
cp hammertalk-ctl "$BIN_DIR/"
chmod +x "$BIN_DIR/hammertalk" "$BIN_DIR/hammertalk-ctl"
success "Binaries installed"

# Install launchd plist
info "Installing launchd service..."
mkdir -p "$LAUNCH_AGENTS_DIR"
sed -e "s|__HAMMERTALK_BIN__|$BIN_DIR/hammertalk|g" \
    -e "s|__HOME__|$HOME|g" \
    "$SCRIPT_DIR/com.hammertalk.daemon.plist" > "$LAUNCH_AGENTS_DIR/$PLIST_NAME.plist"
success "LaunchAgent installed"

# Download model if not present
MODEL_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/hammertalk/models/parakeet-tdt-v3-int8"
if [[ ! -f "$MODEL_DIR/encoder-model.int8.onnx" ]]; then
    info "Downloading Parakeet TDT v3 int8 model (~640MB)..."
    "$SCRIPT_DIR/download-model.sh" parakeet-tdt-v3-int8
    success "Model downloaded"
else
    success "Model already present at $MODEL_DIR"
fi

echo ""
success "Installation complete!"
echo ""
echo "  Start the daemon:"
echo "    launchctl bootstrap gui/\$(id -u) ~/Library/LaunchAgents/$PLIST_NAME.plist"
echo ""
echo "  Stop the daemon:"
echo "    launchctl bootout gui/\$(id -u)/$PLIST_NAME"
echo ""
echo "  View logs:"
echo "    tail -f ~/Library/Logs/hammertalk.log"
echo ""
echo "  Control recording:"
echo "    hammertalk-ctl start    # begin recording"
echo "    hammertalk-ctl stop     # stop and transcribe"
echo ""
echo "  ${YELLOW}Important:${NC} Grant these permissions in System Settings → Privacy & Security:"
echo "    • Microphone → Terminal (or your terminal app)"
echo "    • Accessibility → Terminal (or your terminal app)"
echo ""
echo "  ${BLUE}Push-to-talk (built-in hotkey):${NC}"
echo "    Default hotkey: Fn (globe key)"
echo "    Override: hammertalk --hotkey 'Cmd+Shift+T'"
echo "    Disable:  hammertalk --hotkey none"
echo ""
echo "  ${BLUE}Alternative keybinding tools:${NC}"
echo "    • Karabiner-Elements: bind a key to run hammertalk-ctl start/stop"
echo "    • BetterTouchTool: assign a trigger to the shell commands"
echo "    • Hammerspoon: use hs.hotkey.bind() with hs.task (see README)"
echo ""
