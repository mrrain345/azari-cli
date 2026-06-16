use clap::Args;

use crate::builder::command::podman_images;
use crate::receipt::ReceiptError;

use super::Cli;

/// List images in the user's isolated storage
#[derive(Debug, Args)]
pub struct ImagesArgs {}

impl ImagesArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), ReceiptError> {
        podman_images()
    }
}
