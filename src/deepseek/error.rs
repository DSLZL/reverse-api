use thiserror::Error;
use wasmtime::MemoryAccessError;

#[derive(Debug, Error)]
pub enum DeepSeekError {
    #[error("Rquest error: {0}")]
    Rquest(#[from] rquest::Error),
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Wasmtime error: {0}")]
    Wasmtime(#[from] anyhow::Error),
    #[error("Wasmtime memory access error: {0}")]
    WasmtimeMemoryAccess(#[from] MemoryAccessError),
}

pub type Result<T> = std::result::Result<T, DeepSeekError>;
