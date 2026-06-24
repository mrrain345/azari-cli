use crate::ini::IniExtra;
use crate::recipe::error::RecipeError;
use crate::recipe::fields::files::target_to_filename;
use crate::{builder::Builder, ini::IniMulti};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `[Unit]` section shared by unit files.
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
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

/// `[Install]` section shared by unit files.
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
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

/// Trait for systemd unit types that can be rendered and enabled.
pub trait SystemdUnit: Serialize + Sized {
    /// The unit type for this unit (e.g., "service", "socket", "timer").
    fn unit_type() -> &'static str;

    /// Whether this unit should be enabled.
    fn enabled(&self) -> bool;

    /// Whether this unit has any sections to write to file.
    fn has_sections(&self) -> bool;

    /// Build this unit and add it to the builder.
    fn build(&self, builder: &mut Builder, name: &str, is_user: bool) -> Result<(), RecipeError> {
        let extension = Self::unit_type();
        let unit_name = format!("{name}.{extension}");

        if self.has_sections() {
            let content = render_unit_file(&self)?;
            write_unit_file(builder, &unit_name, is_user, &content)?;
        }

        if self.enabled() {
            enable_unit(builder, &unit_name, is_user);
        }

        Ok(())
    }
}

/// Returns the directory where systemd unit files should be placed.
fn unit_dir(is_user: bool) -> &'static str {
    if is_user {
        "/usr/lib/systemd/user"
    } else {
        "/usr/lib/systemd/system"
    }
}

/// Enables a systemd unit by adding the appropriate `systemctl enable` command to the builder.
fn enable_unit(builder: &mut Builder, unit_name: &str, is_user: bool) {
    if is_user {
        builder.push(format!("RUN systemctl --global enable {unit_name}"));
    } else {
        builder.push(format!("RUN systemctl enable {unit_name}"));
    }
}

/// Creates a systemd unit file with the given content and adds a copy command to the builder.
fn write_unit_file(
    builder: &mut Builder,
    unit_name: &str,
    is_user: bool,
    content: &str,
) -> Result<(), RecipeError> {
    let dir = unit_dir(is_user);
    let path = target_to_filename(&format!("{dir}/{unit_name}"));
    std::fs::write(builder.build_dir().join(&path), content)?;
    builder.push(format!("COPY {path} {dir}/{unit_name}"));
    Ok(())
}

/// Renders a systemd unit struct into an INI-formatted string.
fn render_unit_file<T: Serialize>(unit: &T) -> Result<String, RecipeError> {
    crate::ini::to_string(unit).map_err(|e| std::io::Error::other(e).into())
}
