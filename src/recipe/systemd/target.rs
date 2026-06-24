use crate::recipe::systemd::unit::{InstallSection, SystemdUnit, UnitSection};
use serde::{Deserialize, Serialize};

/// Content for a `.target` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct TargetUnit {
    #[serde(skip_serializing)]
    pub enabled: bool,

    pub unit: Option<UnitSection>,
    pub install: Option<InstallSection>,
}

impl SystemdUnit for TargetUnit {
    fn unit_type() -> &'static str {
        "target"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn has_sections(&self) -> bool {
        self.unit.is_some() || self.install.is_some()
    }
}
