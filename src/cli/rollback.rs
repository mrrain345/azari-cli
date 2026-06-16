use clap::Args;

use crate::builder::command::bootc_rollback;
use crate::receipt::ReceiptError;

use super::Cli;

/// Rollback to the previous bootc deployment
#[derive(Debug, Args)]
pub struct RollbackArgs {}

impl RollbackArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), ReceiptError> {
        bootc_rollback()
    }
}
