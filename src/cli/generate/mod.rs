use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

use clap::{Args, Subcommand};
use schema::GenerateSchemaArgs;
use shell::GenerateShellArgs;

use super::Cli;
use crate::builder::BuildError;

pub mod schema;
pub mod shell;

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub command: GenerateCommand,
}

#[derive(Debug, Subcommand)]
pub enum GenerateCommand {
    /// Generate a JSON schema for the config files
    Schema(GenerateSchemaArgs),
    /// Generate shell completions
    Shell(GenerateShellArgs),
}

impl GenerateArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), BuildError> {
        match &self.command {
            GenerateCommand::Schema(args) => args.run(cli),
            GenerateCommand::Shell(args) => args.run(cli),
        }
    }
}

/// Resolves the output path for a file, given an optional base path and a filename.
///
/// - If the base path is a directory, the filename will be appended to it.
/// - If the base path is a file, it will be used as-is.
/// - If no base path is provided, `None` is returned.
pub(super) fn resolve_path(path: Option<&PathBuf>, filename: &str) -> Option<PathBuf> {
    if let Some(base) = path {
        let resolved = if base.is_dir() {
            base.join(filename)
        } else {
            base.clone()
        };

        return Some(resolved);
    }

    None
}

/// Writes the given content to the specified path, or to stdout if no path is provided.
pub(super) fn write_output(content: &[u8], path: Option<PathBuf>) -> Result<(), BuildError> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    } else {
        io::stdout().write_all(content)?;
    }

    Ok(())
}
