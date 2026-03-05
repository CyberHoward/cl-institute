//! `inst init` — Initialize a new institution project.

use clap::Args;

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, OutputFormat};

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Name of the organization.
    #[arg(long)]
    pub name: String,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
}

pub fn execute(args: InitArgs, ctx: &ProjectContext) -> Result<(), CliError> {
    // Check if already initialized.
    let inst_file = ctx.root.join("institution.toml");
    if inst_file.exists() {
        return Err(CliError::Validation(
            "Project already initialized. Use `inst org update` to modify.".to_string(),
        ));
    }

    let org = ctx.init_project(&args.name)?;

    if args.format == OutputFormat::Json {
        output::print_json(&org)?;
    } else {
        println!(
            "Initialized institution project '{}' at {}",
            org.name,
            ctx.root.display()
        );
        println!("  Organization ID: {}", org.id);
        println!("  Created: {}", org.created_at);
    }

    Ok(())
}
