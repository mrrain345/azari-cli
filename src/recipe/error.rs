use std::path::{Path, PathBuf};

use colored::Colorize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecipeError {
    #[error("Invalid config path: unable to resolve parent directory for {}", pathname(.0))]
    InvalidConfigPath(PathBuf),

    #[error("Error in the {} field in {}:\n    {message}", label(.field), pathname(.path))]
    FieldError {
        path: PathBuf,
        field: String,
        message: String,
    },

    #[error("Field {} has conflicting values in:\n  - {}", label(.field.as_deref().unwrap_or("<unknown>")), .paths.iter().map(|p| pathname(p)).collect::<Vec<_>>().join("\n  - "))]
    FieldConflict {
        field: Option<String>,
        paths: Vec<PathBuf>,
    },

    #[error("{}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))]
    Aggregate(Vec<RecipeError>),

    #[error("Failed to read {}:\n    {source}", pathname(.path))]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse {}:\n    {source}", pathname(.path))]
    Parse {
        path: PathBuf,
        #[source]
        source: Box<serde_saphyr::Error>,
    },
}

pub fn pathname(path: &Path) -> String {
    format!("`{}`", path.display().to_string().yellow().italic())
}

pub fn label(label: &str) -> String {
    format!("{}", label.yellow().bold())
}
