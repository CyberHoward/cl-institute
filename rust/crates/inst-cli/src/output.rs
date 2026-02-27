//! Output formatting for CLI commands.
//!
//! All commands support `--format json` for machine consumption (the TypeScript
//! layer parses this) and default to human-readable text output.

use serde::Serialize;

/// Output format selection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text (default).
    #[default]
    Text,
    /// Machine-readable JSON.
    Json,
}

/// Unified output wrapper that handles both JSON and human-readable rendering.
///
/// Commands construct an `OutputPayload` and call `render()` to emit it.
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct OutputPayload<T: Serialize> {
    #[serde(skip)]
    format: OutputFormat,
    #[serde(skip)]
    human_text: String,
    #[serde(flatten)]
    data: T,
}

#[allow(dead_code)]
impl<T: Serialize> OutputPayload<T> {
    /// Create a new output payload.
    ///
    /// `human_text` is what gets printed in text mode.
    /// `data` is what gets serialized in JSON mode.
    pub fn new(format: OutputFormat, human_text: impl Into<String>, data: T) -> Self {
        Self {
            format,
            human_text: human_text.into(),
            data,
        }
    }

    /// Render the payload to stdout.
    pub fn render(&self) -> Result<(), CliError> {
        match self.format {
            OutputFormat::Text => {
                println!("{}", self.human_text);
                Ok(())
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&self.data)
                    .map_err(|e| CliError::Serialization(e.to_string()))?;
                println!("{json}");
                Ok(())
            }
        }
    }
}

/// Print a success message (non-JSON).
#[allow(dead_code)]
pub fn print_success(msg: &str) {
    println!("{msg}");
}

/// Print a JSON value to stdout.
pub fn print_json<T: Serialize>(value: &T) -> Result<(), CliError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| CliError::Serialization(e.to_string()))?;
    println!("{json}");
    Ok(())
}

/// Format a table from rows of key-value pairs for human-readable output.
pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return "(no results)".to_string();
    }

    // Calculate column widths.
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let mut output = String::new();

    // Header row.
    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:<width$}", h, width = widths[i]))
        .collect();
    output.push_str(&header_line.join("  "));
    output.push('\n');

    // Separator.
    let sep_line: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    output.push_str(&sep_line.join("  "));
    output.push('\n');

    // Data rows.
    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = widths.get(i).copied().unwrap_or(cell.len());
                format!("{:<width$}", cell, width = w)
            })
            .collect();
        output.push_str(&cells.join("  "));
        output.push('\n');
    }

    output
}

/// Format a detail view from key-value pairs for human-readable output.
pub fn format_details(pairs: &[(&str, String)]) -> String {
    let max_key_len = pairs.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    pairs
        .iter()
        .map(|(k, v)| format!("{:>width$}: {v}", k, width = max_key_len))
        .collect::<Vec<_>>()
        .join("\n")
}

// Re-export the CLI error type for use in output functions.
use crate::error::CliError;
