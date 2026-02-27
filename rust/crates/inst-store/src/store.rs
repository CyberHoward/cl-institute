use std::path::{Path, PathBuf};

use inst_model::{Integration, Organization, OrganizationalRole};

use crate::error::Result;
use crate::load;
use crate::project::{InstitutionProject, WorkflowDefinition};
use crate::save;

/// The main file-system store for an institution-as-code project.
///
/// `InstitutionStore` wraps a project root path and provides methods
/// to load and save the entire institution model or individual components.
///
/// # Example
///
/// ```no_run
/// use inst_store::InstitutionStore;
///
/// let store = InstitutionStore::new("/path/to/acme-foundation");
/// let project = store.load().unwrap();
/// println!("Loaded: {}", project.organization.name);
/// ```
#[derive(Debug, Clone)]
pub struct InstitutionStore {
    root: PathBuf,
}

impl InstitutionStore {
    /// Create a new store rooted at the given path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Return the project root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    // -----------------------------------------------------------------------
    // Load operations
    // -----------------------------------------------------------------------

    /// Load the complete institution project from the root directory.
    pub fn load(&self) -> Result<InstitutionProject> {
        load::load_institution(&self.root)
    }

    /// Load just the organization metadata from `institution.toml`.
    pub fn load_organization(&self) -> Result<Organization> {
        load::load_organization(&self.root)
    }

    /// Load all organizational roles from `roles/*.toml`.
    pub fn load_roles(&self) -> Result<Vec<OrganizationalRole>> {
        load::load_roles(&self.root)
    }

    /// Load a specific workflow by its directory name.
    ///
    /// The `name` should be the directory name under `workflows/`
    /// (e.g., `"procurement"`).
    pub fn load_workflow(&self, name: &str) -> Result<WorkflowDefinition> {
        let workflow_dir = self.root.join("workflows").join(name);
        load::load_workflow(&workflow_dir)
    }

    /// Load all integrations from `integrations/*.toml`.
    pub fn load_integrations(&self) -> Result<Vec<Integration>> {
        load::load_integrations(&self.root)
    }

    // -----------------------------------------------------------------------
    // Save operations
    // -----------------------------------------------------------------------

    /// Save the complete institution project to the root directory.
    pub fn save(&self, project: &InstitutionProject) -> Result<()> {
        save::save_institution(project)
    }

    /// Save the organization metadata to `institution.toml`.
    pub fn save_organization(&self, org: &Organization) -> Result<()> {
        save::save_organization(&self.root, org)
    }

    /// Save all organizational roles to `roles/*.toml`.
    pub fn save_roles(&self, roles: &[OrganizationalRole]) -> Result<()> {
        save::save_roles(&self.root, roles)
    }

    /// Save a workflow definition to its directory under `workflows/`.
    pub fn save_workflow(&self, wf: &WorkflowDefinition) -> Result<()> {
        save::save_workflow(&self.root, wf)
    }

    /// Save all integrations to `integrations/*.toml`.
    pub fn save_integrations(&self, integrations: &[Integration]) -> Result<()> {
        save::save_integrations(&self.root, integrations)
    }
}
