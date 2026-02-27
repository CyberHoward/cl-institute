//! `inst integration` — Integration registry management commands.

use chrono::Utc;
use clap::Subcommand;
use inst_model::{Capability, CapabilityId, Integration};

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_table, OutputFormat};

#[derive(Debug, Subcommand)]
pub enum IntegrationCommand {
    /// Register a new integration.
    Register {
        /// Integration name (e.g. "docusign").
        #[arg(long)]
        name: String,

        /// Description of the integration.
        #[arg(long)]
        description: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Add a capability to an integration, or manage capabilities.
    #[command(subcommand)]
    Capability(CapabilityCommand),

    /// List all registered integrations.
    List {
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
pub enum CapabilityCommand {
    /// Add a capability to an integration.
    Add {
        /// Integration name.
        #[arg(long)]
        integration: String,

        /// Capability name (e.g. "route_for_signature").
        #[arg(long)]
        name: String,

        /// Description of the capability.
        #[arg(long)]
        description: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: IntegrationCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        IntegrationCommand::Register {
            name,
            description,
            format,
        } => register(name, description, format, ctx),
        IntegrationCommand::Capability(cap_cmd) => match cap_cmd {
            CapabilityCommand::Add {
                integration,
                name,
                description,
                format,
            } => add_capability(integration, name, description, format, ctx),
        },
        IntegrationCommand::List { format } => list(format, ctx),
    }
}

fn register(
    name: String,
    description: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    // Check for duplicate names.
    let existing = ctx.load_integrations()?;
    if existing.iter().any(|i| i.name == name) {
        return Err(CliError::Validation(format!(
            "Integration '{name}' already exists"
        )));
    }

    let org = ctx.load_organization()?;
    let mut integration = Integration::new(org.id, &name);
    integration.description = description;

    ctx.save_integration(&integration)?;

    if format == OutputFormat::Json {
        output::print_json(&integration)?;
    } else {
        println!("Registered integration '{name}'");
    }

    Ok(())
}

fn add_capability(
    integration_name: String,
    cap_name: String,
    description: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    let mut integration = ctx.load_integration_by_name(&integration_name)?;

    // Check for duplicate capability names.
    if integration.capabilities.iter().any(|c| c.name == cap_name) {
        return Err(CliError::Validation(format!(
            "Capability '{cap_name}' already exists in integration '{integration_name}'"
        )));
    }

    let capability = Capability {
        id: CapabilityId::new(),
        name: cap_name.clone(),
        description,
        input_schema: None,
        output_schema: None,
    };

    integration.capabilities.push(capability);
    integration.updated_at = Utc::now();

    ctx.save_integration(&integration)?;

    if format == OutputFormat::Json {
        output::print_json(&integration)?;
    } else {
        println!(
            "Added capability '{cap_name}' to integration '{integration_name}'"
        );
    }

    Ok(())
}

fn list(format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    let integrations = ctx.load_integrations()?;

    if format == OutputFormat::Json {
        output::print_json(&integrations)?;
    } else {
        let rows: Vec<Vec<String>> = integrations
            .iter()
            .map(|i| {
                vec![
                    i.name.clone(),
                    i.capabilities.len().to_string(),
                    i.description.clone().unwrap_or_default(),
                ]
            })
            .collect();
        let table = format_table(&["NAME", "CAPABILITIES", "DESCRIPTION"], &rows);
        print!("{table}");
    }

    Ok(())
}
