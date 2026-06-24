use clap::Args;

use crate::builder::command::bootc_unlock;
use crate::recipe::RecipeError;

use super::Cli;

#[derive(Debug, Args)]
pub struct UnlockArgs {}

impl UnlockArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), RecipeError> {
        bootc_unlock()
    }
}
