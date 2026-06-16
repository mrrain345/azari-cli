use clap::Args;

use crate::builder::command::podman_clear;
use crate::receipt::{Receipt, ReceiptError, ReceiptField};

use super::Cli;

/// Prune all images from user storage except the current image:latest
#[derive(Debug, Args)]
pub struct ClearArgs {}

impl ClearArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        let path = cli.receipt_path()?;
        let receipt = Receipt::from_file(&path)?;
        let image = receipt
            .image
            .value()?
            .ok_or(ReceiptError::ImageNotSpecified)?;

        podman_clear(&image)
    }
}
