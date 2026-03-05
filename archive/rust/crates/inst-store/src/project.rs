use std::path::PathBuf;

use inst_model::{
    DecisionNode, Edge, EdgeRequirement, EdgeRolePermission, Integration, Organization,
    OrganizationalRole, Policy, Workflow,
};
use serde::{Deserialize, Serialize};

/// Aggregate representing a fully loaded workflow directory.
///
/// A workflow directory contains:
/// - `workflow.toml` — the workflow definition
/// - `decisions/*.toml` — decision nodes in the graph
/// - `edges/*.toml` — edges connecting nodes
/// - `policies/*.md` — policy documents as markdown
///
/// Edge requirements and role permissions are stored inline within
/// edge TOML files under `[requirements]` and `[role_permissions]` arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub workflow: Workflow,
    pub decisions: Vec<DecisionNode>,
    pub edges: Vec<Edge>,
    pub edge_requirements: Vec<EdgeRequirement>,
    pub edge_role_permissions: Vec<EdgeRolePermission>,
    pub policies: Vec<Policy>,
}

/// Aggregate representing a fully loaded institution project directory.
///
/// This is the top-level struct returned by `load_institution`. It holds
/// everything needed to work with an institution-as-code project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionProject {
    /// The root directory of the institution project.
    #[serde(skip)]
    pub root: PathBuf,
    /// The organization metadata from `institution.toml`.
    pub organization: Organization,
    /// All organizational roles from `roles/*.toml`.
    pub roles: Vec<OrganizationalRole>,
    /// All workflow definitions from `workflows/*/`.
    pub workflows: Vec<WorkflowDefinition>,
    /// All integrations from `integrations/*.toml`.
    pub integrations: Vec<Integration>,
}

/// TOML-level representation of an edge file, which may contain
/// inline requirements and role permissions alongside the edge itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EdgeFile {
    #[serde(flatten)]
    pub edge: Edge,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<EdgeRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub role_permissions: Vec<EdgeRolePermission>,
}

/// TOML-level representation of a policy frontmatter header.
///
/// Policy `.md` files can optionally have a TOML frontmatter block
/// delimited by `+++` lines at the top. The body after the frontmatter
/// becomes the `text` field of the `Policy` struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PolicyFrontmatter {
    pub id: inst_model::PolicyId,
    pub organization_id: inst_model::OrganizationId,
    pub scope: String,
    pub strength: inst_model::PolicyStrength,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
