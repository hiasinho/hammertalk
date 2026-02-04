# Hammertalk

Push-to-talk transcription daemon for Wayland (Sway, Hyprland, niri, COSMIC) using Moonshine.

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/hiasinho/hammertalk/master/install-remote.sh | sh
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
