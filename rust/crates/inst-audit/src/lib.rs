//! Append-only, tamper-evident audit log for the Intelligent Institution Initiative.
//!
//! Every mutation to the institutional model and every runtime decision is recorded
//! as an [`AuditEntry`] in a JSONL file with cryptographic hash chaining for
//! tamper evidence.

mod entry;
mod error;
mod filter;
mod log;
mod verification;

pub use entry::{Actor, AuditAction, AuditEntry};
pub use error::AuditError;
pub use filter::AuditFilter;
pub use log::AuditLog;
pub use verification::ChainVerification;
