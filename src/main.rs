mod audio;
mod config;
mod error;
mod lipsync;
mod translation;
mod video;
mod voice_synthesis;

use crate::config::Config;
use clap::{command, Parser};
use lipsync::LipSyncService;
use std::path::PathBuf;
use tracing::error;
use translation::GoogleTranslationService;

const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Parser, Debug)]
#[command(
    name = "AI Video Translator",
    version = "1.0.0",
    about = "AI-assisted video translation and lip sync tool"
)]
pub struct Args {
    #[arg(short, long)]
    pub input: PathBuf,

    #[arg(short, long)]
    pub output: PathBuf,

    #[arg(short = 's', long = "source-lang", default_value = "en-US")]
    pub source_lang: String,

    #[arg(short = 't', long = "target-lang")]
    pub target_lang: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let config = Config::load(DEFAULT_CONFIG_PATH).unwrap();
    let args = Args::parse();

    let audio_processor = audio::AudioProcessor::new(config.processing.audio_sample_rate);

    let _ = audio_processor
        .extract_audio(
            &PathBuf::from(&args.input),
            &PathBuf::from("extracted_audio.wav"),
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to extract audio: {}", e);
        });

    audio_processor
        .resample_audio(
            &PathBuf::from("extracted_audio.wav"),
            &PathBuf::from("final_audio.wav"),
            config.processing.stt_sample_rate,
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to resample audio: {}", e);
        });

    let audio_base64 = audio_processor
        .audio_to_base64(&PathBuf::from("final_audio.wav"))
        .unwrap_or_else(|e| {
            error!("Failed to convert audio to base64: {}", e);
            String::new()
        });

    let translation_processor = GoogleTranslationService::new(config.google.api_key);

    let transcript = translation_processor
        .transcribe_audio_with_alternatives(
            &audio_base64,
            &config.processing.alternative_languages,
            config.processing.stt_sample_rate,
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to transcribe audio: {}", e);
            String::new()
        });

    println!("Transcript: {}", transcript);

    if transcript.is_empty() {
        error!("No transcript available, exiting.");
        return;
    }

    let mut language_used = if !args.source_lang.is_empty() {
        args.source_lang.clone()
    } else {
        config.processing.alternative_languages[0].clone()
    };

    if language_used.is_empty() {
        language_used = translation_processor
            .detect_language(&transcript)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to detect language: {}", e);
                String::new()
            });
    };

    println!("Language: {}", language_used);

    let translated = translation_processor
        .translate_text(&transcript, &language_used, &args.target_lang)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to translate text: {}", e);
            String::new()
        });

    println!("Translated Text: {}", translated);

    // let's help eleven labs a bit
    // because they make a lot of pauses in the middle of sentences
    let mut translated = translated.replace('.', ",").to_lowercase();
    if let Some(find_last_comma) = translated.rfind(',') {
        translated.replace_range(find_last_comma..=find_last_comma, ".");
    }

    let voice_synthesis_processor =
        voice_synthesis::VoiceSynthesizer::new(config.elevenlabs.clone());

    let translated_audio = voice_synthesis_processor
        .text_to_speech(&translated, &config.elevenlabs.voice_id)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to synthesize voice: {}", e);
            Vec::new()
        });

    if translated_audio.is_empty() {
        error!("No audio generated, exiting.");
        return;
    }

    voice_synthesis_processor
        .save_audio(&translated_audio, &PathBuf::from("translated_audio.mp3"))
        .await
        .unwrap_or_else(|e| {
            error!("Failed to save audio: {}", e);
        });

    let video_processor = video::VideoProcessor::new(config.processing.video_fps);

    video_processor
        .extract_frames(
            &PathBuf::from(&args.input),
            &PathBuf::from("extracted_frames"),
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to extract frames: {}", e);
        });

    video_processor
        .combine_frames_with_audio(
            &PathBuf::from("extracted_frames"),
            &PathBuf::from("translated_audio.mp3"),
            &PathBuf::from(&args.output),
        )
        .await
        .unwrap_or_else(|e| {
            error!("Failed to combine frames with audio: {}", e);
        });

    let lip_sync_processor = LipSyncService::new();
    let ngrok_host = &config.lipsync.ngrok_url.trim_end_matches('/');

    let video_url = format!(
        "{}/{}",
        ngrok_host,
        args.input.file_name().unwrap().to_string_lossy()
    );
    let audio_url = format!("{}/translated_audio.mp3", ngrok_host);

    let final_url =
        LipSyncService::run_sync_client(&video_url, &audio_url, &config.lipsync.api_key).await;

    match final_url {
        Ok(Some(url)) => {
            println!("Lip synced video URL: {}", url);

            let output_path = PathBuf::from(&args.output);
            if let Err(e) = lip_sync_processor.download_result(&url, &output_path).await {
                error!("Failed to download lip synced video: {}", e);
            }
        }
        Ok(None) => {
            eprintln!("Lip sync generation failed in Python client.");
        }
        Err(e) => {
            error!("Error running lip sync client: {}", e);
        }
    }
}
