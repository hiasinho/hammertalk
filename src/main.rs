use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use log::{debug, error, info, warn};
use signal_hook::consts::{SIGUSR1, SIGUSR2, SIGTERM, SIGINT};
use signal_hook::iterator::Signals;
use transcribe_rs::engines::moonshine::{MoonshineEngine, MoonshineModelParams, ModelVariant};
use transcribe_rs::TranscriptionEngine;

static RECORDING: AtomicBool = AtomicBool::new(false);
static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

const SAMPLE_RATE: u32 = 16000;

fn get_pid_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join("hammertalk.pid")
}

fn get_model_path() -> PathBuf {
    std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share")
        })
        .join("hammertalk/models/moonshine-tiny")
}

fn write_pid_file() -> std::io::Result<()> {
    let pid_path = get_pid_path();
    let mut file = fs::File::create(&pid_path)?;
    writeln!(file, "{}", std::process::id())?;
    info!("PID file written to {:?}", pid_path);
    Ok(())
}

fn remove_pid_file() {
    let pid_path = get_pid_path();
    if let Err(e) = fs::remove_file(&pid_path) {
        warn!("Failed to remove PID file: {}", e);
    }
}

fn type_text(text: &str) {
    if text.trim().is_empty() {
        warn!("Empty transcription, skipping");
        return;
    }

    info!("Typing: {}", text);
    let result = Command::new("ydotool")
        .args(["type", "--", text])
        .status();

    match result {
        Ok(status) if status.success() => debug!("ydotool succeeded"),
        Ok(status) => warn!("ydotool exited with: {}", status),
        Err(e) => error!("Failed to run ydotool: {}", e),
    }
}

fn record_audio(buffer: Arc<Mutex<Vec<f32>>>) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("No input device available")?;

    info!("Using input device: {}", device.name()?);

    // Try to get a config close to 16kHz mono
    let supported_configs = device.supported_input_configs()?;

    let config = supported_configs
        .filter(|c| c.sample_format() == SampleFormat::F32)
        .min_by_key(|c| {
            let min = c.min_sample_rate().0;
            let max = c.max_sample_rate().0;
            if SAMPLE_RATE >= min && SAMPLE_RATE <= max {
                0
            } else {
                (SAMPLE_RATE as i32 - max as i32).abs()
            }
        })
        .ok_or("No suitable audio config")?;

    let sample_rate = if SAMPLE_RATE >= config.min_sample_rate().0
        && SAMPLE_RATE <= config.max_sample_rate().0
    {
        SAMPLE_RATE
    } else {
        config.max_sample_rate().0
    };

    let config = config.with_sample_rate(cpal::SampleRate(sample_rate));
    let channels = config.channels() as usize;

    info!(
        "Recording at {} Hz, {} channels",
        sample_rate, channels
    );

    let resample_ratio = sample_rate as f32 / SAMPLE_RATE as f32;
    let needs_resample = (resample_ratio - 1.0).abs() > 0.001;

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if RECORDING.load(Ordering::SeqCst) {
                let mut buf = buffer.lock().unwrap();

                // Convert to mono if needed and resample
                for (i, chunk) in data.chunks(channels).enumerate() {
                    let sample: f32 = chunk.iter().sum::<f32>() / channels as f32;

                    if needs_resample {
                        // Simple nearest-neighbor resampling
                        let target_idx = (i as f32 / resample_ratio) as usize;
                        if target_idx >= buf.len() || buf.is_empty() || target_idx != ((i.saturating_sub(1)) as f32 / resample_ratio) as usize {
                            buf.push(sample);
                        }
                    } else {
                        buf.push(sample);
                    }
                }
            }
        },
        |err| error!("Audio stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Hammertalk starting...");

    // Write PID file
    if let Err(e) = write_pid_file() {
        error!("Failed to write PID file: {}", e);
        std::process::exit(1);
    }

    // Load model
    let model_path = get_model_path();
    info!("Loading Moonshine model from {:?}", model_path);

    let mut engine = MoonshineEngine::new();
    if let Err(e) = engine.load_model_with_params(
        &model_path,
        MoonshineModelParams::variant(ModelVariant::Tiny),
    ) {
        error!("Failed to load model: {}", e);
        remove_pid_file();
        std::process::exit(1);
    }
    info!("Model loaded successfully");

    // Set up audio buffer
    let audio_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    // Set up audio stream
    let stream = match record_audio(Arc::clone(&audio_buffer)) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to set up audio: {}", e);
            remove_pid_file();
            std::process::exit(1);
        }
    };

    // Start the stream (it will only record when RECORDING is true)
    if let Err(e) = stream.play() {
        error!("Failed to start audio stream: {}", e);
        remove_pid_file();
        std::process::exit(1);
    }

    // Set up signal handlers
    let mut signals = Signals::new([SIGUSR1, SIGUSR2, SIGTERM, SIGINT]).unwrap();

    info!("Ready. Waiting for signals (USR1=start, USR2=stop)");

    for sig in signals.forever() {
        match sig {
            SIGUSR1 => {
                if !RECORDING.load(Ordering::SeqCst) {
                    info!("Starting recording...");
                    audio_buffer.lock().unwrap().clear();
                    STOP_REQUESTED.store(false, Ordering::SeqCst);
                    RECORDING.store(true, Ordering::SeqCst);
                }
            }
            SIGUSR2 => {
                if RECORDING.load(Ordering::SeqCst) {
                    info!("Stopping recording...");
                    RECORDING.store(false, Ordering::SeqCst);
                    STOP_REQUESTED.store(true, Ordering::SeqCst);

                    // Small delay to ensure buffer is complete
                    thread::sleep(Duration::from_millis(50));

                    let samples = {
                        let buf = audio_buffer.lock().unwrap();
                        buf.clone()
                    };

                    if samples.is_empty() {
                        warn!("No audio recorded");
                        continue;
                    }

                    info!("Transcribing {} samples ({:.2}s)...",
                          samples.len(),
                          samples.len() as f32 / SAMPLE_RATE as f32);

                    match engine.transcribe_samples(samples, None) {
                        Ok(result) => {
                            let text = result.text.trim();
                            if !text.is_empty() {
                                info!("Transcription: {}", text);
                                type_text(text);
                            } else {
                                warn!("Empty transcription result");
                            }
                        }
                        Err(e) => error!("Transcription failed: {}", e),
                    }
                }
            }
            SIGTERM | SIGINT => {
                info!("Shutting down...");
                SHUTDOWN.store(true, Ordering::SeqCst);
                break;
            }
            _ => {}
        }
    }

    drop(stream);
    remove_pid_file();
    info!("Goodbye!");
}
