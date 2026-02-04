use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use log::{debug, error, info, warn};

pub const SAMPLE_RATE: u32 = 16000;

/// Get the path for the PID file
pub fn get_pid_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join("hammertalk.pid")
}

/// Get the path for the Moonshine model
pub fn get_model_path() -> PathBuf {
    std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share")
        })
        .join("hammertalk/models/moonshine-tiny")
}

/// Write the PID file
pub fn write_pid_file() -> std::io::Result<()> {
    let pid_path = get_pid_path();
    let mut file = fs::File::create(&pid_path)?;
    writeln!(file, "{}", std::process::id())?;
    info!("PID file written to {:?}", pid_path);
    Ok(())
}

/// Remove the PID file
pub fn remove_pid_file() {
    let pid_path = get_pid_path();
    if let Err(e) = fs::remove_file(&pid_path) {
        warn!("Failed to remove PID file: {}", e);
    }
}

/// Type text using ydotool
pub fn type_text(text: &str) {
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

/// Convert multi-channel audio to mono by averaging channels
pub fn to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    samples
        .chunks(channels)
        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Resample audio using nearest-neighbor interpolation
pub fn resample(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if source_rate == target_rate {
        return samples.to_vec();
    }

    let ratio = source_rate as f64 / target_rate as f64;
    let output_len = (samples.len() as f64 / ratio).ceil() as usize;

    (0..output_len)
        .map(|i| {
            let src_idx = (i as f64 * ratio) as usize;
            samples.get(src_idx).copied().unwrap_or(0.0)
        })
        .collect()
}

/// Calculate audio duration in seconds
pub fn audio_duration_secs(sample_count: usize, sample_rate: u32) -> f32 {
    sample_count as f32 / sample_rate as f32
}

/// Check if resampling is needed based on sample rates
pub fn needs_resample(source_rate: u32, target_rate: u32) -> bool {
    let ratio = source_rate as f32 / target_rate as f32;
    (ratio - 1.0).abs() > 0.001
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn test_get_pid_path_with_xdg_runtime_dir() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_RUNTIME_DIR", temp.path());

        let pid_path = get_pid_path();

        assert_eq!(pid_path, temp.path().join("hammertalk.pid"));
        env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    #[serial]
    fn test_get_pid_path_fallback() {
        env::remove_var("XDG_RUNTIME_DIR");

        let pid_path = get_pid_path();

        assert_eq!(pid_path, PathBuf::from("/tmp/hammertalk.pid"));
    }

    #[test]
    #[serial]
    fn test_get_model_path_with_xdg_data_home() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_DATA_HOME", temp.path());

        let model_path = get_model_path();

        assert_eq!(model_path, temp.path().join("hammertalk/models/moonshine-tiny"));
        env::remove_var("XDG_DATA_HOME");
    }

    #[test]
    #[serial]
    fn test_get_model_path_fallback() {
        env::remove_var("XDG_DATA_HOME");

        let model_path = get_model_path();

        // Should use home dir fallback
        let expected = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/hammertalk/models/moonshine-tiny");
        assert_eq!(model_path, expected);
    }

    #[test]
    fn test_to_mono_stereo() {
        let stereo = vec![0.5, 0.3, 0.8, 0.2, 1.0, 0.0];
        let mono = to_mono(&stereo, 2);

        assert_eq!(mono.len(), 3);
        assert!((mono[0] - 0.4).abs() < 0.001); // (0.5 + 0.3) / 2
        assert!((mono[1] - 0.5).abs() < 0.001); // (0.8 + 0.2) / 2
        assert!((mono[2] - 0.5).abs() < 0.001); // (1.0 + 0.0) / 2
    }

    #[test]
    fn test_to_mono_already_mono() {
        let mono_input = vec![0.1, 0.2, 0.3, 0.4];
        let mono = to_mono(&mono_input, 1);

        assert_eq!(mono, mono_input);
    }

    #[test]
    fn test_to_mono_quad() {
        let quad = vec![0.4, 0.4, 0.4, 0.4, 0.8, 0.8, 0.8, 0.8];
        let mono = to_mono(&quad, 4);

        assert_eq!(mono.len(), 2);
        assert!((mono[0] - 0.4).abs() < 0.001);
        assert!((mono[1] - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_to_mono_empty() {
        let empty: Vec<f32> = vec![];
        let mono = to_mono(&empty, 2);

        assert!(mono.is_empty());
    }

    #[test]
    fn test_resample_same_rate() {
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let resampled = resample(&samples, 16000, 16000);

        assert_eq!(resampled, samples);
    }

    #[test]
    fn test_resample_downsample_2x() {
        let samples = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7];
        let resampled = resample(&samples, 32000, 16000);

        // Downsampling 2x should roughly halve the samples
        assert_eq!(resampled.len(), 4);
        assert!((resampled[0] - 0.0).abs() < 0.001);
        assert!((resampled[1] - 0.2).abs() < 0.001);
        assert!((resampled[2] - 0.4).abs() < 0.001);
        assert!((resampled[3] - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_resample_upsample_2x() {
        let samples = vec![0.0, 0.4, 0.8, 1.0];
        let resampled = resample(&samples, 8000, 16000);

        // Upsampling 2x should roughly double the samples
        assert_eq!(resampled.len(), 8);
    }

    #[test]
    fn test_resample_empty() {
        let empty: Vec<f32> = vec![];
        let resampled = resample(&empty, 44100, 16000);

        assert!(resampled.is_empty());
    }

    #[test]
    fn test_audio_duration_secs() {
        assert!((audio_duration_secs(16000, 16000) - 1.0).abs() < 0.001);
        assert!((audio_duration_secs(32000, 16000) - 2.0).abs() < 0.001);
        assert!((audio_duration_secs(8000, 16000) - 0.5).abs() < 0.001);
        assert!((audio_duration_secs(0, 16000) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_needs_resample() {
        assert!(!needs_resample(16000, 16000));
        assert!(needs_resample(44100, 16000));
        assert!(needs_resample(48000, 16000));
        assert!(needs_resample(8000, 16000));
        // Very close rates should not need resampling
        assert!(!needs_resample(16001, 16000));
    }

    #[test]
    fn test_sample_rate_constant() {
        assert_eq!(SAMPLE_RATE, 16000);
    }

    #[test]
    #[serial]
    fn test_pid_file_roundtrip() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_RUNTIME_DIR", temp.path());

        // Write PID file
        write_pid_file().unwrap();

        // Verify it exists and contains our PID
        let pid_path = get_pid_path();
        assert!(pid_path.exists());

        let contents = fs::read_to_string(&pid_path).unwrap();
        let written_pid: u32 = contents.trim().parse().unwrap();
        assert_eq!(written_pid, std::process::id());

        // Remove PID file
        remove_pid_file();
        assert!(!pid_path.exists());

        env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    #[serial]
    fn test_remove_pid_file_nonexistent() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_RUNTIME_DIR", temp.path());

        // Should not panic when file doesn't exist
        remove_pid_file();

        env::remove_var("XDG_RUNTIME_DIR");
    }

    // Additional edge case tests

    #[test]
    fn test_to_mono_single_sample() {
        let single = vec![0.5];
        let mono = to_mono(&single, 1);
        assert_eq!(mono, vec![0.5]);
    }

    #[test]
    fn test_to_mono_stereo_single_frame() {
        let stereo = vec![0.2, 0.8];
        let mono = to_mono(&stereo, 2);
        assert_eq!(mono.len(), 1);
        assert!((mono[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_to_mono_preserves_silence() {
        let silence = vec![0.0, 0.0, 0.0, 0.0];
        let mono = to_mono(&silence, 2);
        assert!(mono.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_to_mono_extreme_values() {
        let extreme = vec![1.0, -1.0, -1.0, 1.0];
        let mono = to_mono(&extreme, 2);
        assert_eq!(mono.len(), 2);
        assert!((mono[0] - 0.0).abs() < 0.001); // (1.0 + -1.0) / 2
        assert!((mono[1] - 0.0).abs() < 0.001); // (-1.0 + 1.0) / 2
    }

    #[test]
    fn test_resample_single_sample() {
        let samples = vec![0.5];
        let resampled = resample(&samples, 16000, 16000);
        assert_eq!(resampled, vec![0.5]);
    }

    #[test]
    fn test_resample_48k_to_16k() {
        // Common scenario: 48kHz to 16kHz (3x downsampling)
        let samples: Vec<f32> = (0..48).map(|i| i as f32 / 48.0).collect();
        let resampled = resample(&samples, 48000, 16000);

        // Should be roughly 1/3 the length
        assert!(resampled.len() >= 15 && resampled.len() <= 17);
    }

    #[test]
    fn test_resample_44100_to_16000() {
        // Common scenario: 44.1kHz to 16kHz
        let samples: Vec<f32> = (0..441).map(|i| (i as f32 / 441.0).sin()).collect();
        let resampled = resample(&samples, 44100, 16000);

        // 441 samples at 44100Hz = 0.01s
        // Should produce ~160 samples at 16000Hz
        let expected_approx = (441.0 * 16000.0 / 44100.0) as usize;
        assert!(resampled.len() >= expected_approx - 2 && resampled.len() <= expected_approx + 2);
    }

    #[test]
    fn test_audio_duration_secs_various_rates() {
        // 1 second of audio at various sample rates
        assert!((audio_duration_secs(44100, 44100) - 1.0).abs() < 0.001);
        assert!((audio_duration_secs(48000, 48000) - 1.0).abs() < 0.001);
        assert!((audio_duration_secs(8000, 8000) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_audio_duration_secs_fractional() {
        // 0.5 seconds
        assert!((audio_duration_secs(8000, 16000) - 0.5).abs() < 0.001);
        // 1.5 seconds
        assert!((audio_duration_secs(24000, 16000) - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_needs_resample_common_rates() {
        // Common audio sample rates
        assert!(needs_resample(44100, 16000));
        assert!(needs_resample(48000, 16000));
        assert!(needs_resample(22050, 16000));
        assert!(needs_resample(8000, 16000));
        assert!(!needs_resample(16000, 16000));
    }

    #[test]
    fn test_needs_resample_boundary() {
        // Tolerance is 0.001 (0.1%) - values within this range don't need resampling
        // 16001/16000 = 1.0000625, |1.0000625 - 1.0| = 0.0000625 < 0.001
        assert!(!needs_resample(16001, 16000));
        // 16016/16000 = 1.001, |1.001 - 1.0| = 0.001 - at boundary
        assert!(!needs_resample(16015, 16000)); // Just under threshold
        assert!(needs_resample(16017, 16000));  // Just over threshold
    }

    #[test]
    #[serial]
    fn test_write_pid_file_creates_parent_dir_if_exists() {
        let temp = tempdir().unwrap();
        let nested = temp.path().join("nested");
        fs::create_dir(&nested).unwrap();
        env::set_var("XDG_RUNTIME_DIR", &nested);

        let result = write_pid_file();
        assert!(result.is_ok());

        let pid_path = get_pid_path();
        assert!(pid_path.exists());

        // Cleanup
        remove_pid_file();
        env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_resample_preserves_first_and_last() {
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let resampled = resample(&samples, 16000, 16000);

        // Same rate should preserve exactly
        assert!((resampled[0] - 0.1).abs() < 0.001);
        assert!((resampled[resampled.len() - 1] - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_to_mono_with_6_channels() {
        // 5.1 surround sound
        let surround = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let mono = to_mono(&surround, 6);

        assert_eq!(mono.len(), 1);
        let expected = (0.1 + 0.2 + 0.3 + 0.4 + 0.5 + 0.6) / 6.0;
        assert!((mono[0] - expected).abs() < 0.001);
    }
}
