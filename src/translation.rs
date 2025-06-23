use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Serialize)]
struct RecognitionConfig {
    encoding: String,

    #[serde(rename = "sampleRateHertz")]
    sample_rate_hertz: u32,

    #[serde(rename = "languageCode")]
    language_code: String,

    #[serde(
        rename = "alternativeLanguageCodes",
        skip_serializing_if = "Option::is_none"
    )]
    alternative_language_codes: Option<Vec<String>>,

    #[serde(rename = "enableWordTimeOffsets")]
    enable_word_time_offsets: Option<bool>,
}

#[derive(Debug, Serialize)]
struct RecognitionAudio {
    content: String,
}

#[derive(Debug, Serialize)]
struct GoogleSpeechRequest {
    config: RecognitionConfig,
    audio: RecognitionAudio,
}

#[derive(Debug, Deserialize)]
struct GoogleSpeechResponse {
    results: Vec<RecognitionResult>,
}

#[derive(Debug, Deserialize)]
struct RecognitionResult {
    alternatives: Vec<RecognitionAlternative>,
}

#[derive(Debug, Deserialize)]
struct RecognitionAlternative {
    transcript: String,
}

#[derive(Debug, Serialize)]
struct GoogleTranslateRequest {
    q: String,
    source: String,
    target: String,
    format: String,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslateResponse {
    data: TranslationData,
}

#[derive(Debug, Deserialize)]
struct TranslationData {
    translations: Vec<TranslatedText>,
}

#[derive(Debug, Deserialize)]
struct TranslatedText {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[derive(Debug, Deserialize)]
struct GoogleDetectLanguageResponse {
    data: DetectLanguageData,
}

#[derive(Debug, Deserialize)]
struct DetectLanguageData {
    detections: Vec<Vec<LanguageDetection>>,
}

#[derive(Debug, Deserialize)]
struct LanguageDetection {
    language: String,
}

pub struct GoogleTranslationService {
    client: Client,
    api_key: String,
}

impl GoogleTranslationService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn transcribe_audio_with_alternatives(
        &self,
        audio_base64: &str,
        languages: &[String; 4],
        sample_rate: u32,
    ) -> Result<String> {
        let primary_lang = languages
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("No primary language provided"))?;

        let alt_langs = if languages.len() > 1 {
            Some(languages[1..].iter().map(|s| s.to_string()).collect())
        } else {
            None
        };

        info!(
            "Transcribing with primary: {}, alternatives: {:?}",
            primary_lang, alt_langs
        );

        let request = GoogleSpeechRequest {
            config: RecognitionConfig {
                encoding: "LINEAR16".to_string(),
                sample_rate_hertz: sample_rate,
                language_code: primary_lang.to_string(),
                alternative_language_codes: alt_langs,
                enable_word_time_offsets: Some(false),
            },
            audio: RecognitionAudio {
                content: audio_base64.to_string(),
            },
        };

        let url = format!(
            "https://speech.googleapis.com/v1/speech:recognize?key={}",
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Google Speech-to-Text API")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Google STT API error: {:#?}",
                response.text().await
            ));
        }

        let text = response
            .text()
            .await
            .context("Failed to read STT response text")?;

        info!("Raw STT response: {}", text);

        let parsed: GoogleSpeechResponse =
            serde_json::from_str(&text).context("Failed to parse Google STT response")?;

        let transcript = parsed
            .results
            .iter()
            .flat_map(|r| r.alternatives.iter())
            .filter_map(|alt| {
                if !alt.transcript.is_empty() {
                    Some(alt.transcript.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        if transcript.trim().is_empty() {
            return Err(anyhow::anyhow!("Transcription result was empty"));
        }

        Ok(transcript)
    }

    pub async fn detect_language(&self, text: &str) -> Result<String> {
        info!("Detecting language...");

        let url = format!(
            "https://translation.googleapis.com/language/translate/v2/detect?key={}",
            self.api_key
        );

        #[derive(Serialize)]
        struct DetectRequest<'a> {
            q: &'a str,
        }

        let request = DetectRequest { q: text };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send language detection request")?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Google Detect API error: {}", err_text));
        }

        let parsed: GoogleDetectLanguageResponse = response
            .json()
            .await
            .context("Failed to parse detect language response")?;

        let language_code = parsed
            .data
            .detections
            .get(0)
            .and_then(|vec| vec.get(0))
            .map(|det| det.language.clone())
            .unwrap_or_else(|| "und".to_string()); // 'und' = undetermined

        info!("Detected language: {}", language_code);
        Ok(language_code)
    }

    pub async fn translate_text(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        info!("Translating text to {}", target_lang);

        let request = GoogleTranslateRequest {
            q: text.to_string(),
            source: source_lang.to_string(),
            target: target_lang.to_string(),
            format: "text".to_string(),
        };

        let url = format!(
            "https://translation.googleapis.com/language/translate/v2?key={}",
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Google Translate API")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Google STT API error: {:#?}",
                response.text().await
            ));
        }

        let text = response
            .text()
            .await
            .context("Failed to read Translate response text")?;

        let parsed: GoogleTranslateResponse =
            serde_json::from_str(&text).context("Failed to parse Google Translate response")?;

        Ok(parsed
            .data
            .translations
            .get(0)
            .map(|t| t.translated_text.clone())
            .ok_or_else(|| anyhow::anyhow!("No translation found in response"))?)
    }
}
