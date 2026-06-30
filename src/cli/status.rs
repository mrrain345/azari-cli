use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::bootc_status;

use super::Cli;

#[derive(Debug, Args)]
pub struct StatusArgs {}

impl StatusArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), BuildError> {
        bootc_status()
    }
}
