//! Project context — resolves the project directory and provides helpers
//! for loading and saving institutional model files.
//!
//! The project directory layout follows the institution-as-code convention:
//!
//! ```text
//! <project>/
//!   institution.toml     — Organization metadata
//!   roles/               — OrganizationalRole definitions
//!     *.toml
//!   functions/            — Function definitions
//!     *.toml
//!   workflows/            — Workflow definitions (each a subdirectory)
//!     <name>/
//!       workflow.toml     — Workflow metadata
//!       decisions/        — DecisionNode definitions
//!         *.toml
//!       edges/            — Edge definitions
//!         *.toml
//!   policies/             — Policy documents
//!     *.toml
//!   integrations/         — Integration registry entries
//!     *.toml
//!   audit.jsonl           — Append-only audit log
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use inst_model::*;

use crate::error::CliError;

/// Resolved project context with the root directory path.
pub struct ProjectContext {
    pub root: PathBuf,
}

impl ProjectContext {
    /// Resolve the project directory from the CLI flag, environment variable,
    /// or the current working directory.
    pub fn resolve(project_flag: Option<&Path>) -> Result<Self, CliError> {
        let root = if let Some(p) = project_flag {
            p.to_path_buf()
        } else if let Ok(p) = std::env::var("INST_PROJECT") {
            PathBuf::from(p)
        } else {
            std::env::current_dir()?
        };

        Ok(Self { root })
    }

    /// Check that the project directory has been initialized.
    pub fn ensure_initialized(&self) -> Result<(), CliError> {
        let inst_file = self.root.join("institution.toml");
        if !inst_file.exists() {
            return Err(CliError::ProjectNotInitialized);
        }
        Ok(())
    }

    // ---------------------------------------------------------------
    // Path helpers
    // ---------------------------------------------------------------

    fn institution_file(&self) -> PathBuf {
        self.root.join("institution.toml")
    }

    fn roles_dir(&self) -> PathBuf {
        self.root.join("roles")
    }

    fn functions_dir(&self) -> PathBuf {
        self.root.join("functions")
    }

    fn workflows_dir(&self) -> PathBuf {
        self.root.join("workflows")
    }

    fn policies_dir(&self) -> PathBuf {
        self.root.join("policies")
    }

    fn integrations_dir(&self) -> PathBuf {
        self.root.join("integrations")
    }

    fn audit_file(&self) -> PathBuf {
        self.root.join("audit.jsonl")
    }

    // ---------------------------------------------------------------
    // Initialization
    // ---------------------------------------------------------------

    /// Initialize a new project directory.
    pub fn init_project(&self, name: &str) -> Result<Organization, CliError> {
        // Create directory structure.
        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(self.roles_dir())?;
        fs::create_dir_all(self.functions_dir())?;
        fs::create_dir_all(self.workflows_dir())?;
        fs::create_dir_all(self.policies_dir())?;
        fs::create_dir_all(self.integrations_dir())?;

        let org = Organization::new(name);
        self.save_organization(&org)?;

        // Create empty audit log.
        if !self.audit_file().exists() {
            fs::write(self.audit_file(), "")?;
        }

        Ok(org)
    }

    // ---------------------------------------------------------------
    // Organization
    // ---------------------------------------------------------------

    pub fn load_organization(&self) -> Result<Organization, CliError> {
        let content = fs::read_to_string(self.institution_file())
            .map_err(|_| CliError::ProjectNotInitialized)?;
        let org: Organization = toml_from_str(&content, &self.institution_file())?;
        Ok(org)
    }

    pub fn save_organization(&self, org: &Organization) -> Result<(), CliError> {
        let content = toml_to_string(org, &self.institution_file())?;
        fs::write(self.institution_file(), content)?;
        Ok(())
    }

    // ---------------------------------------------------------------
    // Roles
    // ---------------------------------------------------------------

    pub fn load_roles(&self) -> Result<Vec<OrganizationalRole>, CliError> {
        load_toml_dir(&self.roles_dir())
    }

    pub fn load_role_by_name(&self, name: &str) -> Result<OrganizationalRole, CliError> {
        let roles = self.load_roles()?;
        roles
            .into_iter()
            .find(|r| r.name == name)
            .ok_or_else(|| CliError::NotFound(format!("Role '{name}' not found")))
    }

    pub fn save_role(&self, role: &OrganizationalRole) -> Result<(), CliError> {
        fs::create_dir_all(self.roles_dir())?;
        let path = self.roles_dir().join(format!("{}.toml", slug(&role.name)));
        let content = toml_to_string(role, &path)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn delete_role_file(&self, name: &str) -> Result<(), CliError> {
        let path = self.roles_dir().join(format!("{}.toml", slug(name)));
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    // ---------------------------------------------------------------
    // Functions
    // ---------------------------------------------------------------

    pub fn load_functions(&self) -> Result<Vec<Function>, CliError> {
        load_toml_dir(&self.functions_dir())
    }

    pub fn load_function_by_name(&self, name: &str) -> Result<Function, CliError> {
        let functions = self.load_functions()?;
        functions
            .into_iter()
            .find(|f| f.name == name)
            .ok_or_else(|| CliError::NotFound(format!("Function '{name}' not found")))
    }

    pub fn save_function(&self, function: &Function) -> Result<(), CliError> {
        fs::create_dir_all(self.functions_dir())?;
        let path = self
            .functions_dir()
            .join(format!("{}.toml", slug(&function.name)));
        let content = toml_to_string(function, &path)?;
        fs::write(&path, content)?;
        Ok(())
    }

    // ---------------------------------------------------------------
    // Workflows
    // ---------------------------------------------------------------

    pub fn load_workflows(&self) -> Result<Vec<Workflow>, CliError> {
        let dir = self.workflows_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut workflows = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let wf_file = entry.path().join("workflow.toml");
                if wf_file.exists() {
                    let content = fs::read_to_string(&wf_file)?;
                    let wf: Workflow = toml_from_str(&content, &wf_file)?;
                    workflows.push(wf);
                }
            }
        }
        Ok(workflows)
    }

    pub fn load_workflow_by_name(&self, name: &str) -> Result<Workflow, CliError> {
        let wf_dir = self.workflows_dir().join(slug(name));
        let wf_file = wf_dir.join("workflow.toml");
        if !wf_file.exists() {
            return Err(CliError::NotFound(format!("Workflow '{name}' not found")));
        }
        let content = fs::read_to_string(&wf_file)?;
        let wf: Workflow = toml_from_str(&content, &wf_file)?;
        Ok(wf)
    }

    pub fn save_workflow(&self, workflow: &Workflow) -> Result<(), CliError> {
        let wf_dir = self.workflows_dir().join(slug(&workflow.name));
        fs::create_dir_all(&wf_dir)?;
        fs::create_dir_all(wf_dir.join("decisions"))?;
        fs::create_dir_all(wf_dir.join("edges"))?;
        let wf_file = wf_dir.join("workflow.toml");
        let content = toml_to_string(workflow, &wf_file)?;
        fs::write(&wf_file, content)?;
        Ok(())
    }

    // ---------------------------------------------------------------
    // Decision Nodes
    // ---------------------------------------------------------------

    pub fn load_nodes(&self, workflow_name: &str) -> Result<Vec<DecisionNode>, CliError> {
        let decisions_dir = self
            .workflows_dir()
            .join(slug(workflow_name))
            .join("decisions");
        load_toml_dir(&decisions_dir)
    }

    pub fn load_node_by_label(
        &self,
        workflow_name: &str,
        label: &str,
    ) -> Result<DecisionNode, CliError> {
        let nodes = self.load_nodes(workflow_name)?;
        nodes
            .into_iter()
            .find(|n| n.label == label)
            .ok_or_else(|| {
                CliError::NotFound(format!(
                    "Node '{label}' not found in workflow '{workflow_name}'"
                ))
            })
    }

    pub fn save_node(&self, workflow_name: &str, node: &DecisionNode) -> Result<(), CliError> {
        let decisions_dir = self
            .workflows_dir()
            .join(slug(workflow_name))
            .join("decisions");
        fs::create_dir_all(&decisions_dir)?;
        let path = decisions_dir.join(format!("{}.toml", slug(&node.label)));
        let content = toml_to_string(node, &path)?;
        fs::write(&path, content)?;
        Ok(())
    }

    // ---------------------------------------------------------------
    // Edges
    // ---------------------------------------------------------------

    pub fn load_edges(&self, workflow_name: &str) -> Result<Vec<Edge>, CliError> {
        let edges_dir = self
            .workflows_dir()
            .join(slug(workflow_name))
            .join("edges");
        load_toml_dir(&edges_dir)
    }

    pub fn save_edge(&self, workflow_name: &str, edge: &Edge) -> Result<(), CliError> {
        let edges_dir = self
            .workflows_dir()
            .join(slug(workflow_name))
            .join("edges");
        fs::create_dir_all(&edges_dir)?;
        let filename = format!("{}-to-{}.toml", edge.from_node_id.0, edge.to_node_id.0);
        let path = edges_dir.join(filename);
        let content = toml_to_string(edge, &path)?;
        fs::write(&path, content)?;
        Ok(())
    }

    // ---------------------------------------------------------------
    // Policies
    // ---------------------------------------------------------------

    pub fn load_policies(&self) -> Result<Vec<Policy>, CliError> {
        load_toml_dir(&self.policies_dir())
    }

    pub fn save_policy(&self, policy: &Policy) -> Result<(), CliError> {
        fs::create_dir_all(self.policies_dir())?;
        let path = self
            .policies_dir()
            .join(format!("{}.toml", policy.id.0));
        let content = toml_to_string(policy, &path)?;
        fs::write(&path, content)?;
        Ok(())
    }

    // ---------------------------------------------------------------
    // Integrations
    // ---------------------------------------------------------------

    pub fn load_integrations(&self) -> Result<Vec<Integration>, CliError> {
        load_toml_dir(&self.integrations_dir())
    }

    pub fn load_integration_by_name(&self, name: &str) -> Result<Integration, CliError> {
        let integrations = self.load_integrations()?;
        integrations
            .into_iter()
            .find(|i| i.name == name)
            .ok_or_else(|| CliError::NotFound(format!("Integration '{name}' not found")))
    }

    pub fn save_integration(&self, integration: &Integration) -> Result<(), CliError> {
        fs::create_dir_all(self.integrations_dir())?;
        let path = self
            .integrations_dir()
            .join(format!("{}.toml", slug(&integration.name)));
        let content = toml_to_string(integration, &path)?;
        fs::write(&path, content)?;
        Ok(())
    }

    // ---------------------------------------------------------------
    // Audit log
    // ---------------------------------------------------------------

    #[allow(dead_code)]
    pub fn audit_log_path(&self) -> PathBuf {
        self.audit_file()
    }

    /// Append a raw JSON line to the audit log.
    #[allow(dead_code)]
    pub fn append_audit_line(&self, line: &str) -> Result<(), CliError> {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.audit_file())?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    /// Read audit log lines (most recent `limit` entries).
    pub fn read_audit_lines(&self, limit: Option<usize>) -> Result<Vec<String>, CliError> {
        let path = self.audit_file();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&path)?;
        let lines: Vec<String> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.to_string())
            .collect();
        match limit {
            Some(n) => Ok(lines.into_iter().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect()),
            None => Ok(lines),
        }
    }
}

// ---------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------

/// Convert a name to a filesystem-safe slug.
fn slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// Deserialize a TOML string, mapping errors to CliError.
fn toml_from_str<T: serde::de::DeserializeOwned>(content: &str, path: &Path) -> Result<T, CliError> {
    toml::from_str(content).map_err(|e| {
        CliError::Store(format!("TOML parse error in {}: {e}", path.display()))
    })
}

/// Serialize a value to TOML string, mapping errors to CliError.
fn toml_to_string<T: serde::Serialize>(value: &T, path: &Path) -> Result<String, CliError> {
    toml::to_string_pretty(value).map_err(|e| {
        CliError::Store(format!(
            "TOML serialization error for {}: {e}",
            path.display()
        ))
    })
}

/// Load all TOML files from a directory into a Vec<T>.
fn load_toml_dir<T: serde::de::DeserializeOwned>(dir: &Path) -> Result<Vec<T>, CliError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let content = fs::read_to_string(&path)?;
            let item: T = toml_from_str(&content, &path)?;
            items.push(item);
        }
    }
    Ok(items)
}
