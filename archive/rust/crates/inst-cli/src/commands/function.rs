//! `inst function` — Function (workflow grouping) management commands.

use clap::Subcommand;
use inst_model::Function;

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_table, OutputFormat};

#[derive(Debug, Subcommand)]
pub enum FunctionCommand {
    /// Create a new function (workflow grouping).
    Create {
        /// Function name (e.g. "Procurement").
        #[arg(long)]
        name: String,

        /// Description of the function.
        #[arg(long)]
        description: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// List all functions.
    List {
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: FunctionCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        FunctionCommand::Create {
            name,
            description,
            format,
        } => create(name, description, format, ctx),
        FunctionCommand::List { format } => list(format, ctx),
    }
}

fn create(
    name: String,
    description: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    // Check for duplicate names.
    let existing = ctx.load_functions()?;
    if existing.iter().any(|f| f.name == name) {
        return Err(CliError::Validation(format!(
            "Function '{name}' already exists"
        )));
    }

    let org = ctx.load_organization()?;
    let mut function = Function::new(org.id, &name);
    function.description = description;

    ctx.save_function(&function)?;

    if format == OutputFormat::Json {
        output::print_json(&function)?;
    } else {
        println!("Created function '{name}'");
    }

    Ok(())
}

fn list(format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    let functions = ctx.load_functions()?;

    if format == OutputFormat::Json {
        output::print_json(&functions)?;
    } else {
        let rows: Vec<Vec<String>> = functions
            .iter()
            .map(|f| {
                vec![
                    f.name.clone(),
                    f.description.clone().unwrap_or_default(),
                ]
            })
            .collect();
        let table = format_table(&["NAME", "DESCRIPTION"], &rows);
        print!("{table}");
    }

    Ok(())
}
