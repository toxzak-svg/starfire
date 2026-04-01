//! Aion error types.

#[derive(thiserror::Error, Debug)]
pub enum AionError {
    #[error("Mind not found: {0}")]
    MindNotFound(String),

    #[error("Mind already exists: {0}")]
    MindAlreadyExists(String),

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Mind kind not registered: {0}")]
    MindKindNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Impulse error: {0}")]
    Impulse(String),

    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    #[error("Thought failed: {0}")]
    ThoughtFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    Io(String),
}

pub type AionResult<T> = Result<T, AionError>;

impl From<serde_json::Error> for AionError {
    fn from(e: serde_json::Error) -> Self {
        AionError::Serialization(e.to_string())
    }
}

impl From<rusqlite::Error> for AionError {
    fn from(e: rusqlite::Error) -> Self {
        AionError::Database(e.to_string())
    }
}

impl From<tokio::task::JoinError> for AionError {
    fn from(e: tokio::task::JoinError) -> Self {
        AionError::InvalidState(format!("task join error: {}", e))
    }
}

impl From<std::io::Error> for AionError {
    fn from(e: std::io::Error) -> Self {
        AionError::Io(e.to_string())
    }
}
