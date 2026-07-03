use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::bootc_status;

#[derive(Debug, Args)]
pub struct StatusArgs {}

impl StatusArgs {
    pub fn run(self) -> Result<(), BuildError> {
        bootc_status()
    }
}
