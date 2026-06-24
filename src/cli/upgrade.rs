use clap::Args;

use crate::builder::command::bootc_upgrade;
use crate::recipe::RecipeError;

use super::Cli;

#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Upgrade to a specific version tag (e.g. `1.0.0`, `latest`)
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,
}

impl UpgradeArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), RecipeError> {
        bootc_upgrade(self.version.as_deref())
    }
}
