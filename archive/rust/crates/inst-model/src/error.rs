use thiserror::Error;

use crate::ids::*;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Organization not found: {0}")]
    OrganizationNotFound(OrganizationId),

    #[error("Role not found: {0}")]
    RoleNotFound(RoleId),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(WorkflowId),

    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),

    #[error("Policy not found: {0}")]
    PolicyNotFound(PolicyId),

    #[error("Integration not found: {0}")]
    IntegrationNotFound(IntegrationId),

    #[error("Instance not found: {0}")]
    InstanceNotFound(InstanceId),

    #[error("Duplicate name '{name}' in scope {scope}")]
    DuplicateName { name: String, scope: String },

    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
