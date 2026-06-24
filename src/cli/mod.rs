use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::recipe::RecipeError;

pub mod build;
pub mod clear;
pub mod images;
pub mod install;
pub mod push;
pub mod rollback;
pub mod status;
pub mod switch;
pub mod unlock;
pub mod upgrade;

use build::BuildArgs;
use clear::ClearArgs;
use images::ImagesArgs;
use install::InstallArgs;
use push::PushArgs;
use rollback::RollbackArgs;
use status::StatusArgs;
use switch::SwitchArgs;
use unlock::UnlockArgs;
use upgrade::UpgradeArgs;

/// Azari CLI
#[derive(Debug, Parser)]
#[command(name = "azari", version, about)]
pub struct Cli {
    /// Path to the recipe file (uses AZARI_RECIPE env var if not provided)
    #[arg(short, long, value_name = "PATH", global = true)]
    pub recipe: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Resolves the recipe file path from the following sources, in order:
    ///
    /// 1. The `--recipe` / `-r` CLI flag
    /// 2. The `AZARI_RECIPE` environment variable
    ///
    /// Returns [`RecipeError::RecipeNotProvided`] if neither is set.
    pub fn recipe_path(&self) -> Result<PathBuf, RecipeError> {
        if let Some(path) = &self.recipe {
            return Ok(path.clone());
        }

        match std::env::var_os("AZARI_RECIPE") {
            Some(val) if !val.is_empty() => Ok(PathBuf::from(val)),
            _ => Err(RecipeError::RecipeNotProvided),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show the status of the booted system
    Status(StatusArgs),
    /// Make /usr writable via a transient overlay (all changes will be lost on reboot)
    Unlock(UnlockArgs),

    /// Upgrade the currently installed system
    Upgrade(UpgradeArgs),
    /// Switch the specific version
    Switch(SwitchArgs),
    /// Rollback to the previous deployment
    Rollback(RollbackArgs),

    /// Build the recipe
    Build(BuildArgs),
    /// Push a previously built image to its registry
    Push(PushArgs),

    /// Install the latest image onto a block device
    Install(InstallArgs),
    /// List all locally stored images
    Images(ImagesArgs),
    /// Prune all locally stored images except the current image:latest
    Clear(ClearArgs),
}

impl Command {
    pub fn run(&self, cli: &Cli) -> Result<(), RecipeError> {
        match self {
            Command::Status(args) => args.run(cli),
            Command::Unlock(args) => args.run(cli),
            Command::Upgrade(args) => args.run(cli),
            Command::Switch(args) => args.run(cli),
            Command::Rollback(args) => args.run(cli),
            Command::Build(args) => args.run(cli),
            Command::Push(args) => args.run(cli),
            Command::Install(args) => args.run(cli),
            Command::Images(args) => args.run(cli),
            Command::Clear(args) => args.run(cli),
        }
    }
}
