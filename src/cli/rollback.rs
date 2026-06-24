use clap::Args;

use crate::builder::command::bootc_rollback;
use crate::recipe::RecipeError;

use super::Cli;

#[derive(Debug, Args)]
pub struct RollbackArgs {}

impl RollbackArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), RecipeError> {
        bootc_rollback()
    }
}
