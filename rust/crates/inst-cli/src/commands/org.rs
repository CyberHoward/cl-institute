//! `inst org` — Organization management commands.

use chrono::Utc;
use clap::Subcommand;

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_details, OutputFormat};

#[derive(Debug, Subcommand)]
pub enum OrgCommand {
    /// Show the organization details.
    Show {
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Update the organization.
    Update {
        /// New organization name.
        #[arg(long)]
        name: Option<String>,

        /// New organization description.
        #[arg(long)]
        description: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: OrgCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        OrgCommand::Show { format } => show(format, ctx),
        OrgCommand::Update {
            name,
            description,
            format,
        } => update(name, description, format, ctx),
    }
}

fn show(format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    let org = ctx.load_organization()?;

    if format == OutputFormat::Json {
        output::print_json(&org)?;
    } else {
        let details = format_details(&[
            ("ID", org.id.to_string()),
            ("Name", org.name.clone()),
            (
                "Description",
                org.description.clone().unwrap_or_default(),
            ),
            ("Created", org.created_at.to_string()),
            ("Updated", org.updated_at.to_string()),
        ]);
        println!("{details}");
    }

    Ok(())
}

fn update(
    name: Option<String>,
    description: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    let mut org = ctx.load_organization()?;

    if name.is_none() && description.is_none() {
        return Err(CliError::Validation(
            "Nothing to update. Provide --name or --description.".to_string(),
        ));
    }

    if let Some(n) = name {
        org.name = n;
    }
    if let Some(d) = description {
        org.description = Some(d);
    }
    org.updated_at = Utc::now();

    ctx.save_organization(&org)?;

    if format == OutputFormat::Json {
        output::print_json(&org)?;
    } else {
        println!("Organization updated successfully.");
    }

    Ok(())
}
