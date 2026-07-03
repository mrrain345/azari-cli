use std::path::PathBuf;

use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::podman_clear;
use crate::builder::utils::remove_cache_dir;
use crate::recipe::{Recipe, RecipeField};

#[derive(Debug, Args)]
pub struct ClearArgs {
    /// Remove the entire azari cache directory with all images and build artifacts
    #[arg(long)]
    pub all: bool,
}

impl ClearArgs {
    pub fn run(self, config: Option<PathBuf>) -> Result<(), BuildError> {
        if self.all {
            podman_clear(None)?;
            remove_cache_dir()?;
            return Ok(());
        }

        let path = config.ok_or(BuildError::ConfigNotProvided)?;
        let recipe = Recipe::from_file(&path)?;
        let image = recipe.image.value()?.ok_or(BuildError::ImageNotSpecified)?;

        podman_clear(Some(&image))
    }
}
