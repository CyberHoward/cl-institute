use inst_model::DecisionType;

use crate::traits::Constraint;
use crate::types::{
    ConstraintViolation, Severity, ValidationContext, ViolationLocation,
};

/// Decision nodes with a `decision_type` set should have at least one policy
/// attached with a matching scope.
///
/// The scope matching is based on the `DecisionType` variant name, lowercased.
/// For example, a node with `decision_type = Some(Approval)` should have at
/// least one policy whose scope contains "approval".
///
/// This constraint checks that the organization has policies whose scope
/// is relevant to the type of decision being made.
pub struct RequiredPolicyScope;

impl RequiredPolicyScope {
    /// Map a `DecisionType` to the scope keyword it requires.
    fn scope_keyword(dt: DecisionType) -> &'static str {
        match dt {
            DecisionType::Approval => "approval",
            DecisionType::Classification => "classification",
            DecisionType::Prioritization => "prioritization",
            DecisionType::Allocation => "allocation",
            DecisionType::ExceptionHandling => "exception_handling",
        }
    }
}

impl Constraint for RequiredPolicyScope {
    type Context = ValidationContext;

    fn name(&self) -> &str {
        "required_policy_scope"
    }

    fn description(&self) -> &str {
        "Decision nodes with a decision_type should have at least one policy \
         with a matching scope in the organization"
    }

    fn validate(&self, ctx: &Self::Context) -> Result<(), Vec<ConstraintViolation>> {
        let mut violations = Vec::new();

        for wf_data in &ctx.workflows {
            let wf_id = wf_data.workflow.id.to_string();

            for node in &wf_data.nodes {
                let Some(decision_type) = node.decision_type else {
                    continue;
                };

                let keyword = Self::scope_keyword(decision_type);

                // Check if any policy in the organization has a scope that
                // contains the keyword (case-insensitive substring match
                // against the scope path segments).
                let has_matching_policy = ctx.policies.iter().any(|policy| {
                    policy
                        .organization_id
                        == ctx.organization.id
                        && policy.scope.to_lowercase().contains(keyword)
                });

                if !has_matching_policy {
                    violations.push(ConstraintViolation {
                        constraint_name: self.name().to_string(),
                        severity: Severity::Error,
                        message: format!(
                            "Node '{}' has decision_type '{:?}' but no policy with scope \
                             containing '{}' exists in the organization",
                            node.label, decision_type, keyword
                        ),
                        location: ViolationLocation::Node {
                            workflow_id: wf_id.clone(),
                            node_id: node.id.to_string(),
                        },
                        suggestion: Some(format!(
                            "Add a policy with a scope that includes '{}' (e.g., 'procurement.{}')",
                            keyword, keyword
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
