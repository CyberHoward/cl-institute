use std::collections::{HashMap, HashSet};

use inst_model::NodeId;

use crate::traits::Constraint;
use crate::types::{
    ConstraintViolation, Severity, ValidationContext, ViolationLocation,
};

/// Detect cycles in the workflow graph.
///
/// This is a **warning**, not an error, because some workflows legitimately
/// contain cycles (e.g., review loops, retry paths).  The constraint flags
/// them so the designer is aware.
pub struct NoCyclicDependency;

impl Constraint for NoCyclicDependency {
    type Context = ValidationContext;

    fn name(&self) -> &str {
        "no_cyclic_dependency"
    }

    fn description(&self) -> &str {
        "Detect cycles in the workflow graph (warning — some workflows legitimately cycle)"
    }

    fn validate(&self, ctx: &Self::Context) -> Result<(), Vec<ConstraintViolation>> {
        let mut violations = Vec::new();

        for wf_data in &ctx.workflows {
            let wf_id = wf_data.workflow.id.to_string();
            let wf_name = &wf_data.workflow.name;

            // Build adjacency list.
            let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
            let all_node_ids: HashSet<_> = wf_data.nodes.iter().map(|n| n.id).collect();

            for node in &wf_data.nodes {
                adjacency.entry(node.id).or_default();
            }
            for edge in &wf_data.edges {
                adjacency.entry(edge.from_node_id).or_default().push(edge.to_node_id);
            }

            // DFS-based cycle detection using three-color marking.
            // White = unvisited, Gray = in current DFS path, Black = fully explored.
            let mut white: HashSet<NodeId> = all_node_ids.clone();
            let mut gray: HashSet<NodeId> = HashSet::new();
            let mut black: HashSet<NodeId> = HashSet::new();
            let mut has_cycle = false;

            // We need to track which nodes are part of cycles.
            // Simple approach: collect all nodes found on back-edges.
            let mut cycle_nodes: HashSet<NodeId> = HashSet::new();

            fn dfs(
                node: NodeId,
                adjacency: &HashMap<NodeId, Vec<NodeId>>,
                white: &mut HashSet<NodeId>,
                gray: &mut HashSet<NodeId>,
                black: &mut HashSet<NodeId>,
                has_cycle: &mut bool,
                cycle_nodes: &mut HashSet<NodeId>,
            ) {
                white.remove(&node);
                gray.insert(node);

                if let Some(neighbors) = adjacency.get(&node) {
                    for &neighbor in neighbors {
                        if gray.contains(&neighbor) {
                            // Back-edge found — cycle detected.
                            *has_cycle = true;
                            cycle_nodes.insert(node);
                            cycle_nodes.insert(neighbor);
                        } else if white.contains(&neighbor) {
                            dfs(
                                neighbor,
                                adjacency,
                                white,
                                gray,
                                black,
                                has_cycle,
                                cycle_nodes,
                            );
                        }
                    }
                }

                gray.remove(&node);
                black.insert(node);
            }

            // Run DFS from every unvisited node.
            let nodes_to_visit: Vec<_> = white.iter().copied().collect();
            for node_id in nodes_to_visit {
                if white.contains(&node_id) {
                    dfs(
                        node_id,
                        &adjacency,
                        &mut white,
                        &mut gray,
                        &mut black,
                        &mut has_cycle,
                        &mut cycle_nodes,
                    );
                }
            }

            if has_cycle {
                // Build a descriptive message listing the involved nodes.
                let involved_labels: Vec<String> = wf_data
                    .nodes
                    .iter()
                    .filter(|n| cycle_nodes.contains(&n.id))
                    .map(|n| format!("'{}' ({})", n.label, n.id))
                    .collect();

                violations.push(ConstraintViolation {
                    constraint_name: self.name().to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "Workflow '{}' contains a cycle involving nodes: {}",
                        wf_name,
                        involved_labels.join(", ")
                    ),
                    location: ViolationLocation::Workflow {
                        workflow_id: wf_id,
                    },
                    suggestion: Some(
                        "If this cycle is intentional (e.g., a review loop), this warning \
                         can be ignored. Otherwise, remove the back-edge to eliminate the cycle."
                            .to_string(),
                    ),
                });
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}
