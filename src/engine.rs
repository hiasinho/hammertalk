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
            EngineChoice::MoonshineTiny => Engine::Moonshine(Box::new(MoonshineEngine::new())),
            EngineChoice::WhisperTiny | EngineChoice::WhisperBase => {
                Engine::Whisper(WhisperEngine::new())
            }
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
                engine.load_model_with_params(
                    path,
                    MoonshineModelParams::variant(ModelVariant::Tiny),
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
    ) -> Result<TranscriptionResult, Box<dyn std::error::Error>> {
        match self {
            Engine::Moonshine(engine) => Ok(engine.transcribe_samples(samples, None)?),
            Engine::Whisper(engine) => {
                let params = WhisperInferenceParams {
                    language: Some("en".to_string()),
                    ..Default::default()
                };
                Ok(engine.transcribe_samples(samples, Some(params))?)
            }
        }
    }
}
