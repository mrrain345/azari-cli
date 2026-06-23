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

/// Field for the `services` key.
///
/// Accepts either a flat list of service names to enable (simple form) or a
/// map from names to full unit-file descriptors (complex form).
///
/// **Simple form** — just enable pre-installed services:
/// ```yaml
/// services:
///   - NetworkManager
///   - cups
/// ```
///
/// **Complex form** — define unit files inline:
/// ```yaml
/// services:
///   my-service:
///     enabled: true
///     service:
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
pub struct ServicesField(ReceiptAlt<Vec<String>, ReceiptMap<String, ServiceEntry>>);

// ---- From impl for simple-form deserialization ----

/// Converts a flat list of service names into a `ReceiptMap` where each
/// service is enabled with no explicit unit file.
impl From<Vec<String>> for ReceiptMap<String, ServiceEntry> {
    fn from(names: Vec<String>) -> Self {
        ReceiptMap::new(
            names
                .into_iter()
                .map(|name| {
                    (
                        name,
                        ServiceEntry {
                            enabled: Some(true),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        )
    }
}

impl ReceiptField for ServicesField {
    type Value = Vec<(String, ServiceEntry)>;

    fn name() -> Option<&'static str> {
        Some("services")
    }

    fn value(self) -> Result<Self::Value, ReceiptError> {
        self.0.value()
    }

    fn error(&self) -> Option<ReceiptError> {
        rename_field_error(self.0.error(), |field| {
            format!("services:\"{}\"", field.unwrap_or_default())
        })
    }
}

impl Build for ServicesField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        for (name, entry) in self.value()? {
            build_entry(builder, &name, entry)?;
        }
        Ok(())
    }
}

// ---- Service entry ----

/// A single entry in the `services` map.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServiceEntry {
    /// Whether to enable the service at boot.
    pub enabled: Option<bool>,
    /// When present, installs the unit as a user service.
    pub user: Option<bool>,
    /// Content of the `.service` unit file.
    pub service: Option<ServiceUnitFile>,
    /// Content of the `.socket` unit file.
    pub socket: Option<SocketUnitFile>,
}

// ---- Unit-file structs ----

/// Content for a `.service` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServiceUnitFile {
    pub unit: Option<UnitSection>,
    pub service: Option<ServiceSection>,
    pub install: Option<InstallSection>,
}

/// Content for a `.socket` unit file.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SocketUnitFile {
    /// Whether to enable this socket unit. Not written to the unit file.
    #[serde(skip_serializing)]
    pub enabled: Option<bool>,
    pub unit: Option<UnitSection>,
    pub socket: Option<SocketSection>,
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

// ---- Build implementation ----

fn build_entry(builder: &mut Builder, name: &str, entry: ServiceEntry) -> Result<(), ReceiptError> {
    let enabled = entry.enabled.unwrap_or(false);
    let is_user = entry.user.unwrap_or(false);

    build_service_entry(builder, name, enabled, is_user, entry.service)?;
    build_socket_entry(builder, name, is_user, entry.socket)?;

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
    enabled: bool,
    is_user: bool,
    service: Option<ServiceUnitFile>,
) -> Result<(), ReceiptError> {
    let unit_name = &format!("{name}.service");

    if let Some(unit_file) = service {
        make_unit_file(builder, unit_name, is_user, &render_unit_file(&unit_file)?)?;
    }

    if enabled {
        enable_unit(builder, unit_name, is_user);
    }

    Ok(())
}

fn build_socket_entry(
    builder: &mut Builder,
    name: &str,
    is_user: bool,
    socket: Option<SocketUnitFile>,
) -> Result<(), ReceiptError> {
    if let Some(socket_unit) = socket {
        let enabled = socket_unit.enabled.unwrap_or(false);
        let unit_name = &format!("{name}.socket");

        let has_sections = socket_unit.unit.is_some()
            || socket_unit.socket.is_some()
            || socket_unit.install.is_some();

        if has_sections {
            make_unit_file(
                builder,
                unit_name,
                is_user,
                &render_unit_file(&socket_unit)?,
            )?;
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
