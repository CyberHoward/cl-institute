use std::path::Path;

use inst_model::{Integration, Organization, OrganizationalRole};

use crate::error::{Result, StoreError};
use crate::project::{EdgeFile, InstitutionProject, PolicyFrontmatter, WorkflowDefinition};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a string to a file, wrapping I/O errors with the file path.
fn write_file(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content).map_err(|e| StoreError::io(path, e))
}

/// Serialize a value to TOML and write it to a file.
fn write_toml<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let content = toml::to_string_pretty(value).map_err(|e| StoreError::toml_serialize(path, e))?;
    write_file(path, &content)
}

/// Ensure a directory exists, creating it (and parents) if necessary.
fn ensure_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        std::fs::create_dir_all(dir).map_err(|e| StoreError::io(dir, e))?;
    }
    Ok(())
}

/// Convert a name to a filesystem-safe slug (lowercase, hyphens for spaces/underscores).
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

// ---------------------------------------------------------------------------
// Public save functions
// ---------------------------------------------------------------------------

/// Save the organization metadata to `institution.toml` at the project root.
pub fn save_organization(root: &Path, org: &Organization) -> Result<()> {
    ensure_dir(root)?;
    let path = root.join("institution.toml");
    write_toml(&path, org)
}

/// Save all organizational roles to `roles/*.toml`.
///
/// Each role is written to a file named after its slugified name.
pub fn save_roles(root: &Path, roles: &[OrganizationalRole]) -> Result<()> {
    let roles_dir = root.join("roles");
    ensure_dir(&roles_dir)?;
    for role in roles {
        let filename = format!("{}.toml", slugify(&role.name));
        let path = roles_dir.join(filename);
        write_toml(&path, role)?;
    }
    Ok(())
}

/// Save a workflow definition to a workflow subdirectory.
///
/// The workflow is written to `workflows/<slug>/`, with:
/// - `workflow.toml` — the workflow definition
/// - `decisions/*.toml` — each decision node
/// - `edges/*.toml` — each edge (with inline requirements and role_permissions)
/// - `policies/*.md` — each policy as a markdown file with TOML frontmatter
pub fn save_workflow(root: &Path, wf: &WorkflowDefinition) -> Result<()> {
    let workflows_dir = root.join("workflows");
    let wf_dir = workflows_dir.join(slugify(&wf.workflow.name));
    ensure_dir(&wf_dir)?;

    // Write workflow.toml
    let workflow_path = wf_dir.join("workflow.toml");
    write_toml(&workflow_path, &wf.workflow)?;

    // Write decisions
    let decisions_dir = wf_dir.join("decisions");
    ensure_dir(&decisions_dir)?;
    for decision in &wf.decisions {
        let filename = format!("{}.toml", slugify(&decision.label));
        let path = decisions_dir.join(filename);
        write_toml(&path, decision)?;
    }

    // Write edges (with inline requirements and role_permissions)
    let edges_dir = wf_dir.join("edges");
    ensure_dir(&edges_dir)?;
    for edge in &wf.edges {
        let edge_id_str = edge.id.to_string();
        let edge_name = edge.label.as_deref().unwrap_or(&edge_id_str);
        let filename = format!("{}.toml", slugify(edge_name));
        let path = edges_dir.join(filename);

        // Gather requirements and role_permissions belonging to this edge
        let requirements: Vec<_> = wf
            .edge_requirements
            .iter()
            .filter(|r| r.edge_id == edge.id)
            .cloned()
            .collect();
        let role_permissions: Vec<_> = wf
            .edge_role_permissions
            .iter()
            .filter(|rp| rp.edge_id == edge.id)
            .cloned()
            .collect();

        let edge_file = EdgeFile {
            edge: edge.clone(),
            requirements,
            role_permissions,
        };
        write_toml(&path, &edge_file)?;
    }

    // Write policies as markdown with TOML frontmatter
    let policies_dir = wf_dir.join("policies");
    ensure_dir(&policies_dir)?;
    for policy in &wf.policies {
        save_policy_file(&policies_dir, policy)?;
    }

    Ok(())
}

/// Save all integrations to `integrations/*.toml`.
///
/// Each integration is written to a file named after its slugified name.
pub fn save_integrations(root: &Path, integrations: &[Integration]) -> Result<()> {
    let integrations_dir = root.join("integrations");
    ensure_dir(&integrations_dir)?;
    for integration in integrations {
        let filename = format!("{}.toml", slugify(&integration.name));
        let path = integrations_dir.join(filename);
        write_toml(&path, integration)?;
    }
    Ok(())
}

/// Save the complete institution project to a directory.
///
/// This writes:
/// - `institution.toml` — organization metadata
/// - `roles/*.toml` — organizational roles
/// - `workflows/*/` — all workflow subdirectories
/// - `integrations/*.toml` — integration definitions
pub fn save_institution(project: &InstitutionProject) -> Result<()> {
    let root = &project.root;
    ensure_dir(root)?;

    save_organization(root, &project.organization)?;
    save_roles(root, &project.roles)?;

    for wf in &project.workflows {
        save_workflow(root, wf)?;
    }

    save_integrations(root, &project.integrations)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Policy file writing
// ---------------------------------------------------------------------------

/// Write a policy as a markdown file with TOML frontmatter.
///
/// The file is named after the policy scope (slugified) and placed in the
/// given directory. The format is:
///
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
/// The policy text in markdown...
/// ```
fn save_policy_file(dir: &Path, policy: &inst_model::Policy) -> Result<()> {
    let filename = format!("{}.md", slugify(&policy.scope));
    let path = dir.join(&filename);

    let frontmatter = PolicyFrontmatter {
        id: policy.id,
        organization_id: policy.organization_id,
        scope: policy.scope.clone(),
        strength: policy.strength,
        metadata: policy.metadata.clone(),
        created_at: policy.created_at,
        updated_at: policy.updated_at,
    };

    let fm_toml =
        toml::to_string_pretty(&frontmatter).map_err(|e| StoreError::toml_serialize(&path, e))?;

    let content = format!("+++\n{fm_toml}+++\n\n{}\n", policy.text);
    write_file(&path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Compliance Officer"), "compliance-officer");
        assert_eq!(slugify("budget-approval"), "budget-approval");
        assert_eq!(slugify("SAP Integration"), "sap-integration");
        assert_eq!(slugify("Hello__World"), "hello-world");
    }
}
