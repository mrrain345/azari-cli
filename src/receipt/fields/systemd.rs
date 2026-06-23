use merge::Merge;
use serde::{Deserialize, Serialize};

use crate::builder::{Build, Builder};
use crate::ini::{IniExtra, IniMulti};
use crate::receipt::ReceiptAlt;
use crate::receipt::error::ReceiptError;
use crate::receipt::field::{ReceiptField, rename_field_error};
use crate::receipt::fields::files::target_to_filename;
use crate::receipt::map::ReceiptMap;

// ---- Top-level field ----

/// Field for the `systemd` key.
///
/// Accepts either a flat list of service names to enable (simple form) or a
/// map from names to full unit-file descriptors (complex form).
///
/// **Simple form** — just enable pre-installed services:
/// ```yaml
/// systemd:
///   - NetworkManager
///   - cups
/// ```
///
/// **Complex form** — define unit files inline:
/// ```yaml
/// systemd:
///   my-service:
///     service:
///       enabled: true
///       unit:
///         description: My Service
///         after: network.target
///       service:
///         type: oneshot
///         exec-start: /usr/bin/my-service
///       install:
///         wanted-by: multi-user.target
/// ```
#[derive(Debug, Default, Deserialize, Merge)]
#[serde(transparent)]
pub struct SystemdField(ReceiptAlt<Vec<String>, ReceiptMap<String, SystemdEntry>>);

// ---- From impl for simple-form deserialization ----

/// Converts a flat list of service names into a `ReceiptMap` where each
/// service is enabled with no explicit unit file.
impl From<Vec<String>> for ReceiptMap<String, SystemdEntry> {
    fn from(names: Vec<String>) -> Self {
        ReceiptMap::new(
            names
                .into_iter()
                .map(|name| {
                    (
                        name,
                        SystemdEntry {
                            service: Some(ServiceUnit {
                                enabled: Some(true),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        )
    }
}

impl ReceiptField for SystemdField {
    type Value = Vec<(String, SystemdEntry)>;

    fn name() -> Option<&'static str> {
        Some("systemd")
    }

    fn value(self) -> Result<Self::Value, ReceiptError> {
        self.0.value()
    }

    fn error(&self) -> Option<ReceiptError> {
        rename_field_error(self.0.error(), |field| {
            format!("systemd:\"{}\"", field.unwrap_or_default())
        })
    }
}

impl Build for SystemdField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        for (name, entry) in self.value()? {
            build_entry(builder, &name, entry)?;
        }
        Ok(())
    }
}

// ---- Service entry ----

/// A single entry in the `systemd` field.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SystemdEntry {
    /// Installs the unit as a user service.
    pub user: Option<bool>,
    /// Content of the `.service` unit file.
    pub service: Option<ServiceUnit>,
    /// Content of the `.socket` unit file.
    pub socket: Option<SocketUnit>,
    /// Content of the `.timer` unit file.
    pub timer: Option<TimerUnit>,
    /// Content of the `.path` unit file.
    pub path: Option<PathUnit>,
    /// Content of the `.target` unit file.
    pub target: Option<TargetUnit>,
}

// ---- Unit-file structs ----

/// Content for a `.service` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServiceUnit {
    /// Whether to enable this service unit.
    #[serde(skip_serializing)]
    pub enabled: Option<bool>,

    pub unit: Option<UnitSection>,
    pub service: Option<ServiceSection>,
    pub install: Option<InstallSection>,
}

/// Content for a `.socket` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SocketUnit {
    /// Whether to enable this socket unit.
    #[serde(skip_serializing)]
    pub enabled: Option<bool>,

    pub unit: Option<UnitSection>,
    pub socket: Option<SocketSection>,
    pub install: Option<InstallSection>,
}

/// Content for a `.timer` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct TimerUnit {
    /// Whether to enable this timer unit.
    #[serde(skip_serializing)]
    pub enabled: Option<bool>,

    pub unit: Option<UnitSection>,
    pub timer: Option<TimerSection>,
    pub install: Option<InstallSection>,
}

/// Content for a `.target` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct TargetUnit {
    /// Whether to enable this target unit.
    #[serde(skip_serializing)]
    pub enabled: Option<bool>,

    pub unit: Option<UnitSection>,
    pub install: Option<InstallSection>,
}

/// Content for a `.path` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PathUnit {
    /// Whether to enable this path unit.
    #[serde(skip_serializing)]
    pub enabled: Option<bool>,

    pub unit: Option<UnitSection>,
    pub path: Option<PathSection>,
    pub install: Option<InstallSection>,
}

// ---- Sections ----

/// `[Unit]` section shared by unit files.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct UnitSection {
    /// Human-readable description of the unit.
    pub description: Option<String>,
    /// Units that this unit is ordered after.
    pub after: IniMulti<String>,
    /// Units this unit wants but does not strictly require.
    pub wants: IniMulti<String>,
    /// Units this unit hard-requires to start.
    pub requires: IniMulti<String>,
    /// Documentation URIs.
    pub documentation: Option<String>,
    /// Less-common `[Unit]` directives not listed above.
    #[serde(flatten)]
    pub extra: IniExtra,
}

/// `[Service]` section of a `.service` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
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
    pub restart_sec: Option<serde_value::Value>,
    /// Seconds to wait for startup. Accepts an integer or a string like `30s`.
    pub timeout_start_sec: Option<serde_value::Value>,
    /// `KEY=VALUE` environment variables.
    pub environment: IniMulti<String>,
    /// Less-common `[Service]` directives not listed above.
    #[serde(flatten)]
    pub extra: IniExtra,
}

/// `[Install]` section shared by unit files.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct InstallSection {
    /// Targets that pull this unit in when enabled.
    pub wanted_by: IniMulti<String>,
    /// Additional units to enable or disable alongside this one.
    pub also: IniMulti<String>,
    /// Less-common `[Install]` directives not listed above.
    #[serde(flatten)]
    pub extra: IniExtra,
}

/// `[Socket]` section of a `.socket` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SocketSection {
    /// Filesystem path or address to listen on.
    pub listen_stream: Option<String>,
    /// Octal permission mode for the socket node (e.g. `0660`).
    pub socket_mode: Option<serde_value::Value>,
    /// User that owns the socket node.
    pub socket_user: Option<String>,
    /// Group that owns the socket node.
    pub socket_group: Option<String>,
    /// Less-common `[Socket]` directives not listed above.
    #[serde(flatten)]
    pub extra: IniExtra,
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

// ---- Build implementation ----

fn build_entry(builder: &mut Builder, name: &str, entry: SystemdEntry) -> Result<(), ReceiptError> {
    let is_user = entry.user.unwrap_or(false);

    build_service_entry(builder, name, is_user, entry.service)?;
    build_socket_entry(builder, name, is_user, entry.socket)?;
    build_timer_entry(builder, name, is_user, entry.timer)?;
    build_path_entry(builder, name, is_user, entry.path)?;
    build_target_entry(builder, name, is_user, entry.target)?;

    Ok(())
}

fn unit_dir(is_user: bool) -> &'static str {
    if is_user {
        "/usr/lib/systemd/user"
    } else {
        "/usr/lib/systemd/system"
    }
}

fn enable_unit(builder: &mut Builder, unit_name: &str, is_user: bool) {
    if is_user {
        builder.push(format!("RUN systemctl --global enable {unit_name}"));
    } else {
        builder.push(format!("RUN systemctl enable {unit_name}"));
    }
}

fn make_unit_file(
    builder: &mut Builder,
    unit_name: &str,
    is_user: bool,
    content: &str,
) -> Result<(), ReceiptError> {
    let unit_dir = unit_dir(is_user);
    let unit_path = target_to_filename(&format!("{unit_dir}/{unit_name}"));
    std::fs::write(builder.build_dir().join(&unit_path), content)?;
    builder.push(format!("COPY {unit_path} {unit_dir}/{unit_name}"));
    Ok(())
}

fn build_service_entry(
    builder: &mut Builder,
    name: &str,
    is_user: bool,
    service: Option<ServiceUnit>,
) -> Result<(), ReceiptError> {
    if let Some(unit) = service {
        let enabled = unit.enabled.unwrap_or(false);
        let unit_name = &format!("{name}.service");

        let has_sections = unit.unit.is_some() || unit.service.is_some() || unit.install.is_some();

        if has_sections {
            make_unit_file(builder, unit_name, is_user, &render_unit_file(&unit)?)?;
        }

        if enabled {
            enable_unit(builder, unit_name, is_user);
        }
    }

    Ok(())
}

fn build_socket_entry(
    builder: &mut Builder,
    name: &str,
    is_user: bool,
    socket: Option<SocketUnit>,
) -> Result<(), ReceiptError> {
    if let Some(unit) = socket {
        let enabled = unit.enabled.unwrap_or(false);
        let unit_name = &format!("{name}.socket");

        let has_sections = unit.unit.is_some() || unit.socket.is_some() || unit.install.is_some();

        if has_sections {
            make_unit_file(builder, unit_name, is_user, &render_unit_file(&unit)?)?;
        }

        if enabled {
            enable_unit(builder, unit_name, is_user);
        }
    }

    Ok(())
}

fn build_timer_entry(
    builder: &mut Builder,
    name: &str,
    is_user: bool,
    timer: Option<TimerUnit>,
) -> Result<(), ReceiptError> {
    if let Some(unit) = timer {
        let enabled = unit.enabled.unwrap_or(false);
        let unit_name = &format!("{name}.timer");

        let has_sections = unit.unit.is_some() || unit.timer.is_some() || unit.install.is_some();

        if has_sections {
            make_unit_file(builder, unit_name, is_user, &render_unit_file(&unit)?)?;
        }

        if enabled {
            enable_unit(builder, unit_name, is_user);
        }
    }

    Ok(())
}

fn build_path_entry(
    builder: &mut Builder,
    name: &str,
    is_user: bool,
    path: Option<PathUnit>,
) -> Result<(), ReceiptError> {
    if let Some(unit) = path {
        let enabled = unit.enabled.unwrap_or(false);
        let unit_name = &format!("{name}.path");

        let has_sections = unit.unit.is_some() || unit.path.is_some() || unit.install.is_some();

        if has_sections {
            make_unit_file(builder, unit_name, is_user, &render_unit_file(&unit)?)?;
        }

        if enabled {
            enable_unit(builder, unit_name, is_user);
        }
    }

    Ok(())
}

fn build_target_entry(
    builder: &mut Builder,
    name: &str,
    is_user: bool,
    target: Option<TargetUnit>,
) -> Result<(), ReceiptError> {
    if let Some(unit) = target {
        let enabled = unit.enabled.unwrap_or(false);
        let unit_name = &format!("{name}.target");

        let has_sections = unit.unit.is_some() || unit.install.is_some();

        if has_sections {
            make_unit_file(builder, unit_name, is_user, &render_unit_file(&unit)?)?;
        }

        if enabled {
            enable_unit(builder, unit_name, is_user);
        }
    }

    Ok(())
}

// ---- Rendering ----

fn render_unit_file<T: Serialize>(unit: &T) -> Result<String, ReceiptError> {
    crate::ini::to_string(unit).map_err(|e| std::io::Error::other(e).into())
}
