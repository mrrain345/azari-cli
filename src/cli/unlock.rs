use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::bootc_unlock;

#[derive(Debug, Args)]
pub struct UnlockArgs {}

impl UnlockArgs {
    pub fn run(self) -> Result<(), BuildError> {
        bootc_unlock()
    }
}
