use clap::Args;

use crate::builder::command::bootc_unlock;
use crate::recipe::RecipeError;

use super::Cli;

/// Make /usr writable via a transient overlay
#[derive(Debug, Args)]
pub struct UnlockArgs {}

impl UnlockArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), RecipeError> {
        bootc_unlock()
    }
}
