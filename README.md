<p align="center">
  <img src="assets/ht1.png" alt="Hammertalk">
</p>

# Hammertalk

[![CI](https://github.com/hiasinho/hammertalk/actions/workflows/ci.yml/badge.svg)](https://github.com/hiasinho/hammertalk/actions/workflows/ci.yml)

Push-to-talk transcription daemon for Wayland (Sway, Hyprland, niri, COSMIC) with multiple engine support.

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/hiasinho/hammertalk/master/install-remote.sh | sh
```

This installs Rust (if needed), builds from source, downloads the model, and sets up the systemd service.

## Arch Linux (AUR)

```bash
yay -S hammertalk-bin    # pre-built binary
yay -S hammertalk-git    # build from source
```

After installing, download the model:
```bash
/usr/share/hammertalk/download-model.sh
```

## Manual Install

```bash
git clone https://github.com/hiasinho/hammertalk
cd hammertalk
./download-model.sh   # Download model (~106MB)
./install.sh          # Build and install
```

## Update

Re-run the quick install script to update to the latest version:

```bash
curl -fsSL https://raw.githubusercontent.com/hiasinho/hammertalk/master/install-remote.sh | sh
```

## Engines

Hammertalk supports multiple transcription engines:

| Engine | Model Size | Notes |
|--------|-----------|-------|
| `moonshine-tiny` | ~106MB | Default. Fast, good accuracy. |
| `whisper-tiny` | ~75MB | Smaller model, decent accuracy. |
| `whisper-base` | ~148MB | Better accuracy than tiny. |

Select an engine via CLI flag, environment variable, or config file (in priority order):

```bash
hammertalk --engine whisper-tiny
# or
HAMMERTALK_ENGINE=whisper-base hammertalk
```

Or set it persistently in `~/.config/hammertalk/config.toml`:

```toml
engine = "whisper-tiny"
```

Download the model for your chosen engine:

```bash
./download-model.sh whisper-tiny    # or whisper-base, moonshine-tiny, all
```

For systemd, uncomment and set `HAMMERTALK_ENGINE` in the service file.

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

### Keybindings

**Sway** (`~/.config/sway/config`):
```
bindsym --no-repeat $mod+t exec ~/.local/bin/hammertalk-ctl start
bindsym --release $mod+t exec ~/.local/bin/hammertalk-ctl stop
```

**Hyprland** (`~/.config/hypr/hyprland.conf`):
```
bind = $mainMod, t, exec, ~/.local/bin/hammertalk-ctl start
bindrt = $mainMod, t, exec, ~/.local/bin/hammertalk-ctl stop
```

**niri** (`~/.config/niri/config.kdl`):
```kdl
binds {
    Mod+T { spawn "sh" "-c" "~/.local/bin/hammertalk-ctl start"; }
    Mod+T release { spawn "sh" "-c" "~/.local/bin/hammertalk-ctl stop"; }
}
```

**COSMIC**: Use Settings → Keyboard → Shortcuts to add custom bindings for `~/.local/bin/hammertalk-ctl start` (key press) and `~/.local/bin/hammertalk-ctl stop` (key release). This one's for you, Marek. 😉

Hold the key, speak, release. Text appears at cursor.

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
cargo test          # run tests
./check.sh          # run all checks (format, clippy, tests, audit)
```
