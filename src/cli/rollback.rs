use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::bootc_rollback;

#[derive(Debug, Args)]
pub struct RollbackArgs {}

impl RollbackArgs {
    pub fn run(self) -> Result<(), BuildError> {
        bootc_rollback()
    }
}
