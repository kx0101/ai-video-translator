mod audio;
mod config;
mod error;

use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let audio_processor = audio::AudioProcessor::new(44100);

    let _ = audio_processor
        .extract_audio(
            PathBuf::from("input_video.mp4").as_path(),
            PathBuf::from("output_audio.wav").as_path(),
        )
        .await;

    let duration = audio_processor
        .get_audio_duration(PathBuf::from("output_audio.wav").as_path())
        .unwrap_or_else(|e| {
            error!("Failed to get audio duration: {}", e);
            0.0
        });

    let base64_audio = audio_processor
        .audio_to_base64(PathBuf::from("output_audio.wav").as_path())
        .unwrap_or_else(|e| {
            error!("Failed to convert audio to base64: {}", e);
            String::new()
        });

    audio_processor
        .normalize_audio(
            PathBuf::from("output_audio.wav").as_path(),
            PathBuf::from("normalized_audio.wav").as_path(),
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to normalize audio: {}", e);
        });

    println!("Audio duration: {:.2} seconds", duration);
}
