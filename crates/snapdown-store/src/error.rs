use snapdown_core::error::CoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Database corruption detected: {0}")]
    Corruption(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Vault confinement error: {0}")]
    Confinement(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid setting value: {0}")]
    InvalidValue(String),
}

impl From<StoreError> for CoreError {
    fn from(err: StoreError) -> Self {
        match err {
            StoreError::Confinement(msg) => CoreError::InvalidPath(msg),
            StoreError::Io(e) if e.kind() == std::io::ErrorKind::NotFound => {
                CoreError::NotFound(e.to_string())
            }
            StoreError::InvalidValue(msg) => CoreError::Validation(msg),
            other => CoreError::Validation(other.to_string()),
        }
    }
}
