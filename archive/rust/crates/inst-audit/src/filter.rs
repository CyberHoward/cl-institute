use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entry::{AuditAction, AuditEntry};

/// Filter criteria for querying the audit log.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditFilter {
    /// Filter by action type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<AuditAction>,
    /// Filter by actor user ID (only matches `Actor::User` entries).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<Uuid>,
    /// Only include entries after this timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<DateTime<Utc>>,
    /// Only include entries before this timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<DateTime<Utc>>,
    /// Maximum number of entries to return.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

impl AuditFilter {
    /// Returns true if the given entry matches all active filter criteria.
    pub fn matches(&self, entry: &AuditEntry) -> bool {
        if let Some(ref action) = self.action {
            if entry.action != *action {
                return false;
            }
        }

        if let Some(actor_id) = self.actor_id {
            if entry.actor.user_id() != Some(actor_id) {
                return false;
            }
        }

        if let Some(after) = self.after {
            if entry.timestamp <= after {
                return false;
            }
        }

        if let Some(before) = self.before {
            if entry.timestamp >= before {
                return false;
            }
        }

        true
    }
}
