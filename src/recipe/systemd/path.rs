use crate::ini::{IniExtra, IniMulti};
use crate::recipe::systemd::unit::{InstallSection, SystemdUnit, UnitSection};
use serde::{Deserialize, Serialize};

/// Content for a `.path` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PathUnit {
    #[serde(skip_serializing)]
    pub enabled: bool,

    pub unit: Option<UnitSection>,
    pub path: Option<PathSection>,
    pub install: Option<InstallSection>,
}

/// `[Path]` section of a `.path` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PathSection {
    /// Trigger when a path appears.
    pub path_exists: IniMulti<String>,
    /// Trigger when a glob matches an existing path.
    pub path_exists_glob: IniMulti<String>,
    /// Trigger when a path's metadata changes.
    pub path_changed: IniMulti<String>,
    /// Trigger when a path's contents change.
    pub path_modified: IniMulti<String>,
    /// Less-common `[Path]` directives not listed above.
    #[serde(flatten)]
    pub extra: IniExtra,
}

impl SystemdUnit for PathUnit {
    fn unit_type() -> &'static str {
        "path"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn has_sections(&self) -> bool {
        self.unit.is_some() || self.path.is_some() || self.install.is_some()
    }
}
