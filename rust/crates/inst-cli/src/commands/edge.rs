//! `inst edge` — Edge management commands.

use clap::Subcommand;
use inst_model::Edge;

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_table, OutputFormat};

#[derive(Debug, Subcommand)]
pub enum EdgeCommand {
    /// Create a new edge between two nodes in a workflow.
    Create {
        /// Workflow name.
        #[arg(long)]
        workflow: String,

        /// Source node label.
        #[arg(long)]
        from: String,

        /// Target node label.
        #[arg(long)]
        to: String,

        /// Edge label.
        #[arg(long)]
        label: Option<String>,

        /// Edge rule (natural language specification).
        #[arg(long)]
        rule: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// List all edges in a workflow.
    List {
        /// Workflow name.
        #[arg(long)]
        workflow: String,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: EdgeCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        EdgeCommand::Create {
            workflow,
            from,
            to,
            label,
            rule,
            format,
        } => create(workflow, from, to, label, rule, format, ctx),
        EdgeCommand::List { workflow, format } => list(workflow, format, ctx),
    }
}

fn create(
    workflow_name: String,
    from_label: String,
    to_label: String,
    label: Option<String>,
    rule: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    // Verify workflow exists.
    let workflow = ctx.load_workflow_by_name(&workflow_name)?;

    // Resolve node labels to node IDs.
    let from_node = ctx.load_node_by_label(&workflow_name, &from_label)?;
    let to_node = ctx.load_node_by_label(&workflow_name, &to_label)?;

    let mut edge = Edge::new(workflow.id, from_node.id, to_node.id);
    edge.label = label;
    edge.rule = rule;

    ctx.save_edge(&workflow_name, &edge)?;

    if format == OutputFormat::Json {
        output::print_json(&edge)?;
    } else {
        println!(
            "Created edge from '{from_label}' to '{to_label}' in workflow '{workflow_name}'"
        );
    }

    Ok(())
}

fn list(
    workflow_name: String,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    // Verify workflow exists.
    ctx.load_workflow_by_name(&workflow_name)?;

    let edges = ctx.load_edges(&workflow_name)?;
    let nodes = ctx.load_nodes(&workflow_name)?;

    // Build a lookup from node ID to label for human-readable output.
    let node_label = |id: &inst_model::NodeId| -> String {
        nodes
            .iter()
            .find(|n| n.id == *id)
            .map(|n| n.label.clone())
            .unwrap_or_else(|| id.to_string())
    };

    if format == OutputFormat::Json {
        output::print_json(&edges)?;
    } else {
        let rows: Vec<Vec<String>> = edges
            .iter()
            .map(|e| {
                vec![
                    node_label(&e.from_node_id),
                    node_label(&e.to_node_id),
                    e.label.clone().unwrap_or_default(),
                    e.rule.clone().unwrap_or_default(),
                ]
            })
            .collect();
        let table = format_table(&["FROM", "TO", "LABEL", "RULE"], &rows);
        print!("{table}");
    }

    Ok(())
}
