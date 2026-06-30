use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecipeError {
    #[error("Invalid config path: unable to resolve parent directory for `{0}`")]
    InvalidConfigPath(PathBuf),

    #[error("Error in field `{field}` in `{path}`:\n    {message}")]
    FieldError {
        path: PathBuf,
        field: String,
        message: String,
    },

    #[error("Field `{}` has conflicting values in:\n  - {}", .field.as_deref().unwrap_or("<unknown>"), .paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n  - "))]
    FieldConflict {
        field: Option<String>,
        paths: Vec<PathBuf>,
    },

    #[error("{}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))]
    Aggregate(Vec<RecipeError>),

    #[error("Failed to read `{path}`:\n    {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse `{path}`:\n    {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: Box<serde_saphyr::Error>,
    },
}
