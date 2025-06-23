use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use toml;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub openai: OpenAIConfig,
    pub elevenlabs: ElevenLabsConfig,
    pub azure: AzureConfig,
    pub processing: ProcessingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElevenLabsConfig {
    pub api_key: String,
    pub voice_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AzureConfig {
    pub speech_key: String,
    pub speech_region: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessingConfig {
    pub temp_dir: String,
    pub audio_sample_rate: u32,
    pub video_fps: f32,
    pub max_concurrent_requests: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai: OpenAIConfig {
                api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
                model: "gpt-4".to_string(),
            },
            elevenlabs: ElevenLabsConfig {
                api_key: std::env::var("ELEVENLABS_API_KEY").unwrap_or_default(),
                voice_id: None,
            },
            azure: AzureConfig {
                speech_key: std::env::var("AZURE_SPEECH_KEY").unwrap_or_default(),
                speech_region: std::env::var("AZURE_SPEECH_REGION").unwrap_or_default(),
            },
            processing: ProcessingConfig {
                temp_dir: "/tmp/auto-dubbing".to_string(),
                audio_sample_rate: 44100,
                video_fps: 30.0,
                max_concurrent_requests: 5,
            },
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = std::fs::read_to_string(path)?;
            let config = toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

            Ok(config)
        } else {
            let default_config = Config::default();
            let toml_string = toml::to_string_pretty(&default_config)
                .map_err(|e| anyhow::anyhow!("Failed to serialize default config: {}", e))?;

            std::fs::write(path, toml_string)?;

            Ok(default_config)
        }
    }
}
