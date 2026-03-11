use std::path::Path;

use log::info;
use transcribe_rs::engines::moonshine::{ModelVariant, MoonshineEngine, MoonshineModelParams};
use transcribe_rs::engines::whisper::{WhisperEngine, WhisperInferenceParams, WhisperModelParams};
use transcribe_rs::{TranscriptionEngine, TranscriptionResult};

use crate::EngineChoice;

pub enum Engine {
    Moonshine(Box<MoonshineEngine>),
    Whisper(WhisperEngine),
}

impl Engine {
    pub fn new(choice: &EngineChoice) -> Self {
        match choice {
            EngineChoice::MoonshineTiny | EngineChoice::MoonshineBase => {
                Engine::Moonshine(Box::new(MoonshineEngine::new()))
            }
            EngineChoice::WhisperTiny
            | EngineChoice::WhisperBase
            | EngineChoice::WhisperSmall
            | EngineChoice::WhisperMedium
            | EngineChoice::WhisperLargeV3
            | EngineChoice::WhisperLargeV3Turbo => Engine::Whisper(WhisperEngine::new()),
        }
    }

    pub fn load(
        &mut self,
        choice: &EngineChoice,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Loading {} engine from {:?}", choice, path);
        match self {
            Engine::Moonshine(engine) => {
                let variant = match choice {
                    EngineChoice::MoonshineBase => ModelVariant::Base,
                    _ => ModelVariant::Tiny,
                };
                engine.load_model_with_params(
                    path,
                    MoonshineModelParams::variant(variant),
                )?;
            }
            Engine::Whisper(engine) => {
                engine.load_model_with_params(path, WhisperModelParams::default())?;
            }
        }
        Ok(())
    }

    pub fn transcribe(
        &mut self,
        samples: Vec<f32>,
        language: Option<&str>,
    ) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        match self {
            Engine::Moonshine(engine) => Ok(engine.transcribe_samples(samples, None)?),
            Engine::Whisper(engine) => {
                let params = WhisperInferenceParams {
                    language: language.map(|s| s.to_string()),
                    ..Default::default()
                };
                Ok(engine.transcribe_samples(samples, Some(params))?)
            }
        }
    }
}
