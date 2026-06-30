use std::path::PathBuf;

use thiserror::Error;

use crate::recipe::{RecipeError, label, pathname};

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Config path not provided: use --config/-c <PATH> or set the AZARI_CONFIG env var")]
    ConfigNotProvided,

    #[error("Unsupported distro: {}", label(.0))]
    UnsupportedDistro(String),

    #[error("Distro not specified. Add {} field to your recipe.", label("distro"))]
    DistroNotSpecified,

    #[error(
        "Image name not specified. Add {} field to your recipe.",
        label("image")
    )]
    ImageNotSpecified,

    #[error("Target file {} already exists. Use --wipe to overwrite.", pathname(.0))]
    FileExistsWithoutWipe(PathBuf),

    #[error("{} failed with exit code {}", label(.0), label(&.1.to_string()))]
    CommandFailed(String, i32),

    #[error("Command not found: {}. Please install it before proceeding.", label(.0))]
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
