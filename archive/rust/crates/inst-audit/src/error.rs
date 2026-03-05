use thiserror::Error;

/// Errors that can occur during audit log operations.
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Chain integrity violation at sequence {sequence}: {message}")]
    ChainIntegrity { sequence: u64, message: String },

    #[error("Audit log is empty")]
    EmptyLog,

    #[error("Invalid entry: {0}")]
    InvalidEntry(String),
}
