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

// ---- Sections ----

/// `[Unit]` section for `.service` unit files.
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

/// `[Install]` section for `.service` unit files.
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

// ---- Build implementation ----

fn build_entry(builder: &mut Builder, name: &str, entry: ServiceEntry) -> Result<(), ReceiptError> {
    let enabled = entry.enabled.unwrap_or(false);
    let is_user = entry.user.unwrap_or(false);
    let filename = format!("{name}.service");

    let unit_dir = if is_user {
        "/usr/lib/systemd/user"
    } else {
        "/usr/lib/systemd/system"
    };

    if let Some(unit_file) = entry.service {
        let content = render_unit_file(&unit_file)?;
        let unit_path = target_to_filename(&format!("{unit_dir}/{filename}"));
        std::fs::write(builder.build_dir().join(&unit_path), &content)?;
        builder.push(format!("COPY {unit_path} {unit_dir}/{filename}"));
    }

    if enabled {
        if is_user {
            builder.push(format!("RUN systemctl --global enable {filename}"));
        } else {
            builder.push(format!("RUN systemctl enable {filename}"));
        }
    }

    Ok(())
}

// ---- Rendering ----

fn render_unit_file<T: Serialize>(unit: &T) -> Result<String, ReceiptError> {
    crate::ini::to_string(unit).map_err(|e| std::io::Error::other(e).into())
}
