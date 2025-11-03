use std::fmt;

#[derive(Debug)]
pub enum ZtoError {
    TokenFetch(String),
    RequestFailed(String),
    ParseError(String),
    UnauthorizedError(String),
    ServerError(String),
    Other(String),
}

impl fmt::Display for ZtoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZtoError::TokenFetch(msg) => write!(f, "Token fetch error: {}", msg),
            ZtoError::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
            ZtoError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ZtoError::UnauthorizedError(msg) => write!(f, "Unauthorized: {}", msg),
            ZtoError::ServerError(msg) => write!(f, "Server error: {}", msg),
            ZtoError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ZtoError {}

pub type Result<T> = std::result::Result<T, ZtoError>;
