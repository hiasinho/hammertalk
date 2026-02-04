# Hammertalk

Push-to-talk transcription daemon using Moonshine ONNX models.

## Architecture

- Signal-based IPC: SIGUSR1 starts recording, SIGUSR2 stops and transcribes
- Audio capture via cpal (16kHz mono)
- Transcription via transcribe-rs with Moonshine engine
- Text output via ydotool

## Key Files

- `src/main.rs` - daemon (~230 lines)
- `hammertalk-ctl` - shell script sends signals to daemon
- `hammertalk.service` - systemd user service
- `download-model.sh` - fetches Moonshine ONNX model

## Dependencies

transcribe-rs has version conflicts with ort/ndarray. Cargo.toml pins:
```toml
ort = "=2.0.0-rc.10"
ndarray = "=0.16.1"
```

## Model

- Location: `~/.local/share/hammertalk/models/moonshine-tiny/`
- Files: `encoder_model.onnx`, `decoder_model_merged.onnx`, `tokenizer.json`
- ONNX from: `UsefulSensors/moonshine` (onnx/merged/tiny/float/)
- Tokenizer from: `UsefulSensors/moonshine-tiny`

## Future Ideas

- Streaming transcription: `moonshine-streaming-tiny` exists (Safetensors only, no ONNX yet). transcribe-rs issue #29 tracks this.
- Alternative models: moonshine-base (larger, more accurate), language variants (tiny-uk, tiny-ja, etc.)
- Print mode: output to stdout instead of ydotool
- Audio feedback: beep on start/stop

## Related

- voxtype: similar tool using Whisper
- transcribe-rs: https://github.com/cjpais/transcribe-rs
