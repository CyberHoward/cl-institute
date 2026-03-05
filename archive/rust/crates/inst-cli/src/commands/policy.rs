//! `inst policy` — Policy management commands.

use clap::Subcommand;
use inst_model::{Policy, PolicyStrength};

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_table, OutputFormat};

/// Parse a PolicyStrength from its string representation.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum PolicyStrengthArg {
    Constraint,
    Procedure,
    Preference,
    Context,
}

impl From<PolicyStrengthArg> for PolicyStrength {
    fn from(arg: PolicyStrengthArg) -> Self {
        match arg {
            PolicyStrengthArg::Constraint => PolicyStrength::Constraint,
            PolicyStrengthArg::Procedure => PolicyStrength::Procedure,
            PolicyStrengthArg::Preference => PolicyStrength::Preference,
            PolicyStrengthArg::Context => PolicyStrength::Context,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum PolicyCommand {
    /// Attach a policy to a scope.
    Attach {
        /// Dot-separated scope path (e.g. "procurement.*").
        #[arg(long)]
        scope: String,

        /// Policy strength level.
        #[arg(long, value_enum)]
        strength: PolicyStrengthArg,

        /// Policy text.
        #[arg(long)]
        text: String,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// List policies, optionally filtered by scope.
    List {
        /// Filter by scope (exact or prefix match).
        #[arg(long)]
        scope: Option<String>,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: PolicyCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        PolicyCommand::Attach {
            scope,
            strength,
            text,
            format,
        } => attach(scope, strength, text, format, ctx),
        PolicyCommand::List { scope, format } => list(scope, format, ctx),
    }
}

fn attach(
    scope: String,
    strength: PolicyStrengthArg,
    text: String,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    let org = ctx.load_organization()?;
    let policy = Policy::new(org.id, &scope, PolicyStrength::from(strength), &text);

    ctx.save_policy(&policy)?;

    if format == OutputFormat::Json {
        output::print_json(&policy)?;
    } else {
        let strength_str = format!("{:?}", PolicyStrength::from(strength)).to_lowercase();
        println!("Attached {strength_str} policy to scope '{scope}'");
    }

    Ok(())
}

fn list(
    scope: Option<String>,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    let policies = ctx.load_policies()?;

    let filtered: Vec<&Policy> = if let Some(ref scope_filter) = scope {
        policies
            .iter()
            .filter(|p| scope_matches(&p.scope, scope_filter))
            .collect()
    } else {
        policies.iter().collect()
    };

    if format == OutputFormat::Json {
        output::print_json(&filtered)?;
    } else {
        let rows: Vec<Vec<String>> = filtered
            .iter()
            .map(|p| {
                let strength = format!("{:?}", p.strength).to_lowercase();
                let text_preview = if p.text.len() > 60 {
                    format!("{}...", &p.text[..57])
                } else {
                    p.text.clone()
                };
                vec![p.scope.clone(), strength, text_preview]
            })
            .collect();
        let table = format_table(&["SCOPE", "STRENGTH", "TEXT"], &rows);
        print!("{table}");
    }

    Ok(())
}

/// Check if a policy scope matches a query scope.
///
/// Matching rules:
/// - Exact match: "procurement.vendor-selection" matches "procurement.vendor-selection"
/// - Parent match: policy scope "procurement.*" matches query "procurement.vendor-selection"
/// - Wildcard: a scope ending in ".*" matches any child scope
/// - Hierarchy: a policy at "procurement" matches query "procurement.vendor-selection"
/// - Reverse: a query for "procurement" finds policies at "procurement.*"
fn scope_matches(policy_scope: &str, query_scope: &str) -> bool {
    // Exact match.
    if policy_scope == query_scope {
        return true;
    }

    // Normalize: strip trailing ".*" for comparison.
    let policy_base = policy_scope.strip_suffix(".*").unwrap_or(policy_scope);
    let query_base = query_scope.strip_suffix(".*").unwrap_or(query_scope);

    // Policy is a parent of the query.
    if query_base.starts_with(policy_base)
        && query_base.as_bytes().get(policy_base.len()) == Some(&b'.')
    {
        return true;
    }

    // Query is a parent of the policy (finding child policies).
    if policy_base.starts_with(query_base)
        && policy_base.as_bytes().get(query_base.len()) == Some(&b'.')
    {
        return true;
    }

    false
}
