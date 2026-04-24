use clap::Args;

use crate::builder::command::bootc_upgrade;
use crate::receipt::ReceiptError;

use super::Cli;

/// Upgrade the currently installed bootc system
#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Target image version tag for upgrade. Passed to `bootc upgrade --tag`.
    #[arg(short = 'v', long, value_name = "VERSION")]
    pub version: Option<String>,
}

impl UpgradeArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), ReceiptError> {
        bootc_upgrade(self.version.as_deref())
    }
}
