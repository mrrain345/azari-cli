use merge::Merge;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::RecipeAlt;
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::map::RecipeMap;
use crate::recipe::systemd::ServiceUnit;
use crate::recipe::systemd::entry::SystemdEntry;

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
pub struct SystemdField(RecipeAlt<Vec<String>, RecipeMap<String, SystemdEntry>>);

impl RecipeField for SystemdField {
    type Value = Vec<(String, SystemdEntry)>;

    fn name() -> Option<&'static str> {
        Some("systemd")
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |field| {
            format!("systemd:\"{}\"", field.unwrap_or_default())
        })
    }
}

impl Build for SystemdField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        for (name, entry) in self.value()? {
            entry.build(builder, &name)?;
        }
        Ok(())
    }
}

/// Converts a flat list of service names into a `RecipeMap` where each
/// service is enabled with no explicit unit file.
impl From<Vec<String>> for RecipeMap<String, SystemdEntry> {
    fn from(names: Vec<String>) -> Self {
        RecipeMap::new(
            names
                .into_iter()
                .map(|name| {
                    (
                        name,
                        SystemdEntry {
                            user: false,
                            service: ServiceUnit {
                                enabled: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        )
    }
}
