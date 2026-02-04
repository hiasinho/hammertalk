# Hammertalk

Push-to-talk transcription daemon for Sway using Moonshine.

## Install

```bash
# Download model (~106MB)
./download-model.sh

# Build and install
./install.sh
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
