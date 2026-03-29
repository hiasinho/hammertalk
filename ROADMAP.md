# ROADMAP

## v2.0.0

- **Streaming transcription while recording**: Use VAD-based chunked transcription (`VadChunked` from `transcribe-rs` 0.3.x) to transcribe and type sentences incrementally while the hotkey is held, instead of waiting for release. Requires upgrading `transcribe-rs`, moving transcription to a background thread, and replacing the batch transcription flow with a continuous audio feed loop.
- **macOS menu bar item**: Native menu bar icon showing the current daemon state (idle, recording, transcribing) with controls for starting/stopping the daemon and configuring settings.
