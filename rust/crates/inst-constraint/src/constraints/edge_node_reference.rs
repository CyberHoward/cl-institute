use std::collections::HashSet;

use crate::traits::Constraint;
use crate::types::{
    ConstraintViolation, Severity, ValidationContext, ViolationLocation,
};

/// Every edge's `from_node_id` and `to_node_id` must reference nodes that
/// actually exist within the same workflow.
pub struct EdgeNodeReference;

impl Constraint for EdgeNodeReference {
    type Context = ValidationContext;

    fn name(&self) -> &str {
        "edge_node_reference"
    }

    fn description(&self) -> &str {
        "Every edge must reference existing nodes in the same workflow"
    }

    fn validate(&self, ctx: &Self::Context) -> Result<(), Vec<ConstraintViolation>> {
        let mut violations = Vec::new();

        for wf_data in &ctx.workflows {
            let wf_id = wf_data.workflow.id.to_string();
            let wf_name = &wf_data.workflow.name;

            let node_ids: HashSet<_> = wf_data.nodes.iter().map(|n| n.id).collect();

            for edge in &wf_data.edges {
                if !node_ids.contains(&edge.from_node_id) {
                    violations.push(ConstraintViolation {
                        constraint_name: self.name().to_string(),
                        severity: Severity::Error,
                        message: format!(
                            "Edge '{}' in workflow '{}' references non-existent from_node_id {}",
                            edge.id, wf_name, edge.from_node_id
                        ),
                        location: ViolationLocation::Edge {
                            workflow_id: wf_id.clone(),
                            edge_id: edge.id.to_string(),
                        },
                        suggestion: Some(format!(
                            "Update from_node_id to reference a node that exists in workflow '{}'",
                            wf_name
                        )),
                    });
                }

                if !node_ids.contains(&edge.to_node_id) {
                    violations.push(ConstraintViolation {
                        constraint_name: self.name().to_string(),
                        severity: Severity::Error,
                        message: format!(
                            "Edge '{}' in workflow '{}' references non-existent to_node_id {}",
                            edge.id, wf_name, edge.to_node_id
                        ),
                        location: ViolationLocation::Edge {
                            workflow_id: wf_id.clone(),
                            edge_id: edge.id.to_string(),
                        },
                        suggestion: Some(format!(
                            "Update to_node_id to reference a node that exists in workflow '{}'",
                            wf_name
                        )),
                    });
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}
