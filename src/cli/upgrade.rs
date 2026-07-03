use clap::Args;

use crate::builder::BuildError;
use crate::builder::command::bootc_upgrade;

#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Upgrade to a specific version tag (e.g. `1.0.0`, `latest`)
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,
}

impl UpgradeArgs {
    pub fn run(self) -> Result<(), BuildError> {
        bootc_upgrade(self.version.as_deref())
    }
}
