use thiserror::Error;

/// ChatGPT client errors
#[derive(Error, Debug)]
pub enum ChatGptError {
    #[error("Network error: {0}")]
    Network(#[from] rquest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Invalid proxy format: {0}")]
    InvalidProxy(String),

    #[error("Challenge solve error: {0}")]
    ChallengeSolve(String),

    #[error("VM execution error: {0}")]
    VmExecution(String),

    #[error("IP flagged: Your IP got flagged by ChatGPT")]
    IpFlagged,

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for ChatGPT operations
pub type Result<T> = std::result::Result<T, ChatGptError>;

impl ChatGptError {
    pub fn invalid_proxy(msg: impl Into<String>) -> Self {
        Self::InvalidProxy(msg.into())
    }

    pub fn challenge_solve(msg: impl Into<String>) -> Self {
        Self::ChallengeSolve(msg.into())
    }

    pub fn vm_execution(msg: impl Into<String>) -> Self {
        Self::VmExecution(msg.into())
    }

    pub fn authentication(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }

    pub fn invalid_response(msg: impl Into<String>) -> Self {
        Self::InvalidResponse(msg.into())
    }

    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    pub fn unknown(msg: impl Into<String>) -> Self {
        Self::Unknown(msg.into())
    }
}
