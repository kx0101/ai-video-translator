mod audio;
mod config;
mod error;
mod video;

use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let video_processor = video::VideoProcessor::new(30.0);
    let audio_processor = audio::AudioProcessor::new(44100);

    let _ = audio_processor
        .extract_audio(
            &PathBuf::from("input_video.mp4"),
            &PathBuf::from("output_audio.wav"),
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to extract audio: {}", e);
        });

    let _ = audio_processor
        .normalize_audio(
            &PathBuf::from("output_audio.wav"),
            &PathBuf::from("normalized_audio.wav"),
        )
        .await;

    video_processor
        .extract_frames(&PathBuf::from("input_video.mp4"), &PathBuf::from("frames"))
        .await
        .unwrap_or_else(|e| {
            error!("Failed to extract frames: {}", e);
        });

    video_processor
        .combine_frames_with_audio(
            &PathBuf::from("frames"),
            &PathBuf::from("normalized_audio.wav"),
            &PathBuf::from("output_video_from_func.mp4"),
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to combine frames with audio: {}", e);
        });

    match video_processor.get_video_info(&PathBuf::from("output_video_from_func.mp4")) {
        Ok(info) => {
            info!("Video Info: {:?}", info);
        }
        Err(e) => {
            error!("Failed to get video info: {}", e);
        }
    }

    match video_processor
        .resize_video(
            &PathBuf::from("output_video_from_func.mp4"),
            &PathBuf::from("resized_video.mp4"),
            1280,
            720,
        )
        .await
    {
        Ok(_) => {
            info!("Video resized successfully.");
        }
        Err(e) => {
            error!("Failed to resize video: {}", e);
        }
    }
}
