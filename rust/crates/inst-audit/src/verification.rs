use serde::{Deserialize, Serialize};

/// Result of verifying the integrity of the audit log's hash chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerification {
    /// Whether the entire chain is valid.
    pub is_valid: bool,
    /// Total number of entries that were checked.
    pub entries_checked: u64,
    /// The sequence number of the first entry that failed verification,
    /// if any.
    pub first_invalid_sequence: Option<u64>,
    /// Human-readable description of the first error encountered, if any.
    pub error: Option<String>,
}

impl ChainVerification {
    /// Construct a successful verification result.
    pub fn valid(entries_checked: u64) -> Self {
        Self {
            is_valid: true,
            entries_checked,
            first_invalid_sequence: None,
            error: None,
        }
    }

    /// Construct a failed verification result.
    pub fn invalid(entries_checked: u64, first_invalid_sequence: u64, error: String) -> Self {
        Self {
            is_valid: false,
            entries_checked,
            first_invalid_sequence: Some(first_invalid_sequence),
            error: Some(error),
        }
    }
}
