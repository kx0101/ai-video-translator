use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use toml;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub google: GoogleConfig,
    pub elevenlabs: ElevenLabsConfig,
    pub lipsync: LipSyncConfig,
    pub processing: ProcessingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GoogleConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElevenLabsConfig {
    pub api_key: String,
    pub voice_id: String,
    pub model_id: String,
    pub audio_format: String,
    pub stability: f32,
    pub similarity_boost: f32,
    pub style: f32,
    pub use_speaker_boost: bool,
    pub speed: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LipSyncConfig {
    pub api_key: String,
    pub model: String,
    pub ngrok_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessingConfig {
    pub audio_sample_rate: u32,
    pub video_fps: f64,
    pub stt_sample_rate: u32,
    pub alternative_languages: [String; 4],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            google: GoogleConfig {
                api_key: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").unwrap_or_default(),
            },
            elevenlabs: ElevenLabsConfig {
                api_key: std::env::var("ELEVENLABS_API_KEY").unwrap_or_default(),
                voice_id: "9F4C8ztpNUmXkdDDbz3J".to_string(),
                model_id: "eleven_multilingual_v2".to_string(),
                audio_format: "mp3_44100_128".to_string(),
                stability: 0.75,
                similarity_boost: 0.80,
                style: 0.0,
                use_speaker_boost: false,
                speed: 1.15,
            },
            lipsync: LipSyncConfig {
                api_key: std::env::var("WAV2LIP_API_KEY").unwrap_or_default(),
                model: "lipsync-2".to_string(),
                ngrok_url: "https://ab6b-91-140-28-26.ngrok-free.app".to_string(),
            },
            processing: ProcessingConfig {
                audio_sample_rate: 44100,
                video_fps: 30.0,
                stt_sample_rate: 16000,
                alternative_languages: [
                    "en-US".to_string(),
                    "el-GR".to_string(),
                    "es-ES".to_string(),
                    "fr-FR".to_string(),
                ],
            },
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            let default_config = Config::default();
            let toml_string = toml::to_string_pretty(&default_config)
                .map_err(|e| anyhow::anyhow!("Failed to serialize default config: {}", e))?;

            std::fs::write(path, toml_string)?;

            return Ok(default_config);
        }

        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

        Ok(config)
    }
}
