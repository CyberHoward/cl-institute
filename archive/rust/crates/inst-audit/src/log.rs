use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use crate::entry::{Actor, AuditAction, AuditEntry};
use crate::error::AuditError;
use crate::filter::AuditFilter;
use crate::verification::ChainVerification;

/// The sentinel prev_hash value for the first entry in the chain.
const GENESIS_PREV_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// The main audit log handle for reading and writing entries.
///
/// Operates on a JSONL file (one JSON object per line). Each entry is
/// cryptographically chained to the previous entry via SHA-256 hashes,
/// providing tamper evidence.
pub struct AuditLog {
    /// Path to the JSONL audit log file.
    path: PathBuf,
    /// The hash of the last entry written (used as `prev_hash` for the next entry).
    last_hash: String,
    /// The sequence number of the last entry written.
    last_sequence: u64,
}

impl AuditLog {
    /// Open an existing audit log file, or create a new one if it does not exist.
    ///
    /// When opening an existing log, the file is scanned to determine the
    /// last entry's hash and sequence number so that new entries can be
    /// correctly chained.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AuditError> {
        let path = path.as_ref().to_path_buf();

        // Ensure parent directories exist.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create the file if it doesn't exist.
        if !path.exists() {
            fs::File::create(&path)?;
        }

        // Scan for the last entry to recover chain state.
        let (last_hash, last_sequence) = Self::recover_chain_tail(&path)?;

        Ok(Self {
            path,
            last_hash,
            last_sequence,
        })
    }

    /// Append a new entry to the audit log.
    ///
    /// Automatically assigns a UUID, timestamp, sequence number, and
    /// computes the cryptographic hash chain fields.
    pub fn append(
        &mut self,
        action: AuditAction,
        actor: Actor,
        prior_state: Option<serde_json::Value>,
        new_state: Option<serde_json::Value>,
        reasoning: Option<String>,
    ) -> Result<AuditEntry, AuditError> {
        let sequence = self.last_sequence + 1;
        let prev_hash = self.last_hash.clone();

        // Build the entry with a placeholder hash, then compute the real one.
        let mut entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            sequence,
            action,
            actor,
            prior_state,
            new_state,
            reasoning,
            prev_hash,
            entry_hash: String::new(),
        };

        entry.entry_hash = entry.compute_hash();

        // Serialize and append to the file as a single JSONL line.
        let json_line = serde_json::to_string(&entry)?;

        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        writeln!(file, "{json_line}")?;

        // Update internal chain state.
        self.last_hash = entry.entry_hash.clone();
        self.last_sequence = sequence;

        Ok(entry)
    }

    /// Read all entries from the audit log.
    pub fn read_all(&self) -> Result<Vec<AuditEntry>, AuditError> {
        Self::read_entries_from_file(&self.path)
    }

    /// Read the last `n` entries from the audit log.
    ///
    /// If the log contains fewer than `n` entries, all entries are returned.
    pub fn read_last(&self, n: usize) -> Result<Vec<AuditEntry>, AuditError> {
        let all = self.read_all()?;
        let start = all.len().saturating_sub(n);
        Ok(all[start..].to_vec())
    }

    /// Verify the integrity of the entire hash chain.
    ///
    /// Checks that:
    /// 1. The first entry has the genesis `prev_hash`.
    /// 2. Each entry's `entry_hash` matches the recomputed hash.
    /// 3. Each entry's `prev_hash` matches the preceding entry's `entry_hash`.
    /// 4. Sequence numbers are consecutive starting from 1.
    pub fn verify_chain(&self) -> Result<ChainVerification, AuditError> {
        let entries = self.read_all()?;

        if entries.is_empty() {
            return Ok(ChainVerification::valid(0));
        }

        let mut expected_prev_hash = GENESIS_PREV_HASH.to_string();
        let mut expected_sequence: u64 = 1;

        for (i, entry) in entries.iter().enumerate() {
            let entries_checked = (i as u64) + 1;

            // Verify sequence number.
            if entry.sequence != expected_sequence {
                return Ok(ChainVerification::invalid(
                    entries_checked,
                    entry.sequence,
                    format!(
                        "Expected sequence {expected_sequence}, found {}",
                        entry.sequence
                    ),
                ));
            }

            // Verify prev_hash linkage.
            if entry.prev_hash != expected_prev_hash {
                return Ok(ChainVerification::invalid(
                    entries_checked,
                    entry.sequence,
                    format!(
                        "prev_hash mismatch: expected {expected_prev_hash}, found {}",
                        entry.prev_hash
                    ),
                ));
            }

            // Verify entry_hash integrity.
            if !entry.verify_hash() {
                return Ok(ChainVerification::invalid(
                    entries_checked,
                    entry.sequence,
                    format!(
                        "entry_hash mismatch: stored {}, computed {}",
                        entry.entry_hash,
                        entry.compute_hash()
                    ),
                ));
            }

            expected_prev_hash = entry.entry_hash.clone();
            expected_sequence += 1;
        }

        Ok(ChainVerification::valid(entries.len() as u64))
    }

    /// Query the log with a filter, returning matching entries.
    ///
    /// Entries are returned in log order (oldest first). The `limit` field
    /// in the filter caps the number of results.
    pub fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>, AuditError> {
        let all = self.read_all()?;

        let mut results: Vec<AuditEntry> = all.into_iter().filter(|e| filter.matches(e)).collect();

        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Scan the file to find the hash and sequence of the last entry.
    ///
    /// Returns the genesis hash and sequence 0 if the file is empty.
    fn recover_chain_tail(path: &Path) -> Result<(String, u64), AuditError> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut last_hash = GENESIS_PREV_HASH.to_string();
        let mut last_sequence: u64 = 0;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: AuditEntry = serde_json::from_str(trimmed)?;
            last_hash = entry.entry_hash;
            last_sequence = entry.sequence;
        }

        Ok((last_hash, last_sequence))
    }

    /// Read and deserialize all entries from the JSONL file.
    fn read_entries_from_file(path: &Path) -> Result<Vec<AuditEntry>, AuditError> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: AuditEntry = serde_json::from_str(trimmed)?;
            entries.push(entry);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{Actor, AuditAction};
    use tempfile::NamedTempFile;

    /// Helper: create an AuditLog backed by a temporary file.
    fn temp_log() -> (AuditLog, NamedTempFile) {
        let file = NamedTempFile::new().expect("failed to create temp file");
        let log = AuditLog::open(file.path()).expect("failed to open audit log");
        (log, file)
    }

    #[test]
    fn append_and_read_single_entry() {
        let (mut log, _tmp) = temp_log();

        let entry = log
            .append(
                AuditAction::OrganizationCreated,
                Actor::System,
                None,
                Some(serde_json::json!({"name": "Acme"})),
                Some("Initial creation".into()),
            )
            .unwrap();

        assert_eq!(entry.sequence, 1);
        assert_eq!(entry.prev_hash, GENESIS_PREV_HASH);
        assert!(entry.verify_hash());

        let all = log.read_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, entry.id);
    }

    #[test]
    fn chain_links_correctly() {
        let (mut log, _tmp) = temp_log();

        let e1 = log
            .append(AuditAction::RoleCreated, Actor::System, None, None, None)
            .unwrap();

        let e2 = log
            .append(AuditAction::RoleUpdated, Actor::System, None, None, None)
            .unwrap();

        let e3 = log
            .append(AuditAction::RoleDeleted, Actor::System, None, None, None)
            .unwrap();

        assert_eq!(e2.prev_hash, e1.entry_hash);
        assert_eq!(e3.prev_hash, e2.entry_hash);
        assert_eq!(e1.sequence, 1);
        assert_eq!(e2.sequence, 2);
        assert_eq!(e3.sequence, 3);
    }

    #[test]
    fn verify_chain_on_valid_log() {
        let (mut log, _tmp) = temp_log();

        for _ in 0..5 {
            log.append(AuditAction::WorkflowCreated, Actor::System, None, None, None)
                .unwrap();
        }

        let result = log.verify_chain().unwrap();
        assert!(result.is_valid);
        assert_eq!(result.entries_checked, 5);
        assert!(result.first_invalid_sequence.is_none());
        assert!(result.error.is_none());
    }

    #[test]
    fn verify_chain_detects_tampering() {
        let (mut log, tmp) = temp_log();

        log.append(AuditAction::NodeCreated, Actor::System, None, None, None)
            .unwrap();
        log.append(AuditAction::NodeUpdated, Actor::System, None, None, None)
            .unwrap();

        // Tamper with the file: overwrite it with a modified second entry.
        let mut entries = log.read_all().unwrap();
        entries[1].reasoning = Some("tampered!".into());
        // Don't recompute the hash — simulating external tampering.

        let mut file = std::fs::File::create(tmp.path()).unwrap();
        for entry in &entries {
            let line = serde_json::to_string(entry).unwrap();
            writeln!(file, "{line}").unwrap();
        }

        // Re-open and verify.
        let log2 = AuditLog::open(tmp.path()).unwrap();
        let result = log2.verify_chain().unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.first_invalid_sequence, Some(2));
    }

    #[test]
    fn verify_empty_log_is_valid() {
        let (log, _tmp) = temp_log();
        let result = log.verify_chain().unwrap();
        assert!(result.is_valid);
        assert_eq!(result.entries_checked, 0);
    }

    #[test]
    fn read_last_n() {
        let (mut log, _tmp) = temp_log();

        for _ in 0..10 {
            log.append(AuditAction::EdgeCreated, Actor::System, None, None, None)
                .unwrap();
        }

        let last3 = log.read_last(3).unwrap();
        assert_eq!(last3.len(), 3);
        assert_eq!(last3[0].sequence, 8);
        assert_eq!(last3[2].sequence, 10);

        // More than total returns all.
        let last100 = log.read_last(100).unwrap();
        assert_eq!(last100.len(), 10);
    }

    #[test]
    fn query_by_action() {
        let (mut log, _tmp) = temp_log();

        log.append(AuditAction::PolicyAttached, Actor::System, None, None, None)
            .unwrap();
        log.append(AuditAction::NodeCreated, Actor::System, None, None, None)
            .unwrap();
        log.append(AuditAction::PolicyAttached, Actor::System, None, None, None)
            .unwrap();

        let results = log
            .query(AuditFilter {
                action: Some(AuditAction::PolicyAttached),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn query_by_actor_id() {
        let (mut log, _tmp) = temp_log();

        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();

        log.append(
            AuditAction::DecisionMade,
            Actor::User {
                user_id,
                display_name: "Alice".into(),
            },
            None,
            None,
            None,
        )
        .unwrap();

        log.append(
            AuditAction::DecisionMade,
            Actor::User {
                user_id: other_id,
                display_name: "Bob".into(),
            },
            None,
            None,
            None,
        )
        .unwrap();

        log.append(AuditAction::AgentDecision, Actor::System, None, None, None)
            .unwrap();

        let results = log
            .query(AuditFilter {
                actor_id: Some(user_id),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].actor.user_id(), Some(user_id));
    }

    #[test]
    fn query_with_limit() {
        let (mut log, _tmp) = temp_log();

        for _ in 0..10 {
            log.append(AuditAction::TransitionExecuted, Actor::System, None, None, None)
                .unwrap();
        }

        let results = log
            .query(AuditFilter {
                limit: Some(3),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].sequence, 1);
    }

    #[test]
    fn reopen_continues_chain() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        // First session: write two entries.
        {
            let mut log = AuditLog::open(&path).unwrap();
            log.append(AuditAction::InstanceCreated, Actor::System, None, None, None)
                .unwrap();
            log.append(
                AuditAction::InstanceCompleted,
                Actor::System,
                None,
                None,
                None,
            )
            .unwrap();
        }

        // Second session: reopen and append.
        {
            let mut log = AuditLog::open(&path).unwrap();
            let e3 = log
                .append(
                    AuditAction::InstanceCancelled,
                    Actor::System,
                    None,
                    None,
                    None,
                )
                .unwrap();
            assert_eq!(e3.sequence, 3);
        }

        // Verify the full chain.
        let log = AuditLog::open(&path).unwrap();
        let result = log.verify_chain().unwrap();
        assert!(result.is_valid);
        assert_eq!(result.entries_checked, 3);
    }

    #[test]
    fn entry_hash_is_deterministic() {
        let (mut log, _tmp) = temp_log();

        let entry = log
            .append(
                AuditAction::OverrideApplied,
                Actor::Agent {
                    agent_id: "agent-001".into(),
                    role: "compliance".into(),
                },
                Some(serde_json::json!({"status": "pending"})),
                Some(serde_json::json!({"status": "approved"})),
                Some("Override due to emergency protocol".into()),
            )
            .unwrap();

        // Computing the hash again should yield the same result.
        assert_eq!(entry.entry_hash, entry.compute_hash());
    }
}
