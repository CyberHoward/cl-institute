use serde::{Deserialize, Serialize};

use inst_model::{
    DecisionNode, Edge, EdgeRequirement, EdgeRolePermission, Integration, Organization,
    OrganizationalRole, Policy, Workflow,
};

/// Severity of a constraint violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Hard failure — the model is invalid and must be fixed.
    Error,
    /// Advisory — something looks suspicious but may be intentional.
    Warning,
}

/// Where in the institutional model a violation was detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ViolationLocation {
    Organization {
        organization_id: String,
    },
    Workflow {
        workflow_id: String,
    },
    Node {
        workflow_id: String,
        node_id: String,
    },
    Edge {
        workflow_id: String,
        edge_id: String,
    },
    Role {
        role_id: String,
    },
    Policy {
        policy_id: String,
    },
    Global,
}

/// A single constraint violation with full diagnostic context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    /// Name of the constraint that produced this violation.
    pub constraint_name: String,
    /// How severe the violation is.
    pub severity: Severity,
    /// Human-readable description of what went wrong.
    pub message: String,
    /// Which entity in the model is responsible.
    pub location: ViolationLocation,
    /// Optional suggestion for how to fix the issue.
    pub suggestion: Option<String>,
}

/// All the data needed to validate institutional invariants.
///
/// This bundles the full organizational model so that constraints
/// can cross-reference between workflows, roles, policies, etc.
pub struct ValidationContext {
    pub organization: Organization,
    pub roles: Vec<OrganizationalRole>,
    pub workflows: Vec<WorkflowData>,
    pub policies: Vec<Policy>,
    pub integrations: Vec<Integration>,
}

/// A workflow together with its constituent graph elements.
pub struct WorkflowData {
    pub workflow: Workflow,
    pub nodes: Vec<DecisionNode>,
    pub edges: Vec<Edge>,
    pub edge_requirements: Vec<EdgeRequirement>,
    pub edge_role_permissions: Vec<EdgeRolePermission>,
}

/// Aggregated result of running all constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Every violation found across all constraints.
    pub violations: Vec<ConstraintViolation>,
    /// `true` only if there are zero `Error`-severity violations.
    /// Warnings alone do not make the result invalid.
    pub is_valid: bool,
}

impl ValidationResult {
    /// Build a `ValidationResult` from a collected set of violations.
    pub fn from_violations(violations: Vec<ConstraintViolation>) -> Self {
        let is_valid = !violations
            .iter()
            .any(|v| matches!(v.severity, Severity::Error));
        Self {
            violations,
            is_valid,
        }
    }

    /// Returns only the error-severity violations.
    pub fn errors(&self) -> Vec<&ConstraintViolation> {
        self.violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Error))
            .collect()
    }

    /// Returns only the warning-severity violations.
    pub fn warnings(&self) -> Vec<&ConstraintViolation> {
        self.violations
            .iter()
            .filter(|v| matches!(v.severity, Severity::Warning))
            .collect()
    }
}
