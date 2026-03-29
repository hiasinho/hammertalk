## Build & Run

- **Build**: `cargo build --release`
- **Run**: `./target/release/hammertalk`

## Validation

- **Test**: `cargo test`
- **Lint**: `cargo clippy -- -D warnings`
- **Format**: `cargo fmt --check` (fix with `cargo fmt`)
- **Audit**: `cargo audit` (requires `cargo install cargo-audit`)

Tests that modify env vars use `serial_test` to avoid race conditions.

IMPORTANT: Always run the tests before committing.

## Architecture

- Signal-based IPC: SIGUSR1 starts recording, SIGUSR2 stops and transcribes
- Audio capture via cpal (16kHz mono)
- Transcription via transcribe-rs with Moonshine engine
- Text output via ydotool
