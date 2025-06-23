use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Audio processing error: {0}")]
    AudioProcessing(String),

    #[error("Video processing error: {0}")]
    VideoProcessing(String),

    #[error("Translation error: {0}")]
    Translation(String),

    #[error("Video synthesis error: {0}")]
    VideoSynthesis(String),

    #[error("Lip sync error: {0}")]
    LipSync(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CustomError>;
