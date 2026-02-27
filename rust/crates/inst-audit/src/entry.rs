use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Categories of auditable events in the institutional model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    // Model mutations
    OrganizationCreated,
    OrganizationUpdated,
    RoleCreated,
    RoleUpdated,
    RoleDeleted,
    WorkflowCreated,
    WorkflowUpdated,
    NodeCreated,
    NodeUpdated,
    NodeDeleted,
    EdgeCreated,
    EdgeUpdated,
    EdgeDeleted,
    PolicyAttached,
    PolicyUpdated,
    PolicyDetached,
    IntegrationRegistered,
    IntegrationUpdated,
    // Runtime events
    InstanceCreated,
    DecisionMade,
    TransitionExecuted,
    DocumentSubmitted,
    InstanceCompleted,
    InstanceCancelled,
    // Agent events
    AgentRecommendation,
    AgentDecision,
    OverrideApplied,
}

/// The actor who performed an auditable action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Actor {
    User {
        user_id: Uuid,
        display_name: String,
    },
    Agent {
        agent_id: String,
        role: String,
    },
    System,
}

impl Actor {
    /// Returns the UUID of the actor if it is a `User`, `None` otherwise.
    pub fn user_id(&self) -> Option<Uuid> {
        match self {
            Actor::User { user_id, .. } => Some(*user_id),
            _ => None,
        }
    }
}

/// A single entry in the audit log.
///
/// Each entry is cryptographically chained to the previous entry via `prev_hash`
/// and `entry_hash`, forming a tamper-evident append-only log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub action: AuditAction,
    pub actor: Actor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_state: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_state: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    pub prev_hash: String,
    pub entry_hash: String,
}

/// An intermediate representation used for hashing — identical to `AuditEntry`
/// but without `entry_hash`, since that field is computed from all other fields.
#[derive(Serialize)]
struct AuditEntryForHashing {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    pub action: AuditAction,
    pub actor: Actor,
    pub prior_state: Option<serde_json::Value>,
    pub new_state: Option<serde_json::Value>,
    pub reasoning: Option<String>,
    pub prev_hash: String,
}

impl AuditEntry {
    /// Compute the SHA-256 hash of this entry (over all fields except `entry_hash`).
    ///
    /// The hash is computed from the canonical JSON serialization of the entry
    /// with `entry_hash` excluded.
    pub fn compute_hash(&self) -> String {
        let hashable = AuditEntryForHashing {
            id: self.id,
            timestamp: self.timestamp,
            sequence: self.sequence,
            action: self.action.clone(),
            actor: self.actor.clone(),
            prior_state: self.prior_state.clone(),
            new_state: self.new_state.clone(),
            reasoning: self.reasoning.clone(),
            prev_hash: self.prev_hash.clone(),
        };

        let canonical_json =
            serde_json::to_string(&hashable).expect("AuditEntry serialization should not fail");

        let mut hasher = Sha256::new();
        hasher.update(canonical_json.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify that this entry's `entry_hash` is correct.
    pub fn verify_hash(&self) -> bool {
        self.entry_hash == self.compute_hash()
    }
}

/// Inline hex encoding — avoids adding an extra dependency for a small utility.
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}
