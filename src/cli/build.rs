use clap::Args;

use crate::receipt::{error::ReceiptError, receipt::Receipt};

use super::Cli;

/// Build the receipt
#[derive(Debug, Args)]
pub struct BuildArgs {
    // Future build-specific arguments go here
}

impl BuildArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        let path = cli.receipt_path()?;
        let _receipt = Receipt::from_file(&path)?;
        println!("Receipt loaded from {}", path.display());
        Ok(())
    }
}
