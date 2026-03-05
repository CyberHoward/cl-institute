use std::collections::HashMap;

use crate::traits::Constraint;
use crate::types::{
    ConstraintViolation, Severity, ValidationContext, ViolationLocation,
};

/// Role names must be unique within an organization, and workflow names
/// must be unique within a function (or within the org if no function).
pub struct UniqueNames;

impl Constraint for UniqueNames {
    fn name(&self) -> &str {
        "unique_names"
    }

    fn description(&self) -> &str {
        "Role names must be unique within the organization, and workflow names \
         must be unique within each function"
    }

    fn validate(&self, ctx: &ValidationContext) -> Result<(), Vec<ConstraintViolation>> {
        let mut violations = Vec::new();

        // --- Check role name uniqueness within the organization ---
        let mut role_names: HashMap<String, usize> = HashMap::new();
        for role in &ctx.roles {
            *role_names.entry(role.name.clone()).or_insert(0) += 1;
        }
        for (name, count) in &role_names {
            if *count > 1 {
                // Find all roles with this duplicate name and report the duplicates.
                for role in ctx.roles.iter().filter(|r| &r.name == name) {
                    violations.push(ConstraintViolation {
                        constraint_name: self.name().to_string(),
                        severity: Severity::Error,
                        message: format!(
                            "Role name '{}' is duplicated {} times in the organization",
                            name, count
                        ),
                        location: ViolationLocation::Role {
                            role_id: role.id.to_string(),
                        },
                        suggestion: Some(format!(
                            "Rename this role to be unique within the organization"
                        )),
                    });
                }
            }
        }

        // --- Check workflow name uniqueness within each function ---
        // Group workflows by their function_id (None = "top-level").
        let mut workflows_by_function: HashMap<Option<String>, Vec<(String, String)>> =
            HashMap::new();
        for wf_data in &ctx.workflows {
            let function_key = wf_data.workflow.function_id.map(|f| f.to_string());
            workflows_by_function
                .entry(function_key)
                .or_default()
                .push((wf_data.workflow.id.to_string(), wf_data.workflow.name.clone()));
        }

        for (function_key, workflows) in &workflows_by_function {
            let mut name_counts: HashMap<&str, usize> = HashMap::new();
            for (_, name) in workflows {
                *name_counts.entry(name.as_str()).or_insert(0) += 1;
            }

            for (name, count) in &name_counts {
                if *count > 1 {
                    let scope_desc = match function_key {
                        Some(fid) => format!("function {}", fid),
                        None => "the organization (no function)".to_string(),
                    };

                    for (wf_id, wf_name) in workflows.iter().filter(|(_, n)| n.as_str() == *name)
                    {
                        violations.push(ConstraintViolation {
                            constraint_name: self.name().to_string(),
                            severity: Severity::Error,
                            message: format!(
                                "Workflow name '{}' is duplicated {} times within {}",
                                wf_name, count, scope_desc
                            ),
                            location: ViolationLocation::Workflow {
                                workflow_id: wf_id.clone(),
                            },
                            suggestion: Some(
                                "Rename this workflow to be unique within its function scope"
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
