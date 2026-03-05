//! `inst` — CLI for the Intelligent Institution Initiative.
//!
//! Stateless CLI binary serving as the primary interface for both humans and
//! AI agents. All state lives in the project directory. Every command is a
//! transaction: validate, mutate, commit.
//!
//! This binary is the contract between the Rust core and the TypeScript layer.
//! All communication between layers is via CLI invocations with JSON output.

mod commands;
mod context;
mod error;
mod output;

use std::path::PathBuf;
use std::process;

use clap::Parser;

use commands::Command;
use context::ProjectContext;

/// inst — Intelligent Institution CLI
///
/// The primary interface for modeling, validating, and operating institutional
/// decision-making structures. Supports both human-readable and JSON output
/// for integration with the TypeScript orchestration layer.
#[derive(Debug, Parser)]
#[command(name = "inst", version, about, long_about = None)]
struct Cli {
    /// Path to the institution project directory.
    ///
    /// Defaults to the current directory. Can also be set via the
    /// INST_PROJECT environment variable.
    #[arg(long, global = true, env = "INST_PROJECT")]
    project: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

fn main() {
    let cli = Cli::parse();

    let ctx = match ProjectContext::resolve(cli.project.as_deref()) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(e.exit_code());
        }
    };

    if let Err(e) = cli.command.execute(&ctx) {
        eprintln!("Error: {e}");
        process::exit(e.exit_code());
    }
}
