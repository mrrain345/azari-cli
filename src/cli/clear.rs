use clap::Args;

use crate::builder::command::podman_clear;
use crate::builder::utils::remove_cache_dir;
use crate::receipt::{Receipt, ReceiptError, ReceiptField};

use super::Cli;

/// Prune all images from user storage except the current image:latest
#[derive(Debug, Args)]
pub struct ClearArgs {
    /// Remove the entire azari cache directory with all images and build artifacts.
    #[arg(long)]
    pub all: bool,
}

impl ClearArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        if self.all {
            podman_clear(None)?;
            remove_cache_dir()?;
            return Ok(());
        }

        let path = cli.receipt_path()?;
        let receipt = Receipt::from_file(&path)?;
        let image = receipt
            .image
            .value()?
            .ok_or(ReceiptError::ImageNotSpecified)?;

        podman_clear(Some(&image))
    }
}
