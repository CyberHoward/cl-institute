use std::collections::HashMap;

use inst_model::{NodeType, RoleId};

use crate::traits::Constraint;
use crate::types::{
    ConstraintViolation, Severity, ValidationContext, ViolationLocation,
};

/// For each decision node, at least one role with sufficient `authority_level`
/// must have edge permission to reach it.
///
/// Specifically, for every node that has `requires_authority > 0`, we check
/// that at least one incoming edge has an `EdgeRolePermission` granted to a
/// role whose `authority_level >= node.requires_authority`.
///
/// Start nodes are exempt (no incoming edges by definition).
pub struct AuthorityLevelCheck;

impl Constraint for AuthorityLevelCheck {
    type Context = ValidationContext;

    fn name(&self) -> &str {
        "authority_level_check"
    }

    fn description(&self) -> &str {
        "For each decision node that requires authority, at least one role with \
         sufficient authority_level must have permission on an incoming edge"
    }

    fn validate(&self, ctx: &Self::Context) -> Result<(), Vec<ConstraintViolation>> {
        let mut violations = Vec::new();

        // Build a lookup: RoleId -> authority_level.
        let role_authority: HashMap<RoleId, u32> = ctx
            .roles
            .iter()
            .map(|r| (r.id, r.authority_level))
            .collect();

        for wf_data in &ctx.workflows {
            let wf_id = wf_data.workflow.id.to_string();

            for node in &wf_data.nodes {
                // Only check nodes that require authority and are not start nodes.
                if node.requires_authority == 0 || node.node_type == NodeType::Start {
                    continue;
                }

                // Find all incoming edges to this node.
                let incoming_edges: Vec<_> = wf_data
                    .edges
                    .iter()
                    .filter(|e| e.to_node_id == node.id)
                    .collect();

                if incoming_edges.is_empty() {
                    // No incoming edges — this is caught by GraphConnectivity,
                    // but we note it as a problem here too if authority is required.
                    violations.push(ConstraintViolation {
                        constraint_name: self.name().to_string(),
                        severity: Severity::Error,
                        message: format!(
                            "Node '{}' requires authority_level {} but has no incoming edges with role permissions",
                            node.label, node.requires_authority
                        ),
                        location: ViolationLocation::Node {
                            workflow_id: wf_id.clone(),
                            node_id: node.id.to_string(),
                        },
                        suggestion: Some(
                            "Add an incoming edge with a role permission for a sufficiently authorized role".to_string(),
                        ),
                    });
                    continue;
                }

                // Collect all roles that have permission on any incoming edge.
                let incoming_edge_ids: Vec<_> =
                    incoming_edges.iter().map(|e| e.id).collect();

                let permitted_roles: Vec<_> = wf_data
                    .edge_role_permissions
                    .iter()
                    .filter(|perm| incoming_edge_ids.contains(&perm.edge_id))
                    .collect();

                // Check if any permitted role has sufficient authority.
                let has_sufficient_authority = permitted_roles.iter().any(|perm| {
                    role_authority
                        .get(&perm.role_id)
                        .map_or(false, |&level| level >= node.requires_authority)
                });

                if !has_sufficient_authority {
                    let max_authority = permitted_roles
                        .iter()
                        .filter_map(|perm| role_authority.get(&perm.role_id))
                        .max()
                        .copied()
                        .unwrap_or(0);

                    violations.push(ConstraintViolation {
                        constraint_name: self.name().to_string(),
                        severity: Severity::Error,
                        message: format!(
                            "Node '{}' requires authority_level {} but the highest authority \
                             among permitted roles on incoming edges is {}",
                            node.label, node.requires_authority, max_authority
                        ),
                        location: ViolationLocation::Node {
                            workflow_id: wf_id.clone(),
                            node_id: node.id.to_string(),
                        },
                        suggestion: Some(format!(
                            "Grant edge permission to a role with authority_level >= {}",
                            node.requires_authority
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
