use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::bootc_unlock;

use super::Cli;

#[derive(Debug, Args)]
pub struct UnlockArgs {}

impl UnlockArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), BuildError> {
        bootc_unlock()
    }
}
