//! `inst audit` — Audit log operations.

use clap::Subcommand;

use crate::context::ProjectContext;
use crate::error::CliError;
use crate::output::{self, format_table, OutputFormat};

#[derive(Debug, Subcommand)]
pub enum AuditCommand {
    /// Show recent audit log entries.
    Log {
        /// Number of most recent entries to show.
        #[arg(long, default_value_t = 20)]
        last: usize,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Verify the integrity of the audit log chain.
    Verify {
        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}

pub fn execute(cmd: AuditCommand, ctx: &ProjectContext) -> Result<(), CliError> {
    match cmd {
        AuditCommand::Log { last, format } => log_entries(last, format, ctx),
        AuditCommand::Verify { format } => verify(format, ctx),
    }
}

fn log_entries(
    last: usize,
    format: OutputFormat,
    ctx: &ProjectContext,
) -> Result<(), CliError> {
    let lines = ctx.read_audit_lines(Some(last))?;

    if lines.is_empty() {
        if format == OutputFormat::Json {
            output::print_json(&serde_json::json!({"entries": []}))?;
        } else {
            println!("No audit log entries found.");
        }
        return Ok(());
    }

    // Parse JSONL entries.
    let entries: Vec<serde_json::Value> = lines
        .iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    if format == OutputFormat::Json {
        output::print_json(&serde_json::json!({"entries": entries}))?;
    } else {
        let rows: Vec<Vec<String>> = entries
            .iter()
            .map(|entry| {
                vec![
                    entry
                        .get("sequence")
                        .and_then(|v| v.as_u64())
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    entry
                        .get("timestamp")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    entry
                        .get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    format_actor(entry.get("actor")),
                ]
            })
            .collect();
        let table = format_table(&["SEQ", "TIMESTAMP", "ACTION", "ACTOR"], &rows);
        print!("{table}");
    }

    Ok(())
}

fn verify(format: OutputFormat, ctx: &ProjectContext) -> Result<(), CliError> {
    let lines = ctx.read_audit_lines(None)?;

    if lines.is_empty() {
        if format == OutputFormat::Json {
            output::print_json(&serde_json::json!({
                "valid": true,
                "entries_checked": 0,
                "errors": [],
            }))?;
        } else {
            println!("Audit log is empty. Nothing to verify.");
        }
        return Ok(());
    }

    // Parse all entries and verify the hash chain.
    let mut errors: Vec<String> = Vec::new();
    let mut prev_hash = String::new();
    let mut entries_checked: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let entry: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("Line {}: failed to parse JSON: {e}", i + 1));
                continue;
            }
        };

        entries_checked += 1;

        // Verify prev_hash chain.
        let entry_prev_hash = entry
            .get("prev_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !prev_hash.is_empty() && entry_prev_hash != prev_hash {
            errors.push(format!(
                "Sequence {}: prev_hash mismatch (expected '{prev_hash}', got '{entry_prev_hash}')",
                entry
                    .get("sequence")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
            ));
        }

        // Update prev_hash to this entry's hash for next iteration.
        prev_hash = entry
            .get("entry_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
    }

    let is_valid = errors.is_empty();

    if format == OutputFormat::Json {
        output::print_json(&serde_json::json!({
            "valid": is_valid,
            "entries_checked": entries_checked,
            "errors": errors,
        }))?;
    } else if is_valid {
        println!(
            "Audit log integrity verified: {entries_checked} entries, chain intact."
        );
    } else {
        println!(
            "Audit log verification FAILED: {entries_checked} entries checked, {} error(s):",
            errors.len()
        );
        for err in &errors {
            println!("  - {err}");
        }
        return Err(CliError::InvariantViolation(
            "Audit log chain integrity violation".to_string(),
        ));
    }

    Ok(())
}

/// Format an actor JSON value for human-readable display.
fn format_actor(actor: Option<&serde_json::Value>) -> String {
    match actor {
        Some(a) => {
            let actor_type = a.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
            match actor_type {
                "user" => a
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user")
                    .to_string(),
                "agent" => {
                    let agent_id = a.get("agent_id").and_then(|v| v.as_str()).unwrap_or("?");
                    let role = a.get("role").and_then(|v| v.as_str()).unwrap_or("?");
                    format!("agent:{agent_id}({role})")
                }
                "system" => "system".to_string(),
                _ => actor_type.to_string(),
            }
        }
        None => "(unknown)".to_string(),
    }
}
