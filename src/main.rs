mod audio;
mod config;
mod error;
mod translation;
mod video;

use crate::config::Config;
use clap::{command, Parser};
use std::path::PathBuf;
use tracing::error;
use translation::GoogleTranslationService;

const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Parser, Debug)]
#[command(
    name = "Auto Dubbing",
    version = "1.0.0",
    about = "Automatic video dubbing with AI translation and lip sync"
)]
pub struct Args {
    #[arg(short, long)]
    pub input: PathBuf,

    // we cant generate the final video yet
    // #[arg(short, long)]
    // pub output: PathBuf,
    #[arg(short = 's', long = "source-lang", default_value = "en")]
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
}
