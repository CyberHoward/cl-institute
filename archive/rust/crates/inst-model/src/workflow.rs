use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{FunctionId, OrganizationId, UserId, VersionId, WorkflowId};

/// A functional grouping of related workflows within an organization.
///
/// Maps to `public.functions`.
/// Example: "Procurement", "Compliance", "HR Onboarding".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub id: FunctionId,
    pub organization_id: OrganizationId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Function {
    pub fn new(organization_id: OrganizationId, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: FunctionId::new(),
            organization_id,
            name: name.into(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A workflow definition — a graph of decision nodes connected by edges.
///
/// Maps to `public.workflows`.
/// The workflow is the central organizational unit: it contains nodes (decision points),
/// edges (transitions), and attached policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: WorkflowId,
    pub organization_id: OrganizationId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_id: Option<FunctionId>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Workflow {
    pub fn new(organization_id: OrganizationId, name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: WorkflowId::new(),
            organization_id,
            function_id: None,
            name: name.into(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A snapshot of a workflow at a point in time, for version history.
///
/// Maps to `public.workflow_versions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersion {
    pub id: VersionId,
    pub workflow_id: WorkflowId,
    pub version: u32,
    pub change_reason: String,
    /// JSON snapshot of the full workflow graph (nodes + edges + policies).
    pub snapshot: serde_json::Value,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_new_defaults() {
        let org_id = OrganizationId::new();
        let wf = Workflow::new(org_id, "Procurement");

        assert_eq!(wf.organization_id, org_id);
        assert_eq!(wf.name, "Procurement");
        assert!(wf.function_id.is_none());
        assert!(wf.description.is_none());
        assert_eq!(wf.created_at, wf.updated_at);
    }

    #[test]
    fn workflow_unique_ids() {
        let org_id = OrganizationId::new();
        let wf1 = Workflow::new(org_id, "WF1");
        let wf2 = Workflow::new(org_id, "WF2");
        assert_ne!(wf1.id, wf2.id);
    }

    #[test]
    fn function_new_defaults() {
        let org_id = OrganizationId::new();
        let func = Function::new(org_id, "HR Onboarding");

        assert_eq!(func.organization_id, org_id);
        assert_eq!(func.name, "HR Onboarding");
        assert!(func.description.is_none());
        assert_eq!(func.created_at, func.updated_at);
    }

    #[test]
    fn workflow_with_function_id() {
        let org_id = OrganizationId::new();
        let func = Function::new(org_id, "Procurement");
        let mut wf = Workflow::new(org_id, "Vendor Selection");
        wf.function_id = Some(func.id);

        let json = serde_json::to_string(&wf).unwrap();
        let deserialized: Workflow = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.function_id, Some(func.id));
    }

    #[test]
    fn workflow_function_id_omitted_when_none() {
        let org_id = OrganizationId::new();
        let wf = Workflow::new(org_id, "No Function");
        let json = serde_json::to_string(&wf).unwrap();
        assert!(!json.contains("function_id"));
    }

    #[test]
    fn workflow_json_roundtrip() {
        let org_id = OrganizationId::new();
        let mut wf = Workflow::new(org_id, "Full Roundtrip");
        wf.description = Some("Testing roundtrip".to_string());
        wf.function_id = Some(FunctionId::new());

        let json = serde_json::to_string(&wf).unwrap();
        let deserialized: Workflow = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, wf.id);
        assert_eq!(deserialized.name, wf.name);
        assert_eq!(deserialized.description, wf.description);
        assert_eq!(deserialized.function_id, wf.function_id);
    }

    #[test]
    fn workflow_version_snapshot_holds_arbitrary_json() {
        let wf_id = WorkflowId::new();
        let snapshot = serde_json::json!({
            "nodes": [{"id": "n1", "label": "Start"}],
            "edges": [{"from": "n1", "to": "n2"}],
            "metadata": {"version": 1, "nested": {"deep": true}}
        });

        let version = WorkflowVersion {
            id: VersionId::new(),
            workflow_id: wf_id,
            version: 1,
            change_reason: "Initial version".to_string(),
            snapshot: snapshot.clone(),
            created_by: UserId::new(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&version).unwrap();
        let deserialized: WorkflowVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.snapshot, snapshot);
        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.change_reason, "Initial version");
    }

    #[test]
    fn workflow_version_snapshot_can_be_empty_object() {
        let version = WorkflowVersion {
            id: VersionId::new(),
            workflow_id: WorkflowId::new(),
            version: 0,
            change_reason: "Empty".to_string(),
            snapshot: serde_json::json!({}),
            created_by: UserId::new(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&version).unwrap();
        let deserialized: WorkflowVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.snapshot, serde_json::json!({}));
    }
}
