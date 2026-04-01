use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum KbError {
    #[error("failed to download model: {0}")]
    ModelDownloadFailed(String),

    #[error("failed to index {0}: {1}")]
    IndexingFailed(String, String),

    #[error("database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("entry {0} not found")]
    NotFound(i64),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("embedding error: {0}")]
    EmbeddingError(String),

    #[error("embedding error (ort): {0}")]
    EmbeddingErrorOrt(String),

    #[error("HTTP error: {0}")]
    HttpError(String),
}

pub type Result<T> = std::result::Result<T, KbError>;
