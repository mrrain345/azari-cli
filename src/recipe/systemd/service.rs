use crate::ini::{IniAny, IniExtra, IniMulti};
use crate::recipe::systemd::unit::{InstallSection, SystemdUnit, UnitSection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Content for a `.service` unit file.
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServiceUnit {
    #[serde(skip_serializing)]
    pub enabled: bool,

    pub unit: Option<UnitSection>,
    pub service: Option<ServiceSection>,
    pub install: Option<InstallSection>,
}

/// `[Service]` section of a `.service` unit file.
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServiceSection {
    /// Activation type: `simple`, `oneshot`, `notify`, `forking`, etc.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Commands run before the main process.
    pub exec_start_pre: IniMulti<String>,
    /// Main process command(s).
    pub exec_start: IniMulti<String>,
    /// Command run on `systemctl reload`.
    pub exec_reload: Option<String>,
    /// Restart policy: `always`, `on-failure`, `no`, etc.
    pub restart: Option<String>,
    /// Seconds to wait between restarts. Accepts an integer or a string like `5s`.
    pub restart_sec: Option<IniAny>,
    /// Seconds to wait for startup. Accepts an integer or a string like `30s`.
    pub timeout_start_sec: Option<IniAny>,
    /// `KEY=VALUE` environment variables.
    pub environment: IniMulti<String>,
    /// Less-common `[Service]` directives not listed above.
    #[serde(flatten)]
    pub extra: IniExtra,
}

impl SystemdUnit for ServiceUnit {
    fn unit_type() -> &'static str {
        "service"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn has_sections(&self) -> bool {
        self.unit.is_some() || self.service.is_some() || self.install.is_some()
    }
}
