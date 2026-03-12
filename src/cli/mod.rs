use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::receipt::error::ReceiptError;

pub mod build;

use build::BuildArgs;

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
}

impl Command {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        match self {
            Command::Build(args) => args.run(cli),
        }
    }
}
