use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::podman_images;

#[derive(Debug, Args)]
pub struct ImagesArgs {}

impl ImagesArgs {
    pub fn run(self) -> Result<(), BuildError> {
        podman_images()
    }
}
