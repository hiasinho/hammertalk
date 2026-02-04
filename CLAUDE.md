# Hammertalk

Push-to-talk transcription daemon using Moonshine ONNX models.

## Build & Run

- **Build**: `cargo build --release`
- **Run**: `./target/release/hammertalk`

## Validation

- **Test**: `cargo test`

Tests that modify env vars use `serial_test` to avoid race conditions.

## Architecture

- Signal-based IPC: SIGUSR1 starts recording, SIGUSR2 stops and transcribes
- Audio capture via cpal (16kHz mono)
- Transcription via transcribe-rs with Moonshine engine
- Text output via ydotool
