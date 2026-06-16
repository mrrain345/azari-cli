use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use clap::Args;

use crate::builder::Builder;
use crate::builder::command::{podman_build, podman_push};
use crate::builder::utils::user_tmp_dir;
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

    /// Do not use cached layers when building the image.
    #[arg(long)]
    pub no_cache: bool,

    /// Create and keep a build directory in the specified path.
    #[arg(long, value_name = "PATH")]
    pub build_dir: Option<PathBuf>,

    /// Generate the Containerfile but skip running `podman build`.
    #[arg(long)]
    pub dry: bool,
}

impl BuildArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        // TODO: Remove the need for `cli`, consume self

        handle_tmp_cleanup();

        let path = cli.receipt_path()?;
        let receipt = Receipt::from_file(&path)?;

        let mut builder =
            Builder::from_receipt(receipt, self.version.clone(), self.build_dir.clone())?;

        // Override the image name if specified via CLI flag.
        if let Some(image) = &self.image {
            builder.set_image(image.clone());
        }

        builder.add_trailer(!self.skip_rechunk);
        builder.write_containerfile()?;

        podman_build(&mut builder, self.dry, self.no_cache)?;

        if self.push && !self.dry {
            podman_push(builder.image()?, builder.version(), true)?;
        }

        Ok(())
    }
}

/// Clears the user temporary directory of all files and subdirectories.
fn clear_tmp_dir() -> std::io::Result<()> {
    let tmp_dir = user_tmp_dir();

    for entry in std::fs::read_dir(&tmp_dir)? {
        let path = entry?.path();
        set_permissions_recursively(&path)?;
        std::fs::remove_dir_all(path)?;
    }

    Ok(())
}

/// Recursively sets permissions to 700 for the given path and all its children.
fn set_permissions_recursively(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            set_permissions_recursively(&entry?.path())?;
        }
    }
    Ok(())
}

/// Sets up a `Ctrl-C` handler to ensure the temporary build directory is cleaned up on interrupt.
fn handle_tmp_cleanup() {
    ctrlc::set_handler(|| {
        clear_tmp_dir().unwrap_or_else(|_| {
            let path = user_tmp_dir();
            let path = path.display();
            eprintln!("Failed to clear temporary build directory: \"{path}\"");
        });
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");
}
