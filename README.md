<p align="center">
  <img src="assets/ht1.png" alt="Hammertalk">
</p>

# Hammertalk

[![CI](https://github.com/hiasinho/hammertalk/actions/workflows/ci.yml/badge.svg)](https://github.com/hiasinho/hammertalk/actions/workflows/ci.yml)

Push-to-talk transcription daemon for Wayland (Sway, Hyprland, niri, COSMIC) and macOS with multiple engine support.

## Quick Install

### macOS (Homebrew)

```bash
brew tap hiasinho/tap https://github.com/hiasinho/homebrew-tap
brew install hammertalk
hammertalk-download-model  # downloads default model (~640MB)
```

### Linux

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
./download-model.sh   # Download default model (~640MB)
./install.sh          # Build and install (Linux)
./install-macos.sh    # Build and install (macOS)
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
| `moonshine-tiny` | ~106MB | Fast, good accuracy. English only. |
| `moonshine-base` | ~200MB | Better accuracy than tiny. |
| `whisper-tiny` | ~75MB | Smaller model, decent accuracy. |
| `whisper-base` | ~148MB | Better accuracy than tiny. |
| `whisper-small` | ~488MB | Good balance of speed and accuracy. |
| `whisper-medium` | ~1.5GB | High accuracy, slower. |
| `whisper-large-v3` | ~3.1GB | Best accuracy, requires more resources. |
| `whisper-large-v3-turbo` | ~1.6GB | Near large-v3 accuracy, faster. |
| `parakeet-tdt-v3` | ~2.4GB | NVIDIA NeMo, high accuracy, 25 languages. |
| `parakeet-tdt-v3-int8` | ~640MB | Default. Int8 quantized. Smaller and faster, near-full accuracy. |

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
./download-model.sh                        # default: parakeet-tdt-v3-int8
./download-model.sh whisper-tiny           # or any engine name
./download-model.sh all                    # download all models
```

For systemd, uncomment and set `HAMMERTALK_ENGINE` in the service file.

## Language

By default, Hammertalk transcribes English (`en`). You can set the language via CLI flag, environment variable, or config file (in priority order):

```bash
hammertalk --language de
# or
HAMMERTALK_LANGUAGE=fr hammertalk
```

Or in `~/.config/hammertalk/config.toml`:

```toml
language = "de"
```

Set `language = "auto"` for automatic language detection (works best with multilingual models like `whisper-large-v3`).

> **Note:** Moonshine models only support English. The `--language` option is primarily useful with Whisper models.

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

## Waybar

Add a custom module to your waybar config:

```jsonc
"custom/hammertalk": {
    "exec": "~/.local/bin/hammertalk status --follow --format json",
    "return-type": "json",
    "format": "{}",
    "tooltip": true,
    "on-click": "systemctl --user restart hammertalk",
    "on-click-right": "systemctl --user stop hammertalk"
}
```

Style by state in `~/.config/waybar/style.css`:

```css
#custom-hammertalk.recording { color: #ff5555; }
#custom-hammertalk.transcribing { color: #f1fa8c; }
#custom-hammertalk.stopped { color: #6272a4; }
```

You can also check status from the command line:

```bash
hammertalk status                       # one-shot text output
hammertalk status --follow --format json # continuous JSON stream
```

## macOS

### Install

```bash
git clone https://github.com/hiasinho/hammertalk
cd hammertalk
./install-macos.sh
```

Grant **Microphone** and **Accessibility** permissions in System Settings → Privacy & Security.

### Run manually

```bash
hammertalk
```

The default hotkey is `Fn`. Override it with:
```bash
hammertalk --hotkey "Cmd+Shift+T"
```

Disable the built-in hotkey (signal-only mode):
```bash
hammertalk --hotkey none
```

Or set it in `~/.config/hammertalk/config.toml`:
```toml
hotkey = "Cmd+Shift+T"
```

Hold the key, speak, release. Text appears at cursor.

### launchd (recommended)

```bash
# Start
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.hammertalk.daemon.plist

# Stop
launchctl bootout gui/$(id -u)/com.hammertalk.daemon

# Auto-start on login: set RunAtLoad to true in the plist
```

Configure your engine, model path, and hotkey in `~/Library/LaunchAgents/com.hammertalk.daemon.plist`.

Use `--model-path` to override the default model location:
```bash
hammertalk --engine parakeet-tdt-v3 \
  --model-path /path/to/your/model
```

## Requirements

### Linux
- ydotool (and ydotoold running)
- PipeWire or PulseAudio

### macOS
- Microphone permission
- Accessibility permission (for text input and built-in hotkey)
- Built with `--features hotkey` for push-to-talk (default hotkey: `Fn`)

## Logs

```bash
# Linux
journalctl --user -u hammertalk -f

# macOS
tail -f ~/Library/Logs/hammertalk.log
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
