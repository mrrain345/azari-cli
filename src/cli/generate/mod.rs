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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    // resolve_path tests

    #[test]
    fn resolve_path_none_returns_none() {
        assert_eq!(resolve_path(None, "schema.json"), None);
    }

    #[test]
    fn resolve_path_with_directory_appends_filename() {
        let dir = tempdir().unwrap();
        let result = resolve_path(Some(&dir.path().to_path_buf()), "schema.json");
        assert_eq!(result, Some(dir.path().join("schema.json")));
    }

    #[test]
    fn resolve_path_with_file_path_returns_as_is() {
        let dir = tempdir().unwrap();
        // A path that doesn't exist is not a directory, so it's treated as a file path
        let file = dir.path().join("output.json");
        let result = resolve_path(Some(&file), "schema.json");
        assert_eq!(result, Some(file));
    }

    #[test]
    fn resolve_path_with_existing_file_returns_as_is() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("existing.json");
        fs::write(&file, b"{}").unwrap();
        let result = resolve_path(Some(&file), "schema.json");
        assert_eq!(result, Some(file));
    }

    // write_output tests

    #[test]
    fn write_output_creates_file_with_content() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("output.txt");
        write_output(b"hello world", Some(file.clone())).unwrap();
        assert_eq!(fs::read(file).unwrap(), b"hello world");
    }

    #[test]
    fn write_output_creates_missing_parent_directories() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a/b/c/output.txt");
        write_output(b"content", Some(file.clone())).unwrap();
        assert_eq!(fs::read(file).unwrap(), b"content");
    }

    #[test]
    fn write_output_overwrites_existing_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("output.txt");
        fs::write(&file, b"old").unwrap();
        write_output(b"new", Some(file.clone())).unwrap();
        assert_eq!(fs::read(file).unwrap(), b"new");
    }
}
