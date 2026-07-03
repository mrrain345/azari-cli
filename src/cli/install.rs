use std::path::{Path, PathBuf};

use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::{fallocate, podman_install};
use crate::recipe::{Recipe, RecipeField};

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Target block device (e.g. `/dev/sda`) or file path
    #[arg(value_name = "DEVICE")]
    pub device: String,

    /// Container image to install (e.g. `ghcr.io/user/image`)
    #[arg(long, value_name = "IMAGE")]
    pub image: Option<String>,

    /// Image version tag to install (e.g. `1.0.0`). Defaults to `latest`
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Wipe the target device before installing
    #[arg(long)]
    pub wipe: bool,

    /// Image size when installing to a file. Defaults to `16G`
    #[arg(long, value_name = "SIZE", default_value = "16G")]
    pub size: String,
}

impl InstallArgs {
    pub fn run(self, config: Option<PathBuf>) -> Result<(), BuildError> {
        let image = match self.image {
            Some(image) => image,
            None => {
                let path = config.ok_or(BuildError::ConfigNotProvided)?;
                let recipe = Recipe::from_file(&path)?;
                recipe.image.value()?.ok_or(BuildError::ImageNotSpecified)?
            }
        };

        let version = self.version.as_deref().unwrap_or("latest");

        // Determine whether this is a "install to file" operation.
        // Condition: path is outside /dev AND (is a regular file OR does not exist).
        let device_path = Path::new(&self.device);
        let via_loopback =
            !device_path.starts_with("/dev") && (device_path.is_file() || !device_path.exists());

        if via_loopback {
            if device_path.exists() && !self.wipe {
                return Err(BuildError::FileExistsWithoutWipe(device_path.to_owned()));
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

        podman_install(&image, version, &self.device, self.wipe, via_loopback)
    }
}
