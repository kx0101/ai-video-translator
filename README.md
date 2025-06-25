# AI Video Translator

**AI-assisted video translation and lip sync tool built in Rust.**  

---

![arch](https://github.com/user-attachments/assets/b1254380-7c4f-44e5-b73c-88a58ffd0be3)

## Examples

## Input
https://github.com/user-attachments/assets/48a4931d-1055-44df-9818-5b8b485da122

## Output (I couldn't clone his exact voice due to limitations with the Starter Plan of ElevenLabs)
https://github.com/user-attachments/assets/f440b48d-bdec-4c3c-8391-26491480cf00

## Features

- Audio extraction and resampling
- Transcription using Google Speech-to-Text (with language detection support)
- Translation to any target language (via Google Translate API)
- Voice synthesis using ElevenLabs TTS
- Accurate lip syncing using Sync.so
- Reassembled video with translated voice-over and synced lips

---

## How It Works

Here's a breakdown of the pipeline:

1. **Audio Extraction**  
   Extracts audio from the input video using FFmpeg.

2. **Resampling**  
   Converts it to the sample rate expected by the STT API.

3. **Transcription**  
   Uses Google Cloud's Speech-to-Text to transcribe the spoken content.

4. **Language Detection & Translation**  
   Detects source language (or uses the provided one) and translates the transcript using Google Translate API.

5. **Voice Synthesis**  
   Sends the translated text to ElevenLabs to synthesize speech in a target voice.

6. **Frame Extraction & Video Rebuild**  
   Extracts video frames and combines them with the translated audio as a temporary output.

7. **Lip Sync**  
   Uses [Sync.so](https://www.sync.so) to generate a final video where lip movements match the new audio perfectly.

8. **Final Output**  
   Downloads and saves the final lip synced, dubbed video to disk.

---

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
