use crate::builder::Builder;
use crate::recipe::error::RecipeError;
use crate::recipe::systemd::SystemdUnit;
use crate::recipe::systemd::path::PathUnit;
use crate::recipe::systemd::service::ServiceUnit;
use crate::recipe::systemd::socket::SocketUnit;
use crate::recipe::systemd::target::TargetUnit;
use crate::recipe::systemd::timer::TimerUnit;
use serde::Deserialize;

/// A single entry in the `systemd` field.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SystemdEntry {
    /// Installs the unit as a user service.
    pub user: bool,
    /// Content of the `.service` unit file.
    pub service: ServiceUnit,
    /// Content of the `.socket` unit file.
    pub socket: SocketUnit,
    /// Content of the `.timer` unit file.
    pub timer: TimerUnit,
    /// Content of the `.path` unit file.
    pub path: PathUnit,
    /// Content of the `.target` unit file.
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
