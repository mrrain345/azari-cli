use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::receipt::ReceiptError;

pub mod build;
pub mod install;

use build::BuildArgs;
use install::InstallArgs;

/// Azari CLI
#[derive(Debug, Parser)]
#[command(name = "azari", version, about)]
pub struct Cli {
    /// Path to the receipt file (uses AZARI_RECEIPT env var if not provided)
    #[arg(short, long, value_name = "PATH", global = true)]
    pub receipt: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Resolves the receipt file path from the following sources, in order:
    ///
    /// 1. The `--receipt` / `-r` CLI flag
    /// 2. The `AZARI_RECEIPT` environment variable
    ///
    /// Returns [`ReceiptError::ReceiptNotFound`] if neither is set.
    pub fn receipt_path(&self) -> Result<PathBuf, ReceiptError> {
        if let Some(path) = &self.receipt {
            return Ok(path.clone());
        }

        if let Ok(val) = std::env::var("AZARI_RECEIPT") {
            return Ok(PathBuf::from(val));
        }

        Err(ReceiptError::ReceiptNotProvided)
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build the receipt
    Build(BuildArgs),
    /// Install the latest image onto a block device
    Install(InstallArgs),
}

impl Command {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        match self {
            Command::Build(args) => args.run(cli),
            Command::Install(args) => args.run(cli),
        }
    }
}
