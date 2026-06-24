use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RecipeError {
    #[error("Recipe path not provided: use --recipe/-r <PATH> or set the AZARI_RECIPE env var")]
    RecipeNotProvided,

    #[error("Invalid recipe path: unable to resolve parent directory for `{0}`")]
    InvalidRecipePath(PathBuf),

    #[error("Field `{}` has conflicting values in:\n  - {}", .field.as_deref().unwrap_or("<unknown>"), .paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n  - "))]
    FieldConflict {
        field: Option<String>,
        paths: Vec<PathBuf>,
    },

    #[error("{}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))]
    Aggregate(Vec<RecipeError>),

    #[error("Failed to read recipe file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse recipe file: {0}")]
    Parse(#[from] serde_saphyr::Error),

    #[error("Unsupported distro: {0}")]
    UnsupportedDistro(String),

    #[error("Distro not specified. Add a \"distro\" field to your recipe.")]
    DistroNotSpecified,

    #[error("Image name not specified. Add an \"image\" field to your recipe.")]
    ImageNotSpecified,

    #[error("Target file {0} already exists. Use --wipe to overwrite.")]
    FileExistsWithoutWipe(PathBuf),

    #[error("`{0}` failed with exit code {1}")]
    CommandFailed(String, i32),

    #[error("Command not found: `{0}`. Please install it before proceeding.")]
    CommandNotFound(String),
}
