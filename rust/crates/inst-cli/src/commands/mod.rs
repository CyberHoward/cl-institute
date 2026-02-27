//! CLI command definitions using clap derive macros.

pub mod audit;
pub mod edge;
pub mod function;
pub mod graph;
pub mod init;
pub mod integration;
pub mod node;
pub mod org;
pub mod policy;
pub mod role;
pub mod workflow;

use clap::Subcommand;

use crate::context::ProjectContext;
use crate::error::CliError;

/// Top-level command enum dispatching to subcommand modules.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a new institution project.
    Init(init::InitArgs),

    /// Organization management.
    #[command(subcommand)]
    Org(org::OrgCommand),

    /// Role management.
    #[command(subcommand)]
    Role(role::RoleCommand),

    /// Function (workflow grouping) management.
    #[command(subcommand)]
    Function(function::FunctionCommand),

    /// Workflow management.
    #[command(subcommand)]
    Workflow(workflow::WorkflowCommand),

    /// Decision node management.
    #[command(subcommand)]
    Node(node::NodeCommand),

    /// Edge management.
    #[command(subcommand)]
    Edge(edge::EdgeCommand),

    /// Policy management.
    #[command(subcommand)]
    Policy(policy::PolicyCommand),

    /// Integration registry management.
    #[command(subcommand)]
    Integration(integration::IntegrationCommand),

    /// Graph export operations.
    #[command(subcommand)]
    Graph(graph::GraphCommand),

    /// Audit log operations.
    #[command(subcommand)]
    Audit(audit::AuditCommand),
}

impl Command {
    /// Execute the command.
    pub fn execute(self, ctx: &ProjectContext) -> Result<(), CliError> {
        match self {
            Command::Init(args) => init::execute(args, ctx),
            Command::Org(cmd) => {
                ctx.ensure_initialized()?;
                org::execute(cmd, ctx)
            }
            Command::Role(cmd) => {
                ctx.ensure_initialized()?;
                role::execute(cmd, ctx)
            }
            Command::Function(cmd) => {
                ctx.ensure_initialized()?;
                function::execute(cmd, ctx)
            }
            Command::Workflow(cmd) => {
                ctx.ensure_initialized()?;
                workflow::execute(cmd, ctx)
            }
            Command::Node(cmd) => {
                ctx.ensure_initialized()?;
                node::execute(cmd, ctx)
            }
            Command::Edge(cmd) => {
                ctx.ensure_initialized()?;
                edge::execute(cmd, ctx)
            }
            Command::Policy(cmd) => {
                ctx.ensure_initialized()?;
                policy::execute(cmd, ctx)
            }
            Command::Integration(cmd) => {
                ctx.ensure_initialized()?;
                integration::execute(cmd, ctx)
            }
            Command::Graph(cmd) => {
                ctx.ensure_initialized()?;
                graph::execute(cmd, ctx)
            }
            Command::Audit(cmd) => {
                ctx.ensure_initialized()?;
                audit::execute(cmd, ctx)
            }
        }
    }
}
