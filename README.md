# Hammertalk

Push-to-talk transcription daemon for Sway using Moonshine.

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/hiasinho/hammertalk/main/install-remote.sh | sh
```

This installs Rust (if needed), builds from source, downloads the model, and sets up the systemd service.

## Manual Install

```bash
git clone https://github.com/hiasinho/hammertalk
cd hammertalk
./download-model.sh   # Download model (~106MB)
./install.sh          # Build and install
```

## Usage

### Systemd (recommended)

```bash
systemctl --user start hammertalk
systemctl --user enable hammertalk  # auto-start on login
```

### Manual

```bash
~/.local/bin/hammertalk
```

### Control

```bash
hammertalk-ctl start   # begin recording
hammertalk-ctl stop    # stop and transcribe
hammertalk-ctl status  # check if running
```

### Sway keybindings

Add to `~/.config/sway/config`:

```
bindsym --no-repeat $mod+t exec ~/.local/bin/hammertalk-ctl start
bindsym --release $mod+t exec ~/.local/bin/hammertalk-ctl stop
```

Hold `$mod+t`, speak, release. Text appears at cursor.

## Requirements

- ydotool (and ydotoold running)
- PipeWire or PulseAudio

## Logs

```bash
journalctl --user -u hammertalk -f
```

## Build from source

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

11 unit tests cover path resolution, PID file management, text validation, and resample detection.
