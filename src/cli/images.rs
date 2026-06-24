use clap::Args;

use crate::builder::command::podman_images;
use crate::recipe::RecipeError;

use super::Cli;

/// List images in the user's isolated storage
#[derive(Debug, Args)]
pub struct ImagesArgs {}

impl ImagesArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), RecipeError> {
        podman_images()
    }
}
