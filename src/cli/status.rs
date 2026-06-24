use clap::Args;

use crate::builder::command::bootc_status;
use crate::recipe::RecipeError;

use super::Cli;

/// Show the status of the booted bootc system
#[derive(Debug, Args)]
pub struct StatusArgs {}

impl StatusArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), RecipeError> {
        bootc_status()
    }
}
