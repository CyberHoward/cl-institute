use std::path::Path;

use inst_model::{
    DecisionNode, EdgeRequirement, EdgeRolePermission, Integration, Organization,
    OrganizationalRole, Policy,
};

use crate::error::{Result, StoreError};
use crate::project::{EdgeFile, InstitutionProject, PolicyFrontmatter, WorkflowDefinition};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read a file to string, wrapping I/O errors with the file path.
fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| StoreError::io(path, e))
}

/// Deserialize a TOML file into `T`.
fn read_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content = read_file(path)?;
    toml::from_str(&content).map_err(|e| StoreError::toml_parse(path, e))
}

/// Collect all `.toml` files in a directory, sorted by filename for determinism.
fn toml_files_in(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| StoreError::io(dir, e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    Ok(files)
}

/// Collect all `.md` files in a directory, sorted by filename for determinism.
fn md_files_in(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| StoreError::io(dir, e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    Ok(files)
}

/// Collect all subdirectories of a directory, sorted by name.
fn subdirs_of(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut dirs: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| StoreError::io(dir, e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    dirs.sort();
    Ok(dirs)
}

// ---------------------------------------------------------------------------
// Public load functions
// ---------------------------------------------------------------------------

/// Load the organization metadata from `institution.toml` at the project root.
pub fn load_organization(root: &Path) -> Result<Organization> {
    let path = root.join("institution.toml");
    if !path.exists() {
        return Err(StoreError::FileNotFound(path));
    }
    read_toml(&path)
}

/// Load all organizational roles from `roles/*.toml`.
pub fn load_roles(root: &Path) -> Result<Vec<OrganizationalRole>> {
    let roles_dir = root.join("roles");
    let files = toml_files_in(&roles_dir)?;
    let mut roles = Vec::with_capacity(files.len());
    for file in &files {
        let role: OrganizationalRole = read_toml(file)?;
        roles.push(role);
    }
    Ok(roles)
}

/// Load a single workflow directory.
///
/// The workflow directory is expected to contain:
/// - `workflow.toml` — the workflow definition
/// - `decisions/*.toml` — decision node files
/// - `edges/*.toml` — edge files (may contain inline requirements and role_permissions)
/// - `policies/*.md` — policy markdown files with TOML frontmatter
pub fn load_workflow(workflow_dir: &Path) -> Result<WorkflowDefinition> {
    if !workflow_dir.is_dir() {
        return Err(StoreError::DirectoryNotFound(workflow_dir.to_path_buf()));
    }

    // Load workflow.toml
    let workflow_path = workflow_dir.join("workflow.toml");
    if !workflow_path.exists() {
        return Err(StoreError::FileNotFound(workflow_path));
    }
    let workflow = read_toml(&workflow_path)?;

    // Load decisions
    let decisions_dir = workflow_dir.join("decisions");
    let decision_files = toml_files_in(&decisions_dir)?;
    let mut decisions = Vec::with_capacity(decision_files.len());
    for file in &decision_files {
        let node: DecisionNode = read_toml(file)?;
        decisions.push(node);
    }

    // Load edges (with inline requirements and role_permissions)
    let edges_dir = workflow_dir.join("edges");
    let edge_files = toml_files_in(&edges_dir)?;
    let mut edges = Vec::with_capacity(edge_files.len());
    let mut edge_requirements: Vec<EdgeRequirement> = Vec::new();
    let mut edge_role_permissions: Vec<EdgeRolePermission> = Vec::new();

    for file in &edge_files {
        let edge_file: EdgeFile = read_toml(file)?;
        edges.push(edge_file.edge);
        edge_requirements.extend(edge_file.requirements);
        edge_role_permissions.extend(edge_file.role_permissions);
    }

    // Load policies from markdown files
    let policies_dir = workflow_dir.join("policies");
    let policy_files = md_files_in(&policies_dir)?;
    let mut policies = Vec::with_capacity(policy_files.len());
    for file in &policy_files {
        let policy = load_policy_file(file)?;
        policies.push(policy);
    }

    Ok(WorkflowDefinition {
        workflow,
        decisions,
        edges,
        edge_requirements,
        edge_role_permissions,
        policies,
    })
}

/// Load all integrations from `integrations/*.toml`.
///
/// The `registry.toml` file, if present, is skipped — it is the registry
/// metadata and not an individual integration definition.
pub fn load_integrations(root: &Path) -> Result<Vec<Integration>> {
    let integrations_dir = root.join("integrations");
    let files = toml_files_in(&integrations_dir)?;
    let mut integrations = Vec::with_capacity(files.len());
    for file in &files {
        // Skip registry.toml — it is a registry index, not an integration.
        if file.file_name().and_then(|n| n.to_str()) == Some("registry.toml") {
            continue;
        }
        let integration: Integration = read_toml(file)?;
        integrations.push(integration);
    }
    Ok(integrations)
}

/// Load the complete institution from a project directory.
///
/// This reads:
/// - `institution.toml` — organization metadata
/// - `roles/*.toml` — organizational roles
/// - `workflows/*/` — all workflow subdirectories
/// - `integrations/*.toml` — integration definitions
pub fn load_institution(root: &Path) -> Result<InstitutionProject> {
    if !root.is_dir() {
        return Err(StoreError::DirectoryNotFound(root.to_path_buf()));
    }

    let organization = load_organization(root)?;
    let roles = load_roles(root)?;

    // Load all workflow directories
    let workflows_dir = root.join("workflows");
    let workflow_dirs = subdirs_of(&workflows_dir)?;
    let mut workflows = Vec::with_capacity(workflow_dirs.len());
    for dir in &workflow_dirs {
        let wf = load_workflow(dir)?;
        workflows.push(wf);
    }

    let integrations = load_integrations(root)?;

    Ok(InstitutionProject {
        root: root.to_path_buf(),
        organization,
        roles,
        workflows,
        integrations,
    })
}

// ---------------------------------------------------------------------------
// Policy file parsing
// ---------------------------------------------------------------------------

/// Parse a policy markdown file.
///
/// Policy files support two formats:
///
/// **Format 1: TOML frontmatter** (delimited by `+++` lines)
/// ```text
/// +++
/// id = "..."
/// organization_id = "..."
/// scope = "procurement.vendor-selection"
/// strength = "preference"
/// created_at = ...
/// updated_at = ...
/// +++
///
/// The actual policy text in markdown...
/// ```
///
/// **Format 2: Plain markdown** (no frontmatter)
/// The entire file content becomes the policy text. A default Policy is
/// created with the filename (minus extension) used to derive the scope.
/// This format is not round-trip safe and is mainly for reading hand-written
/// policy files that have not yet been saved through the store.
fn load_policy_file(path: &Path) -> Result<Policy> {
    let content = read_file(path)?;

    if content.starts_with("+++\n") || content.starts_with("+++\r\n") {
        // Parse TOML frontmatter
        let rest = if content.starts_with("+++\n") {
            &content[4..]
        } else {
            &content[5..]
        };

        let end = rest.find("\n+++\n")
            .or_else(|| rest.find("\r\n+++\r\n"))
            .or_else(|| rest.find("\n+++").filter(|&i| i + 4 >= rest.len()));

        match end {
            Some(end_pos) => {
                let frontmatter_str = &rest[..end_pos];
                let body_start = end_pos + rest[end_pos..].find('\n').unwrap_or(end_pos) + 1;
                // Skip the closing +++ line
                let after_closing = &rest[body_start..];
                let body = after_closing
                    .find('\n')
                    .map(|i| &after_closing[i + 1..])
                    .unwrap_or("");
                let text = body.trim().to_string();

                let fm: PolicyFrontmatter =
                    toml::from_str(frontmatter_str).map_err(|e| StoreError::toml_parse(path, e))?;

                Ok(Policy {
                    id: fm.id,
                    organization_id: fm.organization_id,
                    scope: fm.scope,
                    strength: fm.strength,
                    text,
                    metadata: fm.metadata,
                    created_at: fm.created_at,
                    updated_at: fm.updated_at,
                })
            }
            None => {
                // Malformed frontmatter — treat entire file as text
                load_policy_plain(path, &content)
            }
        }
    } else {
        load_policy_plain(path, &content)
    }
}

/// Create a Policy from a plain markdown file with no frontmatter.
///
/// Uses the filename (without extension) as the scope, defaulting to
/// `"unknown"` strength. This is a fallback for hand-authored files.
fn load_policy_plain(path: &Path, content: &str) -> Result<Policy> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let now = chrono::Utc::now();
    Ok(Policy {
        id: inst_model::PolicyId::new(),
        organization_id: inst_model::OrganizationId::new(),
        scope: name.to_string(),
        strength: inst_model::PolicyStrength::Context,
        text: content.trim().to_string(),
        metadata: None,
        created_at: now,
        updated_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    // This is a compilation-only test — the real test would need
    // a full fixture directory on disk.
    #[test]
    fn test_load_organization_not_found() {
        let dir = setup_temp_dir();
        let result = load_organization(dir.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            StoreError::FileNotFound(p) => {
                assert!(p.ends_with("institution.toml"));
            }
            other => panic!("Expected FileNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn test_load_roles_empty() {
        let dir = setup_temp_dir();
        // No roles/ directory at all — should return empty vec
        let roles = load_roles(dir.path()).unwrap();
        assert!(roles.is_empty());
    }

    #[test]
    fn test_load_integrations_empty() {
        let dir = setup_temp_dir();
        let integrations = load_integrations(dir.path()).unwrap();
        assert!(integrations.is_empty());
    }

    #[test]
    fn test_load_policy_plain_markdown() {
        let dir = setup_temp_dir();
        let policy_path = dir.path().join("vendor-preference.md");
        fs::write(&policy_path, "# Vendor Preference\n\nPrefer local vendors.").unwrap();
        let policy = load_policy_file(&policy_path).unwrap();
        assert_eq!(policy.scope, "vendor-preference");
        assert_eq!(policy.text, "# Vendor Preference\n\nPrefer local vendors.");
    }

    #[test]
    fn test_load_acme_foundation_example() {
        // Path to the acme-foundation example relative to the workspace root.
        // The workspace root is rust/, and the example lives at ../../examples/acme-foundation
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let example_dir = manifest_dir
            .parent() // crates/
            .unwrap()
            .parent() // rust/
            .unwrap()
            .parent() // clinstitute/
            .unwrap()
            .join("examples")
            .join("acme-foundation");

        if !example_dir.exists() {
            // Skip if the example directory is not present (e.g., in CI without examples).
            eprintln!("Skipping test: example directory not found at {:?}", example_dir);
            return;
        }

        let project = load_institution(&example_dir)
            .unwrap_or_else(|e| panic!("Failed to load acme-foundation example: {}", e));

        // Verify organization
        assert_eq!(project.organization.name, "Acme Foundation");
        assert!(project.organization.description.is_some());
        assert!(!project.organization.rules.is_empty());

        // Verify roles
        assert_eq!(project.roles.len(), 4);
        let role_names: Vec<&str> = project.roles.iter().map(|r| r.name.as_str()).collect();
        assert!(role_names.contains(&"Board Member"));
        assert!(role_names.contains(&"Compliance Officer"));
        assert!(role_names.contains(&"Finance Director"));
        assert!(role_names.contains(&"Procurement Lead"));

        // All roles must reference the same organization_id
        for role in &project.roles {
            assert_eq!(role.organization_id, project.organization.id);
        }

        // Verify workflows
        assert_eq!(project.workflows.len(), 1);
        let procurement = &project.workflows[0];
        assert_eq!(procurement.workflow.name, "Procurement");
        assert_eq!(procurement.workflow.organization_id, project.organization.id);

        // Verify decision nodes
        assert_eq!(procurement.decisions.len(), 4);
        let start_nodes: Vec<_> = procurement
            .decisions
            .iter()
            .filter(|d| d.node_type == inst_model::NodeType::Start)
            .collect();
        assert_eq!(start_nodes.len(), 1);
        let end_nodes: Vec<_> = procurement
            .decisions
            .iter()
            .filter(|d| d.node_type == inst_model::NodeType::End)
            .collect();
        assert_eq!(end_nodes.len(), 1);

        // All decision nodes must reference the workflow
        for node in &procurement.decisions {
            assert_eq!(node.workflow_id, procurement.workflow.id);
        }

        // Verify edges
        assert_eq!(procurement.edges.len(), 3);
        for edge in &procurement.edges {
            assert_eq!(edge.workflow_id, procurement.workflow.id);
            assert!(edge.label.is_some());
            assert!(edge.rule.is_some());
        }

        // Verify policies
        assert_eq!(procurement.policies.len(), 3);
        for policy in &procurement.policies {
            assert_eq!(policy.organization_id, project.organization.id);
            assert!(!policy.text.is_empty());
        }

        // Verify integrations
        assert_eq!(project.integrations.len(), 2);
        let integration_names: Vec<&str> =
            project.integrations.iter().map(|i| i.name.as_str()).collect();
        assert!(integration_names.contains(&"DocuSign"));
        assert!(integration_names.contains(&"SAP"));

        for integration in &project.integrations {
            assert_eq!(integration.organization_id, project.organization.id);
            assert!(!integration.capabilities.is_empty());
        }

        // Verify specific capability counts
        let docusign = project.integrations.iter().find(|i| i.name == "DocuSign").unwrap();
        assert_eq!(docusign.capabilities.len(), 2);
        let sap = project.integrations.iter().find(|i| i.name == "SAP").unwrap();
        assert_eq!(sap.capabilities.len(), 3);
    }
}
