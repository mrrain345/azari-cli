use clap::Args;

use crate::builder::command::podman_clear;
use crate::builder::utils::remove_cache_dir;
use crate::recipe::{Recipe, RecipeError, RecipeField};

use super::Cli;

#[derive(Debug, Args)]
pub struct ClearArgs {
    /// Remove the entire azari cache directory with all images and build artifacts
    #[arg(long)]
    pub all: bool,
}

impl ClearArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), RecipeError> {
        if self.all {
            podman_clear(None)?;
            remove_cache_dir()?;
            return Ok(());
        }

        let path = cli.config_path()?;
        let recipe = Recipe::from_file(&path)?;
        let image = recipe
            .image
            .value()?
            .ok_or(RecipeError::ImageNotSpecified)?;

        podman_clear(Some(&image))
    }
}
