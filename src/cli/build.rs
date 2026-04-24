use std::path::PathBuf;

use clap::Args;

use crate::builder::command::{podman_build, podman_prune, podman_push};

use crate::builder::{BuildDir, Builder};
use crate::receipt::{Receipt, ReceiptError};

use super::Cli;

/// Build the receipt
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Path to the build directory. Must be empty or non-existent.
    /// When omitted a temporary directory is used and cleaned up on exit.
    #[arg(short = 'b', long, value_name = "PATH")]
    pub build_dir: Option<PathBuf>,

    /// Version tag for the image (e.g. `1.0.0`). The image is always tagged
    /// as `<image>:latest`; when this flag is set it is also tagged as
    /// `<image>:<version>`. The image name comes from the `image` field in
    /// the receipt.
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Override the image name from the receipt (e.g. `docker.io/myorg/myimage`).
    /// Takes precedence over the `image` field in the receipt.
    #[arg(short = 'i', long, value_name = "IMAGE")]
    pub image: Option<String>,

    /// Generate the Containerfile but skip running `podman build`.
    #[arg(long)]
    pub dry: bool,

    /// Push the image to its registry after a successful build.
    #[arg(short = 'p', long)]
    pub push: bool,
}

impl BuildArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        let path = cli.receipt_path()?;
        let receipt = Receipt::from_file(&path)?;

        let build_dir = match &self.build_dir {
            Some(path) => BuildDir::persistent(path.clone())?,
            None => BuildDir::temp()?,
        };

        let mut builder = Builder::from_receipt(receipt, build_dir, self.version.clone())?;
        if let Some(image) = &self.image {
            builder.set_image(image.clone());
        }

        let image = builder
            .image()
            .ok_or(ReceiptError::ImageNotSpecified)?
            .to_string();

        builder.write_containerfile()?;

        if !self.dry {
            podman_build(
                builder.build_dir(),
                &image,
                builder.version(),
                builder.name(),
            )?;
            // Prune dangling layers left by the previous build of this image.
            podman_prune();

            if self.push {
                podman_push(&image, builder.version(), true)?;
            }
        }

        Ok(())
    }
}
