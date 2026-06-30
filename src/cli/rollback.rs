use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::bootc_rollback;

use super::Cli;

#[derive(Debug, Args)]
pub struct RollbackArgs {}

impl RollbackArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), BuildError> {
        bootc_rollback()
    }
}
