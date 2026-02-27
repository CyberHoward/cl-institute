//! `inst role` — Organizational role management commands.

use chrono::Utc;
use clap::Subcommand;
use inst_model::OrganizationalRole;

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_details, format_table, OutputFormat};

#[derive(Debug, Subcommand)]
pub enum RoleCommand {
    /// Create a new organizational role.
    Create {
        /// Role name (e.g. "compliance-officer").
        #[arg(long)]
        name: String,

        /// Authority level (0 = lowest).
        #[arg(long, default_value_t = 0)]
        authority_level: u32,

        /// Description of the role.
        #[arg(long)]
        description: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// List all organizational roles.
    List {
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Show details of a specific role.
    Show {
        /// Role name.
        name: String,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Update an existing role.
    Update {
        /// Role name to update.
        name: String,

        /// New authority level.
        #[arg(long)]
        authority_level: Option<u32>,

        /// New description.
        #[arg(long)]
        description: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Delete a role.
    Delete {
        /// Role name to delete.
        name: String,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: RoleCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        RoleCommand::Create {
            name,
            authority_level,
            description,
            format,
        } => create(name, authority_level, description, format, ctx),
        RoleCommand::List { format } => list(format, ctx),
        RoleCommand::Show { name, format } => show(name, format, ctx),
        RoleCommand::Update {
            name,
            authority_level,
            description,
            format,
        } => update(name, authority_level, description, format, ctx),
        RoleCommand::Delete { name, format } => delete(name, format, ctx),
    }
}

fn create(
    name: String,
    authority_level: u32,
    description: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    // Check for duplicate names.
    let existing = ctx.load_roles()?;
    if existing.iter().any(|r| r.name == name) {
        return Err(CliError::Validation(format!(
            "Role '{name}' already exists"
        )));
    }

    let org = ctx.load_organization()?;
    let mut role = OrganizationalRole::new(org.id, &name);
    role.authority_level = authority_level;
    role.description = description;

    ctx.save_role(&role)?;

    if format == OutputFormat::Json {
        output::print_json(&role)?;
    } else {
        println!("Created role '{name}' (authority level: {authority_level})");
    }

    Ok(())
}

fn list(format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    let roles = ctx.load_roles()?;

    if format == OutputFormat::Json {
        output::print_json(&roles)?;
    } else {
        let rows: Vec<Vec<String>> = roles
            .iter()
            .map(|r| {
                vec![
                    r.name.clone(),
                    r.authority_level.to_string(),
                    r.description.clone().unwrap_or_default(),
                ]
            })
            .collect();
        let table = format_table(&["NAME", "AUTHORITY", "DESCRIPTION"], &rows);
        print!("{table}");
    }

    Ok(())
}

fn show(name: String, format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    let role = ctx.load_role_by_name(&name)?;

    if format == OutputFormat::Json {
        output::print_json(&role)?;
    } else {
        let details = format_details(&[
            ("ID", role.id.to_string()),
            ("Name", role.name.clone()),
            ("Authority Level", role.authority_level.to_string()),
            (
                "Description",
                role.description.clone().unwrap_or_default(),
            ),
            ("Created", role.created_at.to_string()),
            ("Updated", role.updated_at.to_string()),
        ]);
        println!("{details}");
    }

    Ok(())
}

fn update(
    name: String,
    authority_level: Option<u32>,
    description: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    let mut role = ctx.load_role_by_name(&name)?;

    if authority_level.is_none() && description.is_none() {
        return Err(CliError::Validation(
            "Nothing to update. Provide --authority-level or --description.".to_string(),
        ));
    }

    if let Some(al) = authority_level {
        role.authority_level = al;
    }
    if let Some(d) = description {
        role.description = Some(d);
    }
    role.updated_at = Utc::now();

    ctx.save_role(&role)?;

    if format == OutputFormat::Json {
        output::print_json(&role)?;
    } else {
        println!("Updated role '{name}'.");
    }

    Ok(())
}

fn delete(name: String, format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    // Verify it exists first.
    let role = ctx.load_role_by_name(&name)?;

    ctx.delete_role_file(&name)?;

    if format == OutputFormat::Json {
        output::print_json(&serde_json::json!({ "deleted": name, "id": role.id.to_string() }))?;
    } else {
        println!("Deleted role '{name}'.");
    }

    Ok(())
}
