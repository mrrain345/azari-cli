use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::recipe::RecipeAlt;
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::map::RecipeMap;
use crate::recipe::systemd::ServiceUnit;
use crate::recipe::systemd::entry::SystemdEntry;

/// # Systemd
/// Systemd units to enable or define.
///
/// Supports two forms:
///
/// **Simple form:** enable existing services by name.
/// ```yaml
/// systemd:
///   - NetworkManager
///   - cups
/// ```
///
/// **Complex form:** define full units inline.
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
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[serde(transparent)]
pub struct SystemdField(RecipeAlt<Vec<String>, RecipeMap<String, SystemdEntry>>);

impl RecipeField for SystemdField {
    type Value = Vec<(String, SystemdEntry)>;

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

#[cfg(test)]
mod tests {
    use super::SystemdField;
    use crate::recipe::field::RecipeField;

    #[test]
    fn simple_form_deserializes_to_enabled_services() {
        let field: SystemdField = serde_saphyr::from_str("- NetworkManager\n- cups").unwrap();

        let values = field.value().unwrap();
        assert_eq!(values.len(), 2);

        let names: Vec<String> = values.iter().map(|(name, _)| name.clone()).collect();
        assert_eq!(
            names,
            vec!["NetworkManager".to_string(), "cups".to_string()]
        );

        for (_, entry) in values {
            assert!(entry.service.enabled);
            assert!(!entry.socket.enabled);
            assert!(!entry.timer.enabled);
            assert!(!entry.path.enabled);
            assert!(!entry.target.enabled);
        }
    }

    #[test]
    fn null_deserializes_to_empty() {
        let field: SystemdField = serde_saphyr::from_str("~").unwrap();
        assert!(field.value().unwrap().is_empty());
    }
}
