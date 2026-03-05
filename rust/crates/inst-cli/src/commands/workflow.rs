//! `inst workflow` — Workflow management commands.

use clap::Subcommand;
use inst_model::Workflow;

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_details, format_table, OutputFormat};

#[derive(Debug, Subcommand)]
pub enum WorkflowCommand {
    /// Create a new workflow.
    Create {
        /// Workflow name (e.g. "vendor-procurement").
        #[arg(long)]
        name: String,

        /// Function to group this workflow under.
        #[arg(long, alias = "function")]
        function: Option<String>,

        /// Description of the workflow.
        #[arg(long)]
        description: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// List all workflows.
    List {
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Show details of a specific workflow.
    Show {
        /// Workflow name.
        name: String,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Validate a workflow (or all workflows).
    Validate {
        /// Workflow name (validates all if omitted).
        name: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: WorkflowCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        WorkflowCommand::Create {
            name,
            function,
            description,
            format,
        } => create(name, function, description, format, ctx),
        WorkflowCommand::List { format } => list(format, ctx),
        WorkflowCommand::Show { name, format } => show(name, format, ctx),
        WorkflowCommand::Validate { name, format } => validate(name, format, ctx),
    }
}

fn create(
    name: String,
    function_name: Option<String>,
    description: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    // Check for duplicate names.
    let existing = ctx.load_workflows()?;
    if existing.iter().any(|w| w.name == name) {
        return Err(CliError::Validation(format!(
            "Workflow '{name}' already exists"
        )));
    }

    let org = ctx.load_organization()?;
    let mut workflow = Workflow::new(org.id, &name);
    workflow.description = description;

    // Resolve function reference if provided.
    if let Some(fn_name) = function_name {
        let function = ctx.load_function_by_name(&fn_name)?;
        workflow.function_id = Some(function.id);
    }

    ctx.save_workflow(&workflow)?;

    if format == OutputFormat::Json {
        output::print_json(&workflow)?;
    } else {
        println!("Created workflow '{name}'");
    }

    Ok(())
}

fn list(format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    let workflows = ctx.load_workflows()?;

    if format == OutputFormat::Json {
        output::print_json(&workflows)?;
    } else {
        let rows: Vec<Vec<String>> = workflows
            .iter()
            .map(|w| {
                vec![
                    w.name.clone(),
                    w.function_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    w.description.clone().unwrap_or_default(),
                ]
            })
            .collect();
        let table = format_table(&["NAME", "FUNCTION_ID", "DESCRIPTION"], &rows);
        print!("{table}");
    }

    Ok(())
}

fn show(name: String, format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    let workflow = ctx.load_workflow_by_name(&name)?;
    let nodes = ctx.load_nodes(&name)?;
    let edges = ctx.load_edges(&name)?;

    if format == OutputFormat::Json {
        let full = serde_json::json!({
            "workflow": workflow,
            "nodes": nodes,
            "edges": edges,
        });
        output::print_json(&full)?;
    } else {
        let details = format_details(&[
            ("ID", workflow.id.to_string()),
            ("Name", workflow.name.clone()),
            (
                "Function ID",
                workflow
                    .function_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "(none)".to_string()),
            ),
            (
                "Description",
                workflow.description.clone().unwrap_or_default(),
            ),
            ("Nodes", nodes.len().to_string()),
            ("Edges", edges.len().to_string()),
            ("Created", workflow.created_at.to_string()),
            ("Updated", workflow.updated_at.to_string()),
        ]);
        println!("{details}");

        if !nodes.is_empty() {
            println!("\nNodes:");
            for node in &nodes {
                let ntype = format!("{:?}", node.node_type).to_lowercase();
                println!("  - {} ({ntype})", node.label);
            }
        }

        if !edges.is_empty() {
            println!("\nEdges:");
            for edge in &edges {
                let label = edge.label.as_deref().unwrap_or("(unlabeled)");
                println!("  - {} -> {} [{}]", edge.from_node_id, edge.to_node_id, label);
            }
        }
    }

    Ok(())
}

fn validate(
    name: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    use inst_constraint::types::{ValidationContext, WorkflowData};

    let org = ctx.load_organization()?;
    let roles = ctx.load_roles()?;
    let policies = ctx.load_policies()?;
    let integrations = ctx.load_integrations()?;

    let workflow_names: Vec<String> = if let Some(ref n) = name {
        // Verify the workflow exists.
        ctx.load_workflow_by_name(n)?;
        vec![n.clone()]
    } else {
        ctx.load_workflows()?
            .into_iter()
            .map(|w| w.name)
            .collect()
    };

    let mut workflow_data = Vec::new();
    for wf_name in &workflow_names {
        let wf = ctx.load_workflow_by_name(wf_name)?;
        let nodes = ctx.load_nodes(wf_name)?;
        let edges = ctx.load_edges(wf_name)?;
        workflow_data.push(WorkflowData {
            workflow: wf,
            nodes,
            edges,
            edge_requirements: Vec::new(),
            edge_role_permissions: Vec::new(),
        });
    }

    let validation_ctx = ValidationContext {
        organization: org,
        roles,
        workflows: workflow_data,
        policies,
        integrations,
    };

    // Run constraint validation.
    // TODO: When inst-constraint exposes a validate_all() function, call it here.
    // For now, construct the result directly since constraint implementations
    // are being built by another agent.
    let result = run_validation(&validation_ctx);

    if format == OutputFormat::Json {
        output::print_json(&result)?;
    } else {
        if result.is_valid {
            let scope = name.as_deref().unwrap_or("all workflows");
            println!("Validation passed for {scope}.");
            let warnings = result.warnings();
            if !warnings.is_empty() {
                println!("\nWarnings ({}):", warnings.len());
                for w in warnings {
                    println!("  - [{}] {}", w.constraint_name, w.message);
                }
            }
        } else {
            let errors = result.errors();
            let warnings = result.warnings();
            println!(
                "Validation failed: {} error(s), {} warning(s)",
                errors.len(),
                warnings.len()
            );
            for e in errors {
                println!("  ERROR [{}]: {}", e.constraint_name, e.message);
                if let Some(ref suggestion) = e.suggestion {
                    println!("    Suggestion: {suggestion}");
                }
            }
            for w in warnings {
                println!("  WARN  [{}]: {}", w.constraint_name, w.message);
            }
            return Err(CliError::InvariantViolation(
                "Validation failed".to_string(),
            ));
        }
    }

    Ok(())
}

/// Run all constraints against the validation context.
///
/// This is a bridge function that will call into `inst-constraint` once it
/// exposes a top-level validation entry point. For now, it returns a result
/// with no violations as a placeholder.
fn run_validation(
    _ctx: &inst_constraint::types::ValidationContext,
) -> inst_constraint::types::ValidationResult {
    // TODO: Iterate over all registered constraints and collect violations.
    // The inst-constraint crate's constraint modules (GraphConnectivity,
    // AuthorityLevelCheck, EdgeNodeReference, etc.) implement the Constraint
    // trait. Once available, instantiate each and run validate() against ctx.
    inst_constraint::types::ValidationResult::from_violations(Vec::new())
}
