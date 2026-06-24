use clap::Args;

use crate::builder::command::{bootc_switch, podman_transfer};
use crate::recipe::{Recipe, RecipeError, RecipeField};

use super::Cli;

/// Switch the bootc image to a specific version
#[derive(Debug, Args)]
pub struct SwitchArgs {
    /// Target image version tag to switch to (e.g. `1.2.0`)
    #[arg(value_name = "VERSION")]
    pub version: String,

    /// Switch to a locally built image.
    #[arg(long)]
    pub local: bool,
}

impl SwitchArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), RecipeError> {
        let path = cli.recipe_path()?;
        let recipe = Recipe::from_file(&path)?;
        let image = recipe
            .image
            .value()?
            .ok_or(RecipeError::ImageNotSpecified)?;

        if self.local {
            println!(
                "Transferring local image {image}:{} to root storage…",
                self.version
            );
            podman_transfer(&image, &self.version)?;
        }

        bootc_switch(&image, &self.version, self.local)
    }
}
