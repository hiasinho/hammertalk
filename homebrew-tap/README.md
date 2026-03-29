# Homebrew Tap for Hammertalk

Push-to-talk transcription daemon for macOS and Linux.

## Install

```bash
brew tap hiasinho/tap https://github.com/hiasinho/homebrew-tap
brew install hammertalk
```

## Post-install

Download a transcription model:

```bash
hammertalk-download-model                       # default: parakeet-tdt-v3-int8 (~640MB)
hammertalk-download-model parakeet-tdt-v3-int8  # smaller int8 variant (~640MB)
```

On macOS, grant **Microphone** and **Accessibility** permissions in System Settings → Privacy & Security.

## Usage

```bash
# Run with built-in hotkey (macOS)
hammertalk --hotkey "Cmd+Shift+T"

# Or run as a background service
brew services start hammertalk
```

## Update

```bash
brew update
brew upgrade hammertalk
```

## More info

See the [main repository](https://github.com/hiasinho/hammertalk) for full documentation.
