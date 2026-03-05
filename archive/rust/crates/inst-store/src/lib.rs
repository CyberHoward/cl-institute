//! # inst-store
//!
//! File-system persistence for institution-as-code projects.
//!
//! This crate provides `InstitutionStore`, which reads and writes the
//! on-disk directory structure that defines an institution:
//!
//! ```text
//! acme-foundation/
//! ├── institution.toml
//! ├── roles/*.toml
//! ├── workflows/
//! │   └── procurement/
//! │       ├── workflow.toml
//! │       ├── decisions/*.toml
//! │       ├── edges/*.toml
//! │       └── policies/*.md
//! ├── integrations/*.toml
//! ├── compiled/
//! └── audit/
//! ```
//!
//! Human-authored files use TOML. Machine interchange uses JSON.
//! Policy files are markdown with optional TOML frontmatter.

pub mod error;
pub mod load;
pub mod project;
pub mod save;
pub mod store;

pub use error::{Result, StoreError};
pub use load::{load_institution, load_integrations, load_organization, load_roles, load_workflow};
pub use project::{InstitutionProject, WorkflowDefinition};
pub use save::{
    save_institution, save_integrations, save_organization, save_roles, save_workflow,
};
pub use store::InstitutionStore;
