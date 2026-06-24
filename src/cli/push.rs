use clap::Args;

use crate::builder::command::podman_push;
use crate::recipe::{Recipe, RecipeError, RecipeField};

use super::Cli;

/// Push a previously built image to its registry
#[derive(Debug, Args)]
pub struct PushArgs {
    /// Version tag to push (e.g. `1.0.0`).
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Override the image name from the recipe (e.g. `docker.io/myorg/myimage`).
    /// Takes precedence over the `image` field in the recipe.
    #[arg(short = 'i', long, value_name = "IMAGE")]
    pub image: Option<String>,

    /// Skip pushing the `latest` tag.
    #[arg(short = 'L', long)]
    pub no_latest: bool,
}

impl PushArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), RecipeError> {
        let image = match &self.image {
            Some(image) => image.clone(),
            None => {
                let path = cli.recipe_path()?;
                let recipe = Recipe::from_file(&path)?;
                recipe
                    .image
                    .value()?
                    .ok_or(RecipeError::ImageNotSpecified)?
            }
        };

        podman_push(&image, self.version.as_deref(), !self.no_latest)?;

        Ok(())
    }
}
