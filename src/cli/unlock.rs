use clap::Args;

use crate::builder::command::bootc_unlock;
use crate::receipt::ReceiptError;

use super::Cli;

/// Make /usr writable via a transient overlay
#[derive(Debug, Args)]
pub struct UnlockArgs {}

impl UnlockArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), ReceiptError> {
        bootc_unlock()
    }
}
