use anyhow::Context;

use crate::Args;
use crate::{audio, config::Config, lipsync, translation, video, voice_synthesis};
use lipsync::LipSyncService;
use std::path::Path;
use translation::GoogleTranslationService;

pub async fn run(args: Args, config: Config) -> Result<(), anyhow::Error> {
    let audio_processor = audio::AudioProcessor::new(config.processing.audio_sample_rate);

    audio_processor
        .extract_audio(&args.input, Path::new("extracted_audio.wav"))
        .await
        .context("extracting audio")?;

    audio_processor
        .resample_audio(
            Path::new("extracted_audio.wav"),
            Path::new("final_audio.wav"),
            config.processing.stt_sample_rate,
        )
        .await
        .context("resampling audio")?;

    let audio_base64 = audio_processor
        .audio_to_base64(Path::new("final_audio.wav"))
        .context("converting audio to base64")?;

    let translation_processor = GoogleTranslationService::new(config.google.api_key.clone());

    let transcript = translation_processor
        .transcribe_audio_with_alternatives(
            &audio_base64,
            &config.processing.alternative_languages,
            config.processing.stt_sample_rate,
        )
        .await
        .context("transcribing audio")?;

    if transcript.is_empty() {
        anyhow::bail!("Transcript is empty");
    }

    let language_used = if !args.source_lang.is_empty() {
        args.source_lang.clone()
    } else {
        translation_processor
            .detect_language(&transcript)
            .await
            .unwrap_or(config.processing.alternative_languages[0].clone())
    };

    let mut translated = translation_processor
        .translate_text(&transcript, &language_used, &args.target_lang)
        .await
        .context("translating text")?;

    // let's help eleven labs a bit
    // because they make a lot of pauses in the middle of sentences
    translated = translated.replace('.', ",").to_lowercase();
    if let Some(pos) = translated.rfind(',') {
        translated.replace_range(pos..=pos, ".");
    }

    let voice_synth = voice_synthesis::VoiceSynthesizer::new(config.elevenlabs.clone());

    let translated_audio = voice_synth
        .text_to_speech(&translated, &config.elevenlabs.voice_id)
        .await
        .context("synthesizing voice")?;

    if translated_audio.is_empty() {
        anyhow::bail!("Voice synthesis returned empty audio");
    }

    voice_synth
        .save_audio(&translated_audio, Path::new("translated_audio.mp3"))
        .await
        .context("saving synthesized audio")?;

    let video_processor = video::VideoProcessor::new(config.processing.video_fps);

    video_processor
        .extract_frames(&args.input, Path::new("extracted_frames"))
        .await
        .context("extracting frames")?;

    video_processor
        .combine_frames_with_audio(
            Path::new("extracted_frames"),
            Path::new("translated_audio.mp3"),
            &args.output,
        )
        .await
        .context("combining frames and audio")?;

    let lip_sync = LipSyncService::new();
    let ngrok_host = config.lipsync.ngrok_url.trim_end_matches('/');

    let video_url = format!(
        "{}/{}",
        ngrok_host,
        args.input.file_name().unwrap().to_string_lossy()
    );
    let audio_url = format!("{}/translated_audio.mp3", ngrok_host);

    if let Ok(Some(final_url)) =
        LipSyncService::run_sync_client(&video_url, &audio_url, &config.lipsync.api_key).await
    {
        println!("Lip synced video URL: {}", final_url);
        lip_sync
            .download_result(&final_url, &args.output)
            .await
            .context("downloading final video")?;
    } else {
        anyhow::bail!("Lip sync generation failed");
    }

    Ok(())
}
