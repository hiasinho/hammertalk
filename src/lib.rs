use std::fmt;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use log::{debug, error, info, warn};
use serde::Deserialize;

pub mod engine;

pub const SAMPLE_RATE: u32 = 16000;

/// Tolerance for determining if resampling is needed (0.1% deviation from target)
pub const RESAMPLE_TOLERANCE: f32 = 0.001;

/// Delay in milliseconds to allow audio buffer to drain before transcription
pub const BUFFER_DRAIN_DELAY_MS: u64 = 50;

#[derive(Debug, Clone, PartialEq)]
pub enum EngineChoice {
    MoonshineTiny,
    WhisperTiny,
    WhisperBase,
}

impl FromStr for EngineChoice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "moonshine-tiny" | "moonshine_tiny" => Ok(EngineChoice::MoonshineTiny),
            "whisper-tiny" | "whisper_tiny" => Ok(EngineChoice::WhisperTiny),
            "whisper-base" | "whisper_base" => Ok(EngineChoice::WhisperBase),
            _ => Err(format!("unknown engine: {}", s)),
        }
    }
}

impl fmt::Display for EngineChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineChoice::MoonshineTiny => write!(f, "moonshine-tiny"),
            EngineChoice::WhisperTiny => write!(f, "whisper-tiny"),
            EngineChoice::WhisperBase => write!(f, "whisper-base"),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub engine: Option<String>,
}

pub fn get_config_path() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        })
        .join("hammertalk/config.toml")
}

pub fn load_config() -> Config {
    let path = get_config_path();
    match fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str(&contents) {
            Ok(config) => config,
            Err(e) => {
                warn!("Failed to parse config file {:?}: {}", path, e);
                Config::default()
            }
        },
        Err(_) => Config::default(),
    }
}

pub fn parse_engine_choice() -> EngineChoice {
    // Check CLI args: --engine <name>
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--engine") {
        if let Some(name) = args.get(pos + 1) {
            match name.parse() {
                Ok(choice) => return choice,
                Err(_) => warn!("Unknown engine '{}', using default", name),
            }
        }
    }

    // Fall back to env var
    if let Ok(val) = std::env::var("HAMMERTALK_ENGINE") {
        match val.parse() {
            Ok(choice) => return choice,
            Err(_) => warn!("Unknown HAMMERTALK_ENGINE '{}', using default", val),
        }
    }

    // Fall back to config file
    let config = load_config();
    if let Some(engine) = config.engine {
        match engine.parse() {
            Ok(choice) => return choice,
            Err(_) => warn!("Unknown engine '{}' in config file, using default", engine),
        }
    }

    EngineChoice::MoonshineTiny
}

pub fn get_pid_path() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join("hammertalk.pid")
}

pub fn get_model_path(engine: &EngineChoice) -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share")
        })
        .join("hammertalk/models");

    match engine {
        EngineChoice::MoonshineTiny => base.join("moonshine-tiny"),
        EngineChoice::WhisperTiny => base.join("ggml-tiny.en.bin"),
        EngineChoice::WhisperBase => base.join("ggml-base.en.bin"),
    }
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

    let text_with_space = format!("{} ", text);
    info!("Typing: {}", text_with_space);
    let result = Command::new("ydotool")
        .args(["type", "-d", "0", "-H", "0", "--", &text_with_space])
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

        let model_path = get_model_path(&EngineChoice::MoonshineTiny);

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

        let model_path = get_model_path(&EngineChoice::MoonshineTiny);

        let expected = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local/share/hammertalk/models/moonshine-tiny");
        assert_eq!(model_path, expected);
    }

    #[test]
    #[serial]
    fn test_get_model_path_whisper_tiny() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_DATA_HOME", temp.path());

        let model_path = get_model_path(&EngineChoice::WhisperTiny);

        assert_eq!(
            model_path,
            temp.path().join("hammertalk/models/ggml-tiny.en.bin")
        );
        env::remove_var("XDG_DATA_HOME");
    }

    #[test]
    #[serial]
    fn test_get_model_path_whisper_base() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_DATA_HOME", temp.path());

        let model_path = get_model_path(&EngineChoice::WhisperBase);

        assert_eq!(
            model_path,
            temp.path().join("hammertalk/models/ggml-base.en.bin")
        );
        env::remove_var("XDG_DATA_HOME");
    }

    #[test]
    fn test_engine_choice_from_str() {
        assert_eq!(
            "moonshine-tiny".parse::<EngineChoice>().unwrap(),
            EngineChoice::MoonshineTiny
        );
        assert_eq!(
            "moonshine_tiny".parse::<EngineChoice>().unwrap(),
            EngineChoice::MoonshineTiny
        );
        assert_eq!(
            "whisper-tiny".parse::<EngineChoice>().unwrap(),
            EngineChoice::WhisperTiny
        );
        assert_eq!(
            "whisper_tiny".parse::<EngineChoice>().unwrap(),
            EngineChoice::WhisperTiny
        );
        assert_eq!(
            "whisper-base".parse::<EngineChoice>().unwrap(),
            EngineChoice::WhisperBase
        );
        assert_eq!(
            "whisper_base".parse::<EngineChoice>().unwrap(),
            EngineChoice::WhisperBase
        );
        assert!("unknown".parse::<EngineChoice>().is_err());
    }

    #[test]
    fn test_engine_choice_display() {
        assert_eq!(EngineChoice::MoonshineTiny.to_string(), "moonshine-tiny");
        assert_eq!(EngineChoice::WhisperTiny.to_string(), "whisper-tiny");
        assert_eq!(EngineChoice::WhisperBase.to_string(), "whisper-base");
    }

    #[test]
    fn test_engine_choice_case_insensitive() {
        assert_eq!(
            "Whisper-Tiny".parse::<EngineChoice>().unwrap(),
            EngineChoice::WhisperTiny
        );
        assert_eq!(
            "MOONSHINE-TINY".parse::<EngineChoice>().unwrap(),
            EngineChoice::MoonshineTiny
        );
    }

    #[test]
    #[serial]
    fn test_parse_engine_choice_default() {
        env::remove_var("HAMMERTALK_ENGINE");
        let choice = parse_engine_choice();
        assert_eq!(choice, EngineChoice::MoonshineTiny);
    }

    #[test]
    #[serial]
    fn test_parse_engine_choice_env_var() {
        env::set_var("HAMMERTALK_ENGINE", "whisper-base");
        let choice = parse_engine_choice();
        assert_eq!(choice, EngineChoice::WhisperBase);
        env::remove_var("HAMMERTALK_ENGINE");
    }

    #[test]
    #[serial]
    fn test_parse_engine_choice_invalid_env_var() {
        env::set_var("HAMMERTALK_ENGINE", "invalid-engine");
        let choice = parse_engine_choice();
        assert_eq!(choice, EngineChoice::MoonshineTiny);
        env::remove_var("HAMMERTALK_ENGINE");
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
    #[serial]
    fn test_get_config_path_with_xdg_config_home() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_CONFIG_HOME", temp.path());

        let config_path = get_config_path();

        assert_eq!(config_path, temp.path().join("hammertalk/config.toml"));
        env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    #[serial]
    fn test_get_config_path_fallback() {
        env::remove_var("XDG_CONFIG_HOME");

        let config_path = get_config_path();

        let expected = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config/hammertalk/config.toml");
        assert_eq!(config_path, expected);
    }

    #[test]
    #[serial]
    fn test_load_config_with_engine() {
        let temp = tempdir().unwrap();
        let config_dir = temp.path().join("hammertalk");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.toml"),
            "engine = \"whisper-tiny\"\n",
        )
        .unwrap();
        env::set_var("XDG_CONFIG_HOME", temp.path());

        let config = load_config();

        assert_eq!(config.engine, Some("whisper-tiny".to_string()));
        env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    #[serial]
    fn test_load_config_missing_file() {
        let temp = tempdir().unwrap();
        env::set_var("XDG_CONFIG_HOME", temp.path());

        let config = load_config();

        assert!(config.engine.is_none());
        env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    #[serial]
    fn test_load_config_empty() {
        let temp = tempdir().unwrap();
        let config_dir = temp.path().join("hammertalk");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.toml"), "").unwrap();
        env::set_var("XDG_CONFIG_HOME", temp.path());

        let config = load_config();

        assert!(config.engine.is_none());
        env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    #[serial]
    fn test_parse_engine_choice_config_file() {
        env::remove_var("HAMMERTALK_ENGINE");
        let temp = tempdir().unwrap();
        let config_dir = temp.path().join("hammertalk");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.toml"),
            "engine = \"whisper-base\"\n",
        )
        .unwrap();
        env::set_var("XDG_CONFIG_HOME", temp.path());

        let choice = parse_engine_choice();

        assert_eq!(choice, EngineChoice::WhisperBase);
        env::remove_var("XDG_CONFIG_HOME");
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
