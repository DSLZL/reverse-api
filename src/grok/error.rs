use thiserror::Error;

#[derive(Error, Debug)]
pub enum GrokError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Rquest error: {0}")]
    RquestError(#[from] rquest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid proxy format: {0}")]
    InvalidProxy(String),

    #[error("Parsing error: {0}")]
    ParseError(String),

    #[error("Anti-bot rejection")]
    AntiBotRejection,

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Crypto error: {0}")]
    CryptoError(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GrokError>;
