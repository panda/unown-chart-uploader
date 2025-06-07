use thiserror::Error;

#[derive(Error, Debug)]
pub enum UploadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Server error (code {code}): {message}")]
    ServerError { code: u64, message: String },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, UploadError>;
