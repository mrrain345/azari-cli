use crate::ini::{IniAny, IniExtra};
use crate::recipe::systemd::unit::{InstallSection, SystemdUnit, UnitSection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Content for a `.socket` unit file.
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct SocketUnit {
    #[serde(skip_serializing)]
    pub enabled: bool,

    pub unit: Option<UnitSection>,
    pub socket: Option<SocketSection>,
    pub install: Option<InstallSection>,
}

/// `[Socket]` section of a `.socket` unit file.
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct SocketSection {
    /// Filesystem path or address to listen on.
    pub listen_stream: Option<String>,
    /// Octal permission mode for the socket node (e.g. `0660`).
    pub socket_mode: Option<IniAny>,
    /// User that owns the socket node.
    pub socket_user: Option<String>,
    /// Group that owns the socket node.
    pub socket_group: Option<String>,
    /// Less-common `[Socket]` directives not listed above.
    #[serde(flatten)]
    pub extra: IniExtra,
}

impl SystemdUnit for SocketUnit {
    fn unit_type() -> &'static str {
        "socket"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn has_sections(&self) -> bool {
        self.unit.is_some() || self.socket.is_some() || self.install.is_some()
    }
}
