use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{EdgeId, NodeId, RequirementId, RoleId, WorkflowId};

/// An edge connecting two decision nodes in the workflow graph.
///
/// Edges describe *what* needs to happen between decisions (intent-level),
/// not *how* (implementation). The `rule` field is the edge specification
/// that gets compiled to automations by the integration compiler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub workflow_id: WorkflowId,
    pub from_node_id: NodeId,
    pub to_node_id: NodeId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The edge specification — natural language description of what
    /// must happen for this transition. Compiled to automation by the
    /// integration compiler.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Edge {
    pub fn new(workflow_id: WorkflowId, from_node_id: NodeId, to_node_id: NodeId) -> Self {
        let now = Utc::now();
        Self {
            id: EdgeId::new(),
            workflow_id,
            from_node_id,
            to_node_id,
            label: None,
            rule: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// The type of requirement attached to an edge transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementType {
    Document,
    Approval,
}

/// A requirement that must be satisfied for an edge transition.
///
/// Examples: "Upload signed contract", "Budget approval from CFO".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRequirement {
    pub id: RequirementId,
    pub edge_id: EdgeId,
    #[serde(rename = "type")]
    pub requirement_type: RequirementType,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub is_optional: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Which organizational roles are authorized to execute a transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRolePermission {
    pub edge_id: EdgeId,
    pub role_id: RoleId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_new_constructor() {
        let wf_id = WorkflowId::new();
        let from = NodeId::new();
        let to = NodeId::new();
        let edge = Edge::new(wf_id, from, to);

        assert_eq!(edge.workflow_id, wf_id);
        assert_eq!(edge.from_node_id, from);
        assert_eq!(edge.to_node_id, to);
        assert!(edge.label.is_none());
        assert!(edge.rule.is_none());
        assert_eq!(edge.created_at, edge.updated_at);
    }

    #[test]
    fn edge_unique_ids() {
        let wf_id = WorkflowId::new();
        let from = NodeId::new();
        let to = NodeId::new();
        let e1 = Edge::new(wf_id, from, to);
        let e2 = Edge::new(wf_id, from, to);
        assert_ne!(e1.id, e2.id);
    }

    #[test]
    fn edge_json_roundtrip() {
        let wf_id = WorkflowId::new();
        let from = NodeId::new();
        let to = NodeId::new();
        let mut edge = Edge::new(wf_id, from, to);
        edge.label = Some("Approved".to_string());
        edge.rule = Some("Budget approved by manager".to_string());

        let json = serde_json::to_string(&edge).unwrap();
        let deserialized: Edge = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, edge.id);
        assert_eq!(deserialized.workflow_id, wf_id);
        assert_eq!(deserialized.from_node_id, from);
        assert_eq!(deserialized.to_node_id, to);
        assert_eq!(deserialized.label, Some("Approved".to_string()));
        assert_eq!(deserialized.rule, Some("Budget approved by manager".to_string()));
    }

    #[test]
    fn edge_optional_fields_omitted_when_none() {
        let edge = Edge::new(WorkflowId::new(), NodeId::new(), NodeId::new());
        let json = serde_json::to_string(&edge).unwrap();
        assert!(!json.contains("\"label\""));
        assert!(!json.contains("\"rule\""));
    }

    #[test]
    fn requirement_type_document_serializes() {
        let json = serde_json::to_string(&RequirementType::Document).unwrap();
        assert_eq!(json, "\"document\"");
        let rt: RequirementType = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, RequirementType::Document);
    }

    #[test]
    fn requirement_type_approval_serializes() {
        let json = serde_json::to_string(&RequirementType::Approval).unwrap();
        assert_eq!(json, "\"approval\"");
        let rt: RequirementType = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, RequirementType::Approval);
    }

    #[test]
    fn edge_requirement_with_config() {
        let req = EdgeRequirement {
            id: RequirementId::new(),
            edge_id: EdgeId::new(),
            requirement_type: RequirementType::Document,
            label: "Upload contract".to_string(),
            description: Some("Signed contract PDF".to_string()),
            config: Some(serde_json::json!({"max_size_mb": 10, "allowed_types": ["pdf"]})),
            is_optional: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: EdgeRequirement = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.label, "Upload contract");
        assert_eq!(deserialized.description, Some("Signed contract PDF".to_string()));
        assert!(deserialized.config.is_some());
        assert_eq!(deserialized.config.unwrap()["max_size_mb"], 10);
    }

    #[test]
    fn edge_requirement_without_optional_config() {
        let req = EdgeRequirement {
            id: RequirementId::new(),
            edge_id: EdgeId::new(),
            requirement_type: RequirementType::Approval,
            label: "Manager approval".to_string(),
            description: None,
            config: None,
            is_optional: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("\"description\""));
        assert!(!json.contains("\"config\""));
    }

    #[test]
    fn is_optional_defaults_to_false() {
        let json = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "edge_id": "550e8400-e29b-41d4-a716-446655440001",
            "type": "document",
            "label": "Test requirement",
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T00:00:00Z"
        });
        let req: EdgeRequirement = serde_json::from_value(json).unwrap();
        assert!(!req.is_optional);
    }

    #[test]
    fn edge_requirement_type_field_renamed_to_type() {
        let req = EdgeRequirement {
            id: RequirementId::new(),
            edge_id: EdgeId::new(),
            requirement_type: RequirementType::Document,
            label: "Test".to_string(),
            description: None,
            config: None,
            is_optional: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&req).unwrap();
        // The field should be serialized as "type", not "requirement_type"
        assert!(json.contains("\"type\""));
        assert!(!json.contains("\"requirement_type\""));
    }
}
