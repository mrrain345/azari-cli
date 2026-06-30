use std::path::PathBuf;

use thiserror::Error;

use crate::recipe::RecipeError;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Config path not provided: use --config/-c <PATH> or set the AZARI_CONFIG env var")]
    ConfigNotProvided,

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

    #[error(transparent)]
    Recipe(Box<RecipeError>),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<RecipeError> for BuildError {
    fn from(err: RecipeError) -> Self {
        BuildError::Recipe(Box::new(err))
    }
}
