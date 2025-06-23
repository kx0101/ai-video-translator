use crate::error::{CustomError, Result};
use base64::Engine;
use std::path::Path;
use std::process::Command;
use tracing::info;

pub struct AudioProcessor {
    sample_rate: u32,
}

impl AudioProcessor {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    pub async fn extract_audio(&self, video_path: &Path, audio_path: &Path) -> Result<()> {
        info!("Extracting audio from video: {}", video_path.display());

        let output = Command::new("ffmpeg")
            .args([
                "-i",
                video_path.to_str().unwrap(),
                "-vn", // no video
                "-acodec",
                "pcm_s16le", // audio codec
                "-ar",
                &self.sample_rate.to_string(), // sample rate
                "-ac",
                "1", // mono audio
                audio_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| CustomError::AudioProcessing(format!("Failed to run ffmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(CustomError::AudioProcessing(format!(
                "ffmpeg failed with error: {}",
                error
            )));
        }

        info!("Audio extracted successfully to: {}", audio_path.display());
        Ok(())
    }

    pub fn audio_to_base64(&self, audio_path: &Path) -> Result<String> {
        let audio_data = std::fs::read(audio_path).map_err(|e| {
            CustomError::AudioProcessing(format!("Failed to read audio file: {}", e))
        })?;

        let base64_audio = base64::engine::general_purpose::STANDARD.encode(audio_data);
        Ok(base64_audio)
    }

    pub fn save_audio_from_base64(&self, base64_data: &str, output_path: &Path) -> Result<()> {
        let audio_data = base64::engine::general_purpose::STANDARD
            .decode(base64_data)
            .map_err(|e| {
                CustomError::AudioProcessing(format!("Failed to decode base64 audio: {}", e))
            })?;

        std::fs::write(output_path, audio_data).map_err(|e| {
            CustomError::AudioProcessing(format!("Failed to write audio file: {}", e))
        })?;

        info!("Audio saved successfully to: {}", output_path.display());
        Ok(())
    }

    pub fn get_audio_duration(&self, audio_path: &Path) -> Result<f64> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-show_entries",
                "format=duration",
                "-of",
                "csv=p=0", // output format
                audio_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| CustomError::AudioProcessing(format!("Failed to run ffprobe: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(CustomError::AudioProcessing(format!(
                "ffprobe failed with error: {}",
                error
            )));
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration = duration_str.trim().parse::<f64>().map_err(|e| {
            CustomError::AudioProcessing(format!("Failed to parse duration: {}", e))
        })?;

        Ok(duration)
    }

    pub async fn normalize_audio(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        info!("Normalizing audio: {}", input_path.display());

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                input_path.to_str().unwrap(),
                "-af",
                "loudnorm", // loudness normalization filter
                output_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| CustomError::AudioProcessing(format!("Failed to run ffmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(CustomError::AudioProcessing(format!(
                "ffmpeg normalization failed with error: {}",
                error
            )));
        }

        info!(
            "Audio normalized successfully to: {}",
            output_path.display()
        );
        Ok(())
    }
}
