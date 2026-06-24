use merge::Merge;
use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::distro::UserConfig;
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::map::RecipeMap;

/// Describes a single user account to provision inside the container image.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct UserEntry {
    /// GECOS field / display name.
    pub fullname: Option<String>,
    /// Pre-hashed (crypt(3)) password string, passed directly to `useradd -p`.
    /// When `None` the account is left passwordless via `passwd -d`.
    ///
    /// Note: `useradd -p` does **not** hash plaintext — supply an already-hashed
    /// value (e.g. the output of `openssl passwd -6`).
    pub password: Option<String>,
    /// Numeric UID. When `None` the system picks the next available UID.
    pub uid: Option<u32>,
    /// Login shell path (e.g. `/bin/bash`).
    pub shell: Option<String>,
    /// Home directory path. Defaults to `/home/<username>` when `None`.
    pub home: Option<String>,
    /// Supplementary groups the user should belong to.
    pub groups: Vec<String>,
}

/// Field for the `users` key.
///
/// A map from usernames to [`UserEntry`] descriptors. Entries from every
/// source are merged into a single ordered map. Duplicate usernames across
/// sources are treated as a conflict and return [`RecipeError::FieldConflict`].
#[derive(Debug, Default, Deserialize, Merge)]
#[serde(transparent)]
pub struct UsersField(RecipeMap<String, UserEntry>);

impl RecipeField for UsersField {
    type Value = Vec<(String, UserEntry)>;

    fn name() -> Option<&'static str> {
        Some("users")
    }

    fn value(self) -> Result<Self::Value, RecipeError> {
        self.0.value()
    }

    fn error(&self) -> Option<RecipeError> {
        rename_field_error(self.0.error(), |field| {
            format!("users:\"{}\"", field.unwrap_or_default())
        })
    }
}

impl Build for UsersField {
    fn build(self, builder: &mut Builder) -> Result<(), RecipeError> {
        let users = self.value()?;

        if users.is_empty() {
            return Ok(());
        }

        let distro = builder.distro()?;

        for (username, entry) in &users {
            let config = UserConfig {
                username,
                fullname: entry.fullname.as_deref(),
                password: entry.password.as_deref(),
                uid: entry.uid,
                shell: entry.shell.as_deref(),
                home: entry.home.as_deref(),
                groups: &entry.groups,
            };
            for instruction in distro.add_user(&config) {
                builder.push(instruction);
            }
        }

        Ok(())
    }
}
