//! CLI error types and exit code mapping.

use thiserror::Error;

/// Exit codes following the specification:
/// - 0 = success
/// - 1 = validation error
/// - 2 = invariant violation
/// - 3 = IO error
#[allow(dead_code)]
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_VALIDATION_ERROR: i32 = 1;
pub const EXIT_INVARIANT_VIOLATION: i32 = 2;
pub const EXIT_IO_ERROR: i32 = 3;

/// Unified error type for all CLI operations.
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CliError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Project not initialized. Run `inst init` first.")]
    ProjectNotInitialized,

    #[error("Store error: {0}")]
    Store(String),

    #[error("Constraint error: {0}")]
    Constraint(String),

    #[error("Audit error: {0}")]
    Audit(String),

    #[error("{0}")]
    Other(String),
}

impl CliError {
    /// Map this error to the appropriate exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Validation(_) => EXIT_VALIDATION_ERROR,
            CliError::InvariantViolation(_) => EXIT_INVARIANT_VIOLATION,
            CliError::Io(_) => EXIT_IO_ERROR,
            CliError::Store(_) => EXIT_IO_ERROR,
            CliError::Serialization(_) => EXIT_VALIDATION_ERROR,
            CliError::NotFound(_) => EXIT_VALIDATION_ERROR,
            CliError::ProjectNotInitialized => EXIT_VALIDATION_ERROR,
            CliError::Constraint(_) => EXIT_INVARIANT_VIOLATION,
            CliError::Audit(_) => EXIT_IO_ERROR,
            CliError::Other(_) => EXIT_VALIDATION_ERROR,
        }
    }
}
