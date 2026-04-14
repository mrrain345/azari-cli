use clap::Args;

use crate::builder::command::podman_push;
use crate::receipt::{Receipt, ReceiptError, ReceiptField};

use super::Cli;

/// Push a previously built image to its registry
#[derive(Debug, Args)]
pub struct PushArgs {
    /// Version tag to push (e.g. `1.0.0`).
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Override the image name from the receipt (e.g. `docker.io/myorg/myimage`).
    /// Takes precedence over the `image` field in the receipt.
    #[arg(short = 'i', long, value_name = "IMAGE")]
    pub image: Option<String>,

    /// Skip pushing the `latest` tag.
    #[arg(short = 'L', long)]
    pub no_latest: bool,
}

impl PushArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        let image = if let Some(image) = &self.image {
            image.clone()
        } else {
            let path = cli.receipt_path()?;
            let receipt = Receipt::from_file(&path)?;
            receipt
                .image
                .value()?
                .ok_or(ReceiptError::ImageNotSpecified)?
        };

        podman_push(&image, self.version.as_deref(), !self.no_latest)?;

        Ok(())
    }
}
