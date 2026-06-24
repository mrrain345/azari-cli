use crate::ini::{IniExtra, IniMulti};
use crate::recipe::systemd::unit::{InstallSection, SystemdUnit, UnitSection};
use serde::{Deserialize, Serialize};

/// Content for a `.timer` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct TimerUnit {
    #[serde(skip_serializing)]
    pub enabled: bool,

    pub unit: Option<UnitSection>,
    pub timer: Option<TimerSection>,
    pub install: Option<InstallSection>,
}

/// `[Timer]` section of a `.timer` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct TimerSection {
    /// Calendar expression for trigger times, e.g. `daily`.
    pub on_calendar: IniMulti<String>,
    /// Delay relative to system boot.
    pub on_boot_sec: Option<serde_value::Value>,
    /// Delay relative to timer activation.
    pub on_active_sec: Option<serde_value::Value>,
    /// Delay relative to timer start.
    pub on_start_sec: Option<serde_value::Value>,
    /// Delay relative to the last activation of the linked unit.
    pub on_unit_active_sec: Option<serde_value::Value>,
    /// Delay relative to the last deactivation of the linked unit.
    pub on_unit_inactive_sec: Option<serde_value::Value>,
    /// Keep schedule across downtime when true.
    pub persistent: Option<bool>,
    /// Add a random delay to spread load.
    pub randomized_delay_sec: Option<serde_value::Value>,
    /// Less-common `[Timer]` directives not listed above.
    #[serde(flatten)]
    pub extra: IniExtra,
}

impl SystemdUnit for TimerUnit {
    fn unit_type() -> &'static str {
        "timer"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn has_sections(&self) -> bool {
        self.unit.is_some() || self.timer.is_some() || self.install.is_some()
    }
}
