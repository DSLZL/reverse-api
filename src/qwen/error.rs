use std::fmt;

#[derive(Debug)]
pub enum QwenError {
    ApiError(String),
    NetworkError(rquest::Error),
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
    WasmError(anyhow::Error),
    IoError(std::io::Error),
}

impl fmt::Display for QwenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QwenError::ApiError(msg) => write!(f, "API Error: {}", msg),
            QwenError::NetworkError(e) => write!(f, "Network Error: {}", e),
            QwenError::ReqwestError(e) => write!(f, "Reqwest Error: {}", e),
            QwenError::JsonError(e) => write!(f, "JSON Error: {}", e),
            QwenError::WasmError(e) => write!(f, "WASM Error: {}", e),
            QwenError::IoError(e) => write!(f, "IO Error: {}", e),
        }
    }
}

impl std::error::Error for QwenError {}

impl From<rquest::Error> for QwenError {
    fn from(error: rquest::Error) -> Self {
        QwenError::NetworkError(error)
    }
}

impl From<serde_json::Error> for QwenError {
    fn from(error: serde_json::Error) -> Self {
        QwenError::JsonError(error)
    }
}

impl From<anyhow::Error> for QwenError {
    fn from(error: anyhow::Error) -> Self {
        QwenError::WasmError(error)
    }
}

impl From<std::io::Error> for QwenError {
    fn from(error: std::io::Error) -> Self {
        QwenError::IoError(error)
    }
}

impl From<reqwest::Error> for QwenError {
    fn from(error: reqwest::Error) -> Self {
        QwenError::ReqwestError(error)
    }
}

pub type Result<T> = std::result::Result<T, QwenError>;
