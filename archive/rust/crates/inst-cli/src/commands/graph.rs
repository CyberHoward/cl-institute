//! `inst graph` — Graph export operations.

use clap::Subcommand;

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output;

/// Export format for graph output.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum GraphFormat {
    /// Graphviz DOT format.
    Dot,
    /// JSON format.
    Json,
}

#[derive(Debug, Subcommand)]
pub enum GraphCommand {
    /// Export a workflow graph.
    Export {
        /// Workflow name.
        #[arg(long)]
        workflow: String,

        /// Export format.
        #[arg(long, value_enum)]
        format: GraphFormat,
    },
}

pub fn execute(cmd: GraphCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        GraphCommand::Export { workflow, format } => export(workflow, format, ctx),
    }
}

fn export(
    workflow_name: String,
    format: GraphFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    let workflow = ctx.load_workflow_by_name(&workflow_name)?;
    let nodes = ctx.load_nodes(&workflow_name)?;
    let edges = ctx.load_edges(&workflow_name)?;

    match format {
        GraphFormat::Dot => {
            let dot = generate_dot(&workflow.name, &nodes, &edges);
            println!("{dot}");
        }
        GraphFormat::Json => {
            let graph_json = serde_json::json!({
                "workflow": workflow,
                "nodes": nodes,
                "edges": edges,
            });
            output::print_json(&graph_json)?;
        }
    }

    Ok(())
}

/// Generate a Graphviz DOT representation of the workflow graph.
fn generate_dot(
    workflow_name: &str,
    nodes: &[inst_model::DecisionNode],
    edges: &[inst_model::Edge],
) -> String {
    let mut dot = String::new();
    dot.push_str(&format!("digraph \"{}\" {{\n", escape_dot(workflow_name)));
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box, style=rounded];\n\n");

    // Emit nodes with shape based on type.
    for node in nodes {
        let shape = match node.node_type {
            inst_model::NodeType::Start => "ellipse",
            inst_model::NodeType::Intermediate => "box",
            inst_model::NodeType::End => "doublecircle",
        };
        let mut attrs = format!("shape={shape}");
        if let Some(ref dt) = node.decision_type {
            let dt_str = format!("{dt:?}").to_lowercase();
            attrs.push_str(&format!(", xlabel=\"{dt_str}\""));
        }
        dot.push_str(&format!(
            "  \"{}\" [label=\"{}\", {attrs}];\n",
            node.id.0,
            escape_dot(&node.label),
        ));
    }

    dot.push('\n');

    // Emit edges.
    for edge in edges {
        let label = edge
            .label
            .as_deref()
            .unwrap_or("");
        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            edge.from_node_id.0,
            edge.to_node_id.0,
            escape_dot(label),
        ));
    }

    dot.push_str("}\n");
    dot
}

/// Escape special characters for DOT format.
fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
