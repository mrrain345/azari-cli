use std::path::Path;

use clap::Args;

use crate::builder::command::{fallocate, podman_install, podman_prune_old_root_images};
use crate::receipt::{Receipt, ReceiptError, ReceiptField};

use super::Cli;

/// Install the latest image onto a block device
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Target block device (e.g. `/dev/sda`) or file path
    #[arg(value_name = "DEVICE")]
    pub device: String,

    /// Image version tag to install (e.g. `1.0.0`). Defaults to `latest`.
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Wipe the target device before installing
    #[arg(long)]
    pub wipe: bool,

    /// Image size when installing to a file (e.g. `20G`). Defaults to `16G`.
    #[arg(long, value_name = "SIZE", default_value = "16G")]
    pub size: String,
}

impl InstallArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        let path = cli.receipt_path()?;
        let receipt = Receipt::from_file(&path)?;
        let image = receipt
            .image
            .value()?
            .ok_or(ReceiptError::ImageNotSpecified)?;

        let version = self.version.as_deref().unwrap_or("latest");

        // Determine whether this is a "install to file" operation.
        // Condition: path is outside /dev AND (is a regular file OR does not exist).
        let device_path = Path::new(&self.device);
        let via_loopback =
            !device_path.starts_with("/dev") && (device_path.is_file() || !device_path.exists());

        if via_loopback {
            if device_path.exists() {
                if !self.wipe {
                    return Err(ReceiptError::FileExistsWithoutWipe(
                        device_path.to_path_buf(),
                    ));
                }
            }
            println!(
                "Installing image {image}:{version} to file {} ({})",
                self.device, self.size
            );
            fallocate(device_path, &self.size)?;
        } else {
            println!(
                "Installing image {image}:{version} onto device {}",
                self.device
            );
        }

        let install_result = podman_install(&image, version, &self.device, self.wipe, via_loopback);

        if install_result.is_ok() {
            println!("Pruning old images from root storage…");

            // Remove azari-managed images from root storage that are older than 30 days
            podman_prune_old_root_images();
        }

        println!("Done.");

        install_result
    }
}
