use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::podman_images;

use super::Cli;

#[derive(Debug, Args)]
pub struct ImagesArgs {}

impl ImagesArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), BuildError> {
        podman_images()
    }
}
