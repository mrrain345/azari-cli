use clap::Args;

use crate::builder::command::{bootc_switch, podman_transfer};
use crate::recipe::{Recipe, RecipeError, RecipeField};

use super::Cli;

#[derive(Debug, Args)]
pub struct SwitchArgs {
    /// Version tag to switch to (e.g. `1.0.0`, `latest`)
    #[arg(value_name = "VERSION")]
    pub version: String,

    /// Switch to a locally built image
    #[arg(long)]
    pub local: bool,
}

impl SwitchArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), RecipeError> {
        let path = cli.config_path()?;
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
