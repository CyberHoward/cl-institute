use std::collections::{HashSet, VecDeque};

use inst_model::NodeType;

use crate::traits::Constraint;
use crate::types::{
    ConstraintViolation, Severity, ValidationContext, ViolationLocation,
};

/// Validates that every workflow has exactly one start node, at least one
/// end node, and that all intermediate and end nodes are reachable from start.
pub struct GraphConnectivity;

impl Constraint for GraphConnectivity {
    type Context = ValidationContext;

    fn name(&self) -> &str {
        "graph_connectivity"
    }

    fn description(&self) -> &str {
        "Every workflow must have exactly one start node, at least one end node, \
         and all nodes must be reachable from the start node"
    }

    fn validate(&self, ctx: &Self::Context) -> Result<(), Vec<ConstraintViolation>> {
        let mut violations = Vec::new();

        for wf_data in &ctx.workflows {
            let wf_id = wf_data.workflow.id.to_string();
            let wf_name = &wf_data.workflow.name;

            // Count start and end nodes.
            let start_nodes: Vec<_> = wf_data
                .nodes
                .iter()
                .filter(|n| n.node_type == NodeType::Start)
                .collect();
            let end_nodes: Vec<_> = wf_data
                .nodes
                .iter()
                .filter(|n| n.node_type == NodeType::End)
                .collect();

            if start_nodes.is_empty() {
                violations.push(ConstraintViolation {
                    constraint_name: self.name().to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "Workflow '{}' has no start node",
                        wf_name
                    ),
                    location: ViolationLocation::Workflow {
                        workflow_id: wf_id.clone(),
                    },
                    suggestion: Some(
                        "Add a node with node_type = Start".to_string(),
                    ),
                });
            } else if start_nodes.len() > 1 {
                violations.push(ConstraintViolation {
                    constraint_name: self.name().to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "Workflow '{}' has {} start nodes (expected exactly 1)",
                        wf_name,
                        start_nodes.len()
                    ),
                    location: ViolationLocation::Workflow {
                        workflow_id: wf_id.clone(),
                    },
                    suggestion: Some(
                        "Remove extra start nodes so that only one remains".to_string(),
                    ),
                });
            }

            if end_nodes.is_empty() {
                violations.push(ConstraintViolation {
                    constraint_name: self.name().to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "Workflow '{}' has no end node",
                        wf_name
                    ),
                    location: ViolationLocation::Workflow {
                        workflow_id: wf_id.clone(),
                    },
                    suggestion: Some(
                        "Add at least one node with node_type = End".to_string(),
                    ),
                });
            }

            // Reachability: BFS from the (single) start node.
            if start_nodes.len() == 1 {
                let start_id = start_nodes[0].id;
                let all_node_ids: HashSet<_> =
                    wf_data.nodes.iter().map(|n| n.id).collect();

                // Build adjacency list from edges.
                let mut reachable = HashSet::new();
                let mut queue = VecDeque::new();
                reachable.insert(start_id);
                queue.push_back(start_id);

                while let Some(current) = queue.pop_front() {
                    for edge in &wf_data.edges {
                        if edge.from_node_id == current
                            && !reachable.contains(&edge.to_node_id)
                        {
                            reachable.insert(edge.to_node_id);
                            queue.push_back(edge.to_node_id);
                        }
                    }
                }

                for node in &wf_data.nodes {
                    if !reachable.contains(&node.id) && all_node_ids.contains(&node.id) {
                        violations.push(ConstraintViolation {
                            constraint_name: self.name().to_string(),
                            severity: Severity::Error,
                            message: format!(
                                "Node '{}' (id={}) in workflow '{}' is not reachable from the start node",
                                node.label, node.id, wf_name
                            ),
                            location: ViolationLocation::Node {
                                workflow_id: wf_id.clone(),
                                node_id: node.id.to_string(),
                            },
                            suggestion: Some(
                                "Add an edge from a reachable node to this node, or remove it"
                                    .to_string(),
                            ),
                        });
                    }
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
