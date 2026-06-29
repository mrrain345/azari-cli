use crate::builder::Builder;
use crate::recipe::error::RecipeError;
use crate::recipe::systemd::SystemdUnit;
use crate::recipe::systemd::path::PathUnit;
use crate::recipe::systemd::service::ServiceUnit;
use crate::recipe::systemd::socket::SocketUnit;
use crate::recipe::systemd::target::TargetUnit;
use crate::recipe::systemd::timer::TimerUnit;
use schemars::JsonSchema;
use serde::Deserialize;

/// # Systemd Entry
/// Name of the systemd unit
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct SystemdEntry {
    /// # User
    /// Installs the unit as a user service.
    pub user: bool,
    pub service: ServiceUnit,
    pub socket: SocketUnit,
    pub timer: TimerUnit,
    pub path: PathUnit,
    pub target: TargetUnit,
}

impl SystemdEntry {
    pub fn build(self, builder: &mut Builder, name: &str) -> Result<(), RecipeError> {
        let is_user = self.user;

        self.service.build(builder, name, is_user)?;
        self.socket.build(builder, name, is_user)?;
        self.timer.build(builder, name, is_user)?;
        self.path.build(builder, name, is_user)?;
        self.target.build(builder, name, is_user)?;

        Ok(())
    }
}
