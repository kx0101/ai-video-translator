# AI Video Translator

**End-to-end AI-assisted video translation and dubbing pipeline built in Rust, integrates transcription, translation, voice synthesis, and lip syncing into a fully automated workflow**

---

## Why

Traditional video dubbing is manual and time-consuming. This tool automates the process using AI tooling, making it possible to localize content in any language with minimal effort, ideal for creators, educators and more!

![arch](https://github.com/user-attachments/assets/b1254380-7c4f-44e5-b73c-88a58ffd0be3)

## Examples

## Input
https://github.com/user-attachments/assets/48a4931d-1055-44df-9818-5b8b485da122

## Output (I couldn't clone his exact voice due to limitations with the Starter Plan of ElevenLabs)
https://github.com/user-attachments/assets/f440b48d-bdec-4c3c-8391-26491480cf00

## How It Works

Here's a breakdown of the pipeline:
```
    Audio Extraction → via FFmpeg

    Resampling → normalized to STT input requirements

    Transcription → via Google STT API

    Translation → auto-detected or user-defined source/target languages

    Voice Synthesis → ElevenLabs API

    Frame Extraction & Audio Merge → FFmpeg intermediate video

    Lip Sync → Sync.so REST API

    Final Output → Saved to disk
```

## Example

```sh
cargo run --release -- \
  --input ./example/input_video.mp4 \
  --output ./output/final_video.mp4 \
  --target-lang es
```

This will take input_video.mp4, detect its language (or use --source-lang if specified), translate its transcript to Spanish, generate a new voiceover, and produce a lip synced dubbed video as final_video.mp4.

## Configuration

Create a config.toml file in the root:

```bash
[processing]
audio_sample_rate = 44100
stt_sample_rate = 16000
video_fps = 30
alternative_languages = ["en-US", "el-GR", "es-ES", "fr-FR"]

[google]
api_key = "YOUR_GOOGLE_API_KEY"

[elevenlabs]
api_key = "YOUR_ELEVEN_LABS_KEY"
voice_id = "YOUR_VOICE_ID"

[lipsync]
api_key = "YOUR_SYNC_SO_API_KEY"
ngrok_url = "https://your-ngrok-tunnel.ngrok-free.app"
```

## Modules

`audio.rs` for audio extraction/resampling

`translation.rs` for transcription, translation, and language detection

`voice_synthesis.rs` for TTS using ElevenLabs

`video.rs` for frame extraction and assembly

`lipsync.rs` for Sync.so integration

`config.rs` for TOML config loading


## Important Notes

**The ngrok_url is required to expose your local video/audio files to Sync.so for processing**

**voice_id can be one of your custom or stock ElevenLabs voices**
