use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use log::{error, info, warn};
use signal_hook::consts::{SIGINT, SIGTERM, SIGUSR1, SIGUSR2};
use signal_hook::iterator::Signals;
use transcribe_rs::engines::moonshine::{ModelVariant, MoonshineEngine, MoonshineModelParams};
use transcribe_rs::TranscriptionEngine;

use hammertalk::{
    fatal_exit, get_model_path, needs_resample, remove_pid_file, type_text, write_pid_file,
    BUFFER_DRAIN_DELAY_MS, SAMPLE_RATE,
};

static RECORDING: AtomicBool = AtomicBool::new(false);

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

    let sample_rate =
        if SAMPLE_RATE >= config.min_sample_rate().0 && SAMPLE_RATE <= config.max_sample_rate().0 {
            SAMPLE_RATE
        } else {
            config.max_sample_rate().0
        };

    let config = config.with_sample_rate(cpal::SampleRate(sample_rate));
    let channels = config.channels() as usize;

    info!("Recording at {} Hz, {} channels", sample_rate, channels);

    let resample_ratio = sample_rate as f32 / SAMPLE_RATE as f32;
    let should_resample = needs_resample(sample_rate, SAMPLE_RATE);

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if RECORDING.load(Ordering::SeqCst) {
                let mut buf = buffer.lock().unwrap();

                // Convert to mono if needed and resample
                for (i, chunk) in data.chunks(channels).enumerate() {
                    let sample: f32 = chunk.iter().sum::<f32>() / channels as f32;

                    if should_resample {
                        // Nearest-neighbor resampling: only push sample when we've moved
                        // to a new target index, effectively decimating higher sample rates
                        let target_idx = (i as f32 / resample_ratio) as usize;
                        let prev_target_idx =
                            ((i.saturating_sub(1)) as f32 / resample_ratio) as usize;
                        let is_new_target = target_idx >= buf.len()
                            || buf.is_empty()
                            || target_idx != prev_target_idx;
                        if is_new_target {
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
        fatal_exit(&format!("Failed to write PID file: {}", e));
    }

    // Load model
    let model_path = get_model_path();
    info!("Loading Moonshine model from {:?}", model_path);

    let mut engine = MoonshineEngine::new();
    if let Err(e) = engine.load_model_with_params(
        &model_path,
        MoonshineModelParams::variant(ModelVariant::Tiny),
    ) {
        fatal_exit(&format!("Failed to load model: {}", e));
    }
    info!("Model loaded successfully");

    // Set up audio buffer
    let audio_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    // Set up audio stream
    let stream = match record_audio(Arc::clone(&audio_buffer)) {
        Ok(s) => s,
        Err(e) => fatal_exit(&format!("Failed to set up audio: {}", e)),
    };

    // Start the stream (it will only record when RECORDING is true)
    if let Err(e) = stream.play() {
        fatal_exit(&format!("Failed to start audio stream: {}", e));
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
                    RECORDING.store(true, Ordering::SeqCst);
                }
            }
            SIGUSR2 => {
                if RECORDING.load(Ordering::SeqCst) {
                    info!("Stopping recording...");
                    RECORDING.store(false, Ordering::SeqCst);

                    // Small delay to ensure buffer is complete
                    thread::sleep(Duration::from_millis(BUFFER_DRAIN_DELAY_MS));

                    let samples = {
                        let buf = audio_buffer.lock().unwrap();
                        buf.clone()
                    };

                    if samples.is_empty() {
                        warn!("No audio recorded");
                        continue;
                    }

                    info!(
                        "Transcribing {} samples ({:.2}s)...",
                        samples.len(),
                        samples.len() as f32 / SAMPLE_RATE as f32
                    );

                    match engine.transcribe_samples(samples, None) {
                        Ok(result) => {
                            let text = result.text.trim();
                            info!("Transcription: {}", text);
                            type_text(text);
                        }
                        Err(e) => error!("Transcription failed: {}", e),
                    }
                }
            }
            SIGTERM | SIGINT => {
                info!("Shutting down...");
                break;
            }
            _ => {}
        }
    }

    drop(stream);
    remove_pid_file();
    info!("Goodbye!");
}
