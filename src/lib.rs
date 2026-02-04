use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use log::{debug, error, info, warn};

pub const SAMPLE_RATE: u32 = 16000;

/// Tolerance for determining if resampling is needed (0.1% deviation from target)
pub const RESAMPLE_TOLERANCE: f32 = 0.001;

/// Delay in milliseconds to allow audio buffer to drain before transcription
pub const BUFFER_DRAIN_DELAY_MS: u64 = 50;

pub fn get_pid_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join("hammertalk.pid")
}

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

pub fn write_pid_file() -> std::io::Result<()> {
    let pid_path = get_pid_path();
    let mut file = fs::File::create(&pid_path)?;
    writeln!(file, "{}", std::process::id())?;
    info!("PID file written to {:?}", pid_path);
    Ok(())
}

pub fn remove_pid_file() {
    let pid_path = get_pid_path();
    if let Err(e) = fs::remove_file(&pid_path) {
        warn!("Failed to remove PID file: {}", e);
    }
}

pub fn should_type_text(text: &str) -> bool {
    !text.trim().is_empty()
}

pub fn type_text(text: &str) {
    if !should_type_text(text) {
        warn!("Empty transcription, skipping");
        return;
    }

    info!("Typing: {}", text);
    let result = Command::new("ydotool")
        .args(["type", "-d", "0", "-H", "0", "--", text])
        .status();

    match result {
        Ok(status) if status.success() => debug!("ydotool succeeded"),
        Ok(status) => warn!("ydotool exited with: {}", status),
        Err(e) => error!("Failed to run ydotool: {}", e),
    }
}

pub fn needs_resample(source_rate: u32, target_rate: u32) -> bool {
    let ratio = source_rate as f32 / target_rate as f32;
    (ratio - 1.0).abs() > RESAMPLE_TOLERANCE
}

/// Exit with error after cleanup. Used for fatal initialization failures.
pub fn fatal_exit(msg: &str) -> ! {
    log::error!("{}", msg);
    remove_pid_file();
    std::process::exit(1);
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

        assert_eq!(
            model_path,
            temp.path().join("hammertalk/models/moonshine-tiny")
        );
        env::remove_var("XDG_DATA_HOME");
    }

    #[test]
    #[serial]
    fn test_get_model_path_fallback() {
        env::remove_var("XDG_DATA_HOME");

        let model_path = get_model_path();

        let expected = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/hammertalk/models/moonshine-tiny");
        assert_eq!(model_path, expected);
    }

    #[test]
    #[serial]
    fn test_pid_file_roundtrip() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_RUNTIME_DIR", temp.path());

        write_pid_file().unwrap();

        let pid_path = get_pid_path();
        assert!(pid_path.exists());

        let contents = fs::read_to_string(&pid_path).unwrap();
        let written_pid: u32 = contents.trim().parse().unwrap();
        assert_eq!(written_pid, std::process::id());

        remove_pid_file();
        assert!(!pid_path.exists());

        env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    #[serial]
    fn test_remove_pid_file_nonexistent() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_RUNTIME_DIR", temp.path());

        remove_pid_file();

        env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_should_type_text_with_content() {
        assert!(should_type_text("hello"));
        assert!(should_type_text("  hello  "));
        assert!(should_type_text("hello world"));
    }

    #[test]
    fn test_should_type_text_empty() {
        assert!(!should_type_text(""));
        assert!(!should_type_text("   "));
        assert!(!should_type_text("\t\n"));
    }

    #[test]
    fn test_needs_resample() {
        assert!(!needs_resample(16000, 16000));
        assert!(needs_resample(44100, 16000));
        assert!(needs_resample(48000, 16000));
        assert!(needs_resample(8000, 16000));
        assert!(!needs_resample(16001, 16000));
    }

    #[test]
    fn test_needs_resample_boundary() {
        assert!(!needs_resample(16015, 16000));
        assert!(needs_resample(16017, 16000));
    }

    #[test]
    fn test_sample_rate_constant() {
        assert_eq!(SAMPLE_RATE, 16000);
    }

    #[test]
    fn test_resample_tolerance_constant() {
        assert_eq!(RESAMPLE_TOLERANCE, 0.001);
    }

    #[test]
    fn test_buffer_drain_delay_constant() {
        assert_eq!(BUFFER_DRAIN_DELAY_MS, 50);
    }
}
