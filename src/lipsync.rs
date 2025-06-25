use crate::error::{CustomError, Result};
use reqwest::Client;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub struct LipSyncService {
    client: Client,
}

impl LipSyncService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn run_sync_client(
        video_url: &str,
        audio_url: &str,
        api_key: &str,
    ) -> Result<Option<String>> {
        let mut child = Command::new("python3")
            .arg("-u") // flush every print
            .arg("src/wav2lip.py")
            .arg("--video-url")
            .arg(video_url)
            .arg("--audio-url")
            .arg(audio_url)
            .arg("--api-key")
            .arg(api_key)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout).lines();

        let mut final_url = None;

        while let Some(line) = reader.next_line().await? {
            println!("[python] {}", line);

            if line.contains("completed successfully, output url:") {
                if let Some(url) = line.split("output url:").nth(1) {
                    final_url = Some(url.trim().to_string());
                }
            }
        }

        let status = child.wait().await?;

        if !status.success() {
            if let Some(stderr) = child.stderr.take() {
                let mut err_reader = BufReader::new(stderr).lines();

                while let Some(line) = err_reader.next_line().await? {
                    eprintln!("[python err] {}", line);
                }
            }

            return Err(CustomError::LipSync(format!(
                "Python script failed with status: {}",
                status
            )));
        }

        Ok(final_url)
    }

    pub async fn download_result(&self, url: &str, output_path: &std::path::Path) -> Result<()> {
        let bytes = self.client.get(url).send().await?.bytes().await?;

        tokio::fs::write(output_path, &bytes).await.map_err(|e| {
            CustomError::VoiceSynthesis(format!("Failed to save lip synced video: {}", e))
        })?;

        Ok(())
    }
}
