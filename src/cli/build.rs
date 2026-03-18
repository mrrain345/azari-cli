use std::path::PathBuf;

use clap::Args;

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
}

impl BuildArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        let path = cli.receipt_path()?;
        let receipt = Receipt::from_file(&path)?;

        let build_dir = match &self.build_dir {
            Some(path) => BuildDir::persistent(path.clone())?,
            None => BuildDir::temp()?,
        };

        let builder = Builder::from_receipt(receipt, build_dir)?;
        builder.write_containerfile()?;

        Ok(())
    }
}
