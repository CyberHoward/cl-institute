//! `inst node` — Decision node management commands.

use clap::Subcommand;
use inst_model::{DecisionNode, DecisionType, NodeType};

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_table, OutputFormat};

/// Parse a NodeType from its string representation.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum NodeTypeArg {
    Start,
    Intermediate,
    End,
}

impl From<NodeTypeArg> for NodeType {
    fn from(arg: NodeTypeArg) -> Self {
        match arg {
            NodeTypeArg::Start => NodeType::Start,
            NodeTypeArg::Intermediate => NodeType::Intermediate,
            NodeTypeArg::End => NodeType::End,
        }
    }
}

/// Parse a DecisionType from its string representation.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum DecisionTypeArg {
    Approval,
    Classification,
    Prioritization,
    Allocation,
    ExceptionHandling,
}

impl From<DecisionTypeArg> for DecisionType {
    fn from(arg: DecisionTypeArg) -> Self {
        match arg {
            DecisionTypeArg::Approval => DecisionType::Approval,
            DecisionTypeArg::Classification => DecisionType::Classification,
            DecisionTypeArg::Prioritization => DecisionType::Prioritization,
            DecisionTypeArg::Allocation => DecisionType::Allocation,
            DecisionTypeArg::ExceptionHandling => DecisionType::ExceptionHandling,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum NodeCommand {
    /// Create a new decision node in a workflow.
    Create {
        /// Workflow name to add the node to.
        #[arg(long)]
        workflow: String,

        /// Node type: start, intermediate, or end.
        #[arg(long, value_enum, rename_all = "lower")]
        r#type: NodeTypeArg,

        /// Human-readable label for the node.
        #[arg(long)]
        label: String,

        /// Decision type for this node.
        #[arg(long, value_enum)]
        decision_type: Option<DecisionTypeArg>,

        /// Minimum authority level required.
        #[arg(long, default_value_t = 0)]
        requires_authority: u32,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// List all nodes in a workflow.
    List {
        /// Workflow name.
        #[arg(long)]
        workflow: String,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: NodeCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        NodeCommand::Create {
            workflow,
            r#type,
            label,
            decision_type,
            requires_authority,
            format,
        } => create(workflow, r#type, label, decision_type, requires_authority, format, ctx),
        NodeCommand::List { workflow, format } => list(workflow, format, ctx),
    }
}

fn create(
    workflow_name: String,
    node_type: NodeTypeArg,
    label: String,
    decision_type: Option<DecisionTypeArg>,
    requires_authority: u32,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    // Verify workflow exists.
    let workflow = ctx.load_workflow_by_name(&workflow_name)?;

    // Check for duplicate labels within the workflow.
    let existing_nodes = ctx.load_nodes(&workflow_name)?;
    if existing_nodes.iter().any(|n| n.label == label) {
        return Err(CliError::Validation(format!(
            "Node with label '{label}' already exists in workflow '{workflow_name}'"
        )));
    }

    let mut node = DecisionNode::new(workflow.id, NodeType::from(node_type), &label);
    node.decision_type = decision_type.map(DecisionType::from);
    node.requires_authority = requires_authority;
    node.index = existing_nodes.len() as i32;

    ctx.save_node(&workflow_name, &node)?;

    if format == OutputFormat::Json {
        output::print_json(&node)?;
    } else {
        let ntype = format!("{:?}", node.node_type).to_lowercase();
        println!("Created node '{label}' ({ntype}) in workflow '{workflow_name}'");
    }

    Ok(())
}

fn list(workflow_name: String, format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    // Verify workflow exists.
    ctx.load_workflow_by_name(&workflow_name)?;

    let nodes = ctx.load_nodes(&workflow_name)?;

    if format == OutputFormat::Json {
        output::print_json(&nodes)?;
    } else {
        let rows: Vec<Vec<String>> = nodes
            .iter()
            .map(|n| {
                vec![
                    n.label.clone(),
                    format!("{:?}", n.node_type).to_lowercase(),
                    n.decision_type
                        .map(|dt| format!("{dt:?}").to_lowercase())
                        .unwrap_or_default(),
                    n.requires_authority.to_string(),
                ]
            })
            .collect();
        let table = format_table(&["LABEL", "TYPE", "DECISION_TYPE", "AUTHORITY"], &rows);
        print!("{table}");
    }

    Ok(())
}
