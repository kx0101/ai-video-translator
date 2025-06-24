use crate::config::ElevenLabsConfig;
use crate::error::{CustomError, Result};
use reqwest::Client;
use serde::Serialize;
use std::path::Path;
use tracing::info;

#[derive(Debug, Serialize)]
struct TextToSpeechRequest {
    text: String,
    model_id: String,
    voice_settings: VoiceSettings,
}

#[derive(Debug, Serialize)]
struct VoiceSettings {
    stability: f32,
    similarity_boost: f32,
    style: f32,
    use_speaker_boost: bool,
    speed: f32,
}

pub struct VoiceSynthesizer {
    client: Client,
    config: ElevenLabsConfig,
}

impl VoiceSynthesizer {
    pub fn new(config: ElevenLabsConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn text_to_speech(&self, text: &str, voice_id: &str) -> Result<Vec<u8>> {
        info!("Generating speech for text: {}", &text);

        let request = TextToSpeechRequest {
            text: text.to_string(),
            model_id: self.config.model_id.clone(),
            voice_settings: VoiceSettings {
                stability: 0.75,
                similarity_boost: 0.80,
                style: 0.0,
                use_speaker_boost: false,
                speed: 1.20,
            },
        };

        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}?output_format={}",
            voice_id, self.config.audio_format
        );

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(CustomError::VoiceSynthesis(format!(
                "Text-to-speech failed: {}",
                error_text
            )));
        }

        let audio_data = response.bytes().await?;

        Ok(audio_data.to_vec())
    }

    pub async fn save_audio(&self, audio_data: &[u8], output_path: &Path) -> Result<()> {
        std::fs::write(output_path, audio_data)
            .map_err(|e| CustomError::VoiceSynthesis(format!("Failed to save audio: {}", e)))?;

        info!("Audio saved to: {:?}", output_path);
        Ok(())
    }
}
