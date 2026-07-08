use std::path::PathBuf;

use clap::Args;
use colored::Colorize;

use crate::builder::command::{podman_build, podman_push};
use crate::builder::utils::{clear_tmp_dir, user_tmp_dir};
use crate::builder::{BuildError, Builder, BuilderOptions};
use crate::recipe::Recipe;

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Version tag for the image (e.g. `1.0.0`)
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,

    /// Push the image to its registry after a successful build
    #[arg(short = 'p', long)]
    pub push: bool,

    /// Override the image name from the config (e.g. `ghcr.io/user/image`)
    #[arg(short = 'i', long, value_name = "IMAGE")]
    pub image: Option<String>,

    /// Skip rechunking the image
    #[arg(long)]
    pub skip_rechunk: bool,

    /// Do not use cached layers when building the image
    #[arg(long)]
    pub no_cache: bool,

    /// Create and keep a build directory in the specified path
    #[arg(long, value_name = "PATH")]
    pub build_dir: Option<PathBuf>,

    /// Generate the Containerfile but skip building the image
    #[arg(long)]
    pub dry: bool,
}

impl BuildArgs {
    pub fn run(self, config: Option<PathBuf>) -> Result<(), BuildError> {
        handle_tmp_cleanup();

        let path = config.ok_or(BuildError::ConfigNotProvided)?;
        let recipe = Recipe::from_file(&path)?;

        let mut builder = Builder::from_recipe_with(
            recipe,
            BuilderOptions {
                version: self.version,
                build_dir: self.build_dir,
                output_image: self.image,
            },
        )?;

        builder.add_trailer(!self.skip_rechunk);
        builder.write_containerfile()?;

        podman_build(&mut builder, self.dry, self.no_cache)?;

        if self.push && !self.dry {
            podman_push(
                builder.meta().output_image()?,
                builder.meta().version(),
                true,
            )?;
        }

        Ok(())
    }
}

/// Sets up a `Ctrl-C` handler to ensure the temporary build directory is cleaned up on interrupt.
fn handle_tmp_cleanup() {
    ctrlc::set_handler(|| {
        clear_tmp_dir().unwrap_or_else(|_| {
            let path = user_tmp_dir();
            eprintln!(
                "{}",
                format!(
                    "Failed to clear temporary build directory: `{}`",
                    path.display().to_string().yellow().italic()
                )
                .red()
            );
        });
        std::process::exit(130);
    })
    .expect("Error setting Ctrl-C handler");
}
