use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{NodeId, WorkflowId};

/// The type of a node in the decision graph.
///
/// Maps to `public.node_type` enum in DB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Start,
    Intermediate,
    End,
}

/// The category of judgment required at a decision point.
///
/// From the architecture doc's Decision Point Ontology (Section 2.2).
/// This is provisional — the real ontology emerges from modeling actual institutions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    Approval,
    Classification,
    Prioritization,
    Allocation,
    ExceptionHandling,
}

/// A node in the workflow decision graph — the atomic unit of the system.
///
/// Maps to `public.nodes` in the DB schema.
/// In the architecture, this is the "judgment point" where institutional
/// decisions require context, policy, precedent, and discretion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNode {
    pub id: NodeId,
    pub workflow_id: WorkflowId,
    pub node_type: NodeType,
    pub label: String,
    /// Ordering index within the workflow.
    #[serde(default)]
    pub index: i32,
    /// Visual position X coordinate.
    #[serde(default)]
    pub x: f64,
    /// Visual position Y coordinate.
    #[serde(default)]
    pub y: f64,
    /// The category of decision at this node (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_type: Option<DecisionType>,
    /// Minimum authority level required to make this decision.
    #[serde(default)]
    pub requires_authority: u32,
    /// The schema for what this decision produces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DecisionNode {
    pub fn new(workflow_id: WorkflowId, node_type: NodeType, label: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: NodeId::new(),
            workflow_id,
            node_type,
            label: label.into(),
            index: 0,
            x: 0.0,
            y: 0.0,
            decision_type: None,
            requires_authority: 0,
            output_schema: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_start_node() {
        let wf_id = WorkflowId::new();
        let node = DecisionNode::new(wf_id, NodeType::Start, "Begin Process");

        assert_eq!(node.workflow_id, wf_id);
        assert_eq!(node.node_type, NodeType::Start);
        assert_eq!(node.label, "Begin Process");
        assert_eq!(node.index, 0);
        assert_eq!(node.x, 0.0);
        assert_eq!(node.y, 0.0);
        assert!(node.decision_type.is_none());
        assert_eq!(node.requires_authority, 0);
        assert!(node.output_schema.is_none());
    }

    #[test]
    fn new_intermediate_node() {
        let wf_id = WorkflowId::new();
        let node = DecisionNode::new(wf_id, NodeType::Intermediate, "Review Step");
        assert_eq!(node.node_type, NodeType::Intermediate);
    }

    #[test]
    fn new_end_node() {
        let wf_id = WorkflowId::new();
        let node = DecisionNode::new(wf_id, NodeType::End, "Complete");
        assert_eq!(node.node_type, NodeType::End);
    }

    #[test]
    fn node_type_serializes_as_snake_case() {
        assert_eq!(serde_json::to_string(&NodeType::Start).unwrap(), "\"start\"");
        assert_eq!(serde_json::to_string(&NodeType::Intermediate).unwrap(), "\"intermediate\"");
        assert_eq!(serde_json::to_string(&NodeType::End).unwrap(), "\"end\"");
    }

    #[test]
    fn node_type_roundtrip_all_variants() {
        for nt in [NodeType::Start, NodeType::Intermediate, NodeType::End] {
            let json = serde_json::to_string(&nt).unwrap();
            let deserialized: NodeType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, nt);
        }
    }

    #[test]
    fn decision_type_serializes_as_snake_case() {
        assert_eq!(serde_json::to_string(&DecisionType::Approval).unwrap(), "\"approval\"");
        assert_eq!(serde_json::to_string(&DecisionType::Classification).unwrap(), "\"classification\"");
        assert_eq!(serde_json::to_string(&DecisionType::Prioritization).unwrap(), "\"prioritization\"");
        assert_eq!(serde_json::to_string(&DecisionType::Allocation).unwrap(), "\"allocation\"");
        assert_eq!(serde_json::to_string(&DecisionType::ExceptionHandling).unwrap(), "\"exception_handling\"");
    }

    #[test]
    fn decision_type_roundtrip_all_variants() {
        for dt in [
            DecisionType::Approval,
            DecisionType::Classification,
            DecisionType::Prioritization,
            DecisionType::Allocation,
            DecisionType::ExceptionHandling,
        ] {
            let json = serde_json::to_string(&dt).unwrap();
            let deserialized: DecisionType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, dt);
        }
    }

    #[test]
    fn position_fields_default_to_zero() {
        let json = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "workflow_id": "550e8400-e29b-41d4-a716-446655440001",
            "node_type": "start",
            "label": "Start Node",
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T00:00:00Z"
        });
        let node: DecisionNode = serde_json::from_value(json).unwrap();
        assert_eq!(node.x, 0.0);
        assert_eq!(node.y, 0.0);
        assert_eq!(node.index, 0);
        assert_eq!(node.requires_authority, 0);
    }

    #[test]
    fn decision_node_json_roundtrip() {
        let wf_id = WorkflowId::new();
        let mut node = DecisionNode::new(wf_id, NodeType::Intermediate, "Budget Approval");
        node.decision_type = Some(DecisionType::Approval);
        node.requires_authority = 3;
        node.x = 150.5;
        node.y = 200.0;
        node.index = 2;
        node.output_schema = Some("approval_result".to_string());

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: DecisionNode = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, node.id);
        assert_eq!(deserialized.node_type, NodeType::Intermediate);
        assert_eq!(deserialized.decision_type, Some(DecisionType::Approval));
        assert_eq!(deserialized.requires_authority, 3);
        assert_eq!(deserialized.x, 150.5);
        assert_eq!(deserialized.y, 200.0);
        assert_eq!(deserialized.index, 2);
        assert_eq!(deserialized.output_schema, Some("approval_result".to_string()));
    }

    #[test]
    fn decision_type_omitted_when_none() {
        let wf_id = WorkflowId::new();
        let node = DecisionNode::new(wf_id, NodeType::Start, "Start");
        let json = serde_json::to_string(&node).unwrap();
        assert!(!json.contains("decision_type"));
    }

    #[test]
    fn output_schema_omitted_when_none() {
        let wf_id = WorkflowId::new();
        let node = DecisionNode::new(wf_id, NodeType::End, "End");
        let json = serde_json::to_string(&node).unwrap();
        assert!(!json.contains("output_schema"));
    }
}
