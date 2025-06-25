use crate::error::{CustomError, Result};
use std::path::Path;
use std::process::Command;
use tracing::info;

pub struct VideoProcessor {
    fps: f64,
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub fps: f64,
}

impl VideoProcessor {
    pub fn new(fps: f64) -> Self {
        Self { fps }
    }

    pub async fn extract_frames(&self, video_path: &Path, frames_dir: &Path) -> Result<()> {
        info!("Extracting frames from video: {}", video_path.display());

        std::fs::create_dir_all(frames_dir).map_err(|e| {
            CustomError::VideoProcessing(format!("Failed to create frames directory: {}", e))
        })?;

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                video_path.to_str().unwrap(),
                "-vf",
                &format!("fps={}", self.fps),
                &format!("{}/frame_%06d.png", frames_dir.to_str().unwrap()),
            ])
            .output()
            .map_err(|e| CustomError::VideoProcessing(format!("Failed to run ffmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(CustomError::VideoProcessing(format!(
                "ffmpeg failed with error: {}",
                error
            )));
        }

        info!("Frames extracted successfully to: {}", frames_dir.display());
        Ok(())
    }

    pub async fn combine_frames_with_audio(
        &self,
        frames_dir: &Path,
        audio_path: &Path,
        output_path: &Path,
    ) -> Result<()> {
        info!(
            "Combining frames with audio into video: {}",
            output_path.display()
        );

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-framerate",
                &self.fps.to_string(),
                "-i",
                &format!("{}/frame_%06d.png", frames_dir.to_str().unwrap()),
                "-i",
                audio_path.to_str().unwrap(),
                "-c:v",
                "libx264",
                "-preset",
                "slow", // slower preset = better compression
                "-crf",
                "23",
                "-c:a",
                "aac",
                "-b:a",
                "128k", // audio bitrate
                "-pix_fmt",
                "yuv420p",
                "-movflags",
                "+faststart", // for better streaming compatibility
                "-shortest",
                output_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| CustomError::VideoProcessing(format!("Failed to run ffmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(CustomError::VideoProcessing(format!(
                "ffmpeg failed with error: {}",
                error
            )));
        }

        info!("Video created successfully at: {}", output_path.display());
        Ok(())
    }

    pub fn get_video_info(&self, video_path: &Path) -> Result<VideoInfo> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                video_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| CustomError::VideoProcessing(format!("Failed to run ffprobe: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(CustomError::VideoProcessing(format!(
                "ffprobe failed with error: {}",
                error
            )));
        }

        let info_str = String::from_utf8_lossy(&output.stdout);
        let info: serde_json::Value = serde_json::from_str(&info_str).map_err(|e| {
            CustomError::VideoProcessing(format!("Failed to parse video info: {}", e))
        })?;

        let video_stream = info["streams"]
            .as_array()
            .and_then(|streams| {
                streams
                    .iter()
                    .find(|stream| stream["codec_type"].as_str() == Some("video"))
            })
            .ok_or_else(|| CustomError::VideoProcessing("No video stream found".to_string()))?;

        let width = &video_stream["width"];
        let height = &video_stream["height"];
        let duration = video_stream["duration"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| CustomError::VideoProcessing("Invalid video duration".to_string()))?;

        Ok(VideoInfo {
            width: width.as_u64().unwrap_or(0) as u32,
            height: height.as_u64().unwrap_or(0) as u32,
            duration,
            fps: self.fps,
        })
    }

    pub async fn resize_video(
        &self,
        input_path: &Path,
        output_path: &Path,
        width: u32,
        height: u32,
    ) -> Result<()> {
        info!(
            "Resizing video: {}, with width = {} and height = {}",
            input_path.display(),
            width,
            height
        );

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                input_path.to_str().unwrap(),
                "-vf",
                &format!("scale={}:{}", width, height),
                "-c:a",
                "copy",
                output_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| CustomError::VideoProcessing(format!("Failed to run ffmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(CustomError::VideoProcessing(format!(
                "ffmpeg resize failed with error: {}",
                error
            )));
        }

        info!("Video resized successfully to: {}", output_path.display());
        Ok(())
    }
}
