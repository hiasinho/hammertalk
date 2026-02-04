#!/bin/bash
# Install hammertalk

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_DIR="$HOME/.local/bin"
SERVICE_DIR="$HOME/.config/systemd/user"

echo "Building release binary..."
cd "$SCRIPT_DIR"
cargo build --release

# Stop service if running (binary may be locked)
if systemctl --user is-active --quiet hammertalk 2>/dev/null; then
    echo "Stopping running service..."
    systemctl --user stop hammertalk
    RESTART_SERVICE=1
fi

echo "Installing binary to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp target/release/hammertalk "$BIN_DIR/"
cp hammertalk-ctl "$BIN_DIR/"

echo "Installing systemd service..."
mkdir -p "$SERVICE_DIR"
cp hammertalk.service "$SERVICE_DIR/"
systemctl --user daemon-reload

echo "Installing git hooks..."
if [[ -d "$SCRIPT_DIR/.git" ]]; then
    cp "$SCRIPT_DIR/hooks/pre-commit" "$SCRIPT_DIR/.git/hooks/pre-commit"
    chmod +x "$SCRIPT_DIR/.git/hooks/pre-commit"
fi

# Restart service if it was running
if [[ "${RESTART_SERVICE:-}" == "1" ]]; then
    echo "Restarting service..."
    systemctl --user start hammertalk
fi

echo ""
echo "Installation complete!"
echo ""
echo "Next steps:"
echo "  1. Download the model:  ./download-model.sh"
echo "  2. Start the service:   systemctl --user start hammertalk"
echo "  3. Enable auto-start:   systemctl --user enable hammertalk"
echo "  4. Add keybindings to your compositor (see README for examples)"
