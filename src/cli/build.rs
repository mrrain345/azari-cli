use clap::Args;

use crate::builder::Builder;
use crate::receipt::{Receipt, ReceiptError};

use super::Cli;

/// Build the receipt
#[derive(Debug, Args)]
pub struct BuildArgs {
    // Future build-specific arguments go here
}

impl BuildArgs {
    pub fn run(&self, cli: &Cli) -> Result<(), ReceiptError> {
        let path = cli.receipt_path()?;
        let receipt = Receipt::from_file(&path)?;
        let builder = Builder::from_receipt(receipt)?;

        let output = builder.to_containerfile();
        println!("{output}");

        Ok(())
    }
}
