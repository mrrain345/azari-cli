use std::path::PathBuf;

use clap::Args;

use crate::builder::command::{podman_build, podman_push};
use crate::builder::{BuildDir, Builder};
use crate::receipt::{Receipt, ReceiptError};

use super::Cli;

/// Build the receipt
#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Version tag for the image (e.g. `1.0.0`).
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Push the image to its registry after a successful build.
    #[arg(short = 'p', long)]
    pub push: bool,

    /// Override the image name from the receipt (e.g. `ghcr.io/user/image`).
    #[arg(short = 'i', long, value_name = "IMAGE")]
    pub image: Option<String>,

    /// Skip rechunking with chunkah.
    #[arg(long)]
    pub skip_rechunk: bool,

    /// Path to the build directory.
    #[arg(short = 'b', long, value_name = "PATH")]
    pub build_dir: Option<PathBuf>,

    /// Generate the Containerfile but skip running `podman build`.
    #[arg(long)]
    pub dry: bool,
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

        builder.add_trailer(!self.skip_rechunk);
        builder.write_containerfile()?;

        podman_build(&mut builder, self.dry)?;

        if self.push && !self.dry {
            podman_push(builder.image()?, builder.version(), true)?;
        }

        Ok(())
    }
}
