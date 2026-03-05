use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{InstanceId, NodeId, RequirementId, SubmissionId, UserId, WorkflowId};

/// Runtime status of a workflow instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InstanceStatus {
    InProgress,
    Completed,
    Rejected,
    Cancelled,
}

/// A running instance of a workflow — the execution state.
///
/// Tracks the current position in the decision graph and
/// the trace of decisions made so far.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: InstanceId,
    pub workflow_id: WorkflowId,
    pub applicant_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_node_id: Option<NodeId>,
    pub status: InstanceStatus,
    /// Ordered trace of transitions and decisions made during execution.
    #[serde(default)]
    pub trace: Vec<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkflowInstance {
    pub fn new(workflow_id: WorkflowId, applicant_name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: InstanceId::new(),
            workflow_id,
            applicant_name: applicant_name.into(),
            current_node_id: None,
            status: InstanceStatus::InProgress,
            trace: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A document submitted to satisfy an edge requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSubmission {
    pub id: SubmissionId,
    pub edge_requirement_id: RequirementId,
    pub workflow_instance_id: InstanceId,
    pub submitted_by_user_id: UserId,
    pub file_name: String,
    pub file_path: String,
    pub file_size_bytes: i64,
    pub file_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub submitted_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_instance_new_starts_as_in_progress() {
        let wf_id = WorkflowId::new();
        let instance = WorkflowInstance::new(wf_id, "Alice Johnson");

        assert_eq!(instance.workflow_id, wf_id);
        assert_eq!(instance.applicant_name, "Alice Johnson");
        assert_eq!(instance.status, InstanceStatus::InProgress);
        assert!(instance.current_node_id.is_none());
        assert!(instance.trace.is_empty());
        assert_eq!(instance.created_at, instance.updated_at);
    }

    #[test]
    fn instance_unique_ids() {
        let wf_id = WorkflowId::new();
        let i1 = WorkflowInstance::new(wf_id, "Person A");
        let i2 = WorkflowInstance::new(wf_id, "Person B");
        assert_ne!(i1.id, i2.id);
    }

    #[test]
    fn instance_status_serializes_as_screaming_snake_case() {
        assert_eq!(serde_json::to_string(&InstanceStatus::InProgress).unwrap(), "\"IN_PROGRESS\"");
        assert_eq!(serde_json::to_string(&InstanceStatus::Completed).unwrap(), "\"COMPLETED\"");
        assert_eq!(serde_json::to_string(&InstanceStatus::Rejected).unwrap(), "\"REJECTED\"");
        assert_eq!(serde_json::to_string(&InstanceStatus::Cancelled).unwrap(), "\"CANCELLED\"");
    }

    #[test]
    fn instance_status_roundtrip_all_variants() {
        for status in [
            InstanceStatus::InProgress,
            InstanceStatus::Completed,
            InstanceStatus::Rejected,
            InstanceStatus::Cancelled,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: InstanceStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, status);
        }
    }

    #[test]
    fn document_submission_all_fields_present() {
        let submission = DocumentSubmission {
            id: SubmissionId::new(),
            edge_requirement_id: RequirementId::new(),
            workflow_instance_id: InstanceId::new(),
            submitted_by_user_id: UserId::new(),
            file_name: "contract.pdf".to_string(),
            file_path: "/uploads/2025/01/contract.pdf".to_string(),
            file_size_bytes: 1_048_576,
            file_type: "application/pdf".to_string(),
            notes: Some("Signed by both parties".to_string()),
            submitted_at: Utc::now(),
        };

        let json = serde_json::to_string(&submission).unwrap();
        let deserialized: DocumentSubmission = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, submission.id);
        assert_eq!(deserialized.edge_requirement_id, submission.edge_requirement_id);
        assert_eq!(deserialized.workflow_instance_id, submission.workflow_instance_id);
        assert_eq!(deserialized.submitted_by_user_id, submission.submitted_by_user_id);
        assert_eq!(deserialized.file_name, "contract.pdf");
        assert_eq!(deserialized.file_path, "/uploads/2025/01/contract.pdf");
        assert_eq!(deserialized.file_size_bytes, 1_048_576);
        assert_eq!(deserialized.file_type, "application/pdf");
        assert_eq!(deserialized.notes, Some("Signed by both parties".to_string()));
    }

    #[test]
    fn document_submission_notes_omitted_when_none() {
        let submission = DocumentSubmission {
            id: SubmissionId::new(),
            edge_requirement_id: RequirementId::new(),
            workflow_instance_id: InstanceId::new(),
            submitted_by_user_id: UserId::new(),
            file_name: "doc.txt".to_string(),
            file_path: "/uploads/doc.txt".to_string(),
            file_size_bytes: 100,
            file_type: "text/plain".to_string(),
            notes: None,
            submitted_at: Utc::now(),
        };

        let json = serde_json::to_string(&submission).unwrap();
        assert!(!json.contains("\"notes\""));
    }

    #[test]
    fn trace_starts_empty() {
        let wf_id = WorkflowId::new();
        let instance = WorkflowInstance::new(wf_id, "Test");
        assert!(instance.trace.is_empty());
    }

    #[test]
    fn trace_can_hold_arbitrary_json() {
        let wf_id = WorkflowId::new();
        let mut instance = WorkflowInstance::new(wf_id, "Test");
        instance.trace.push(serde_json::json!({
            "action": "transition",
            "from_node": "n1",
            "to_node": "n2",
            "timestamp": "2025-01-15T10:30:00Z"
        }));
        instance.trace.push(serde_json::json!({
            "action": "decision",
            "node": "n2",
            "result": "approved"
        }));

        let json = serde_json::to_string(&instance).unwrap();
        let deserialized: WorkflowInstance = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.trace.len(), 2);
        assert_eq!(deserialized.trace[0]["action"], "transition");
        assert_eq!(deserialized.trace[1]["result"], "approved");
    }

    #[test]
    fn workflow_instance_json_roundtrip() {
        let wf_id = WorkflowId::new();
        let node_id = NodeId::new();
        let mut instance = WorkflowInstance::new(wf_id, "Bob Smith");
        instance.current_node_id = Some(node_id);
        instance.status = InstanceStatus::Completed;

        let json = serde_json::to_string(&instance).unwrap();
        let deserialized: WorkflowInstance = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, instance.id);
        assert_eq!(deserialized.workflow_id, wf_id);
        assert_eq!(deserialized.applicant_name, "Bob Smith");
        assert_eq!(deserialized.current_node_id, Some(node_id));
        assert_eq!(deserialized.status, InstanceStatus::Completed);
    }
}
