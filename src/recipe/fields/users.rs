use merge::Merge;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::builder::BuildError;
use crate::builder::{Build, Builder};
use crate::distro::UserConfig;
use crate::recipe::error::RecipeError;
use crate::recipe::field::{RecipeField, rename_field_error};
use crate::recipe::map::RecipeMap;

/// # User Entry
/// Username of the user.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case")]
pub struct UserEntry {
    /// # Fullname
    /// Full display name.
    pub fullname: Option<String>,
    /// # Password
    /// Plaintext or hashed password string.
    ///
    /// To generate a hashed password string, use the `openssl passwd -6` command.
    ///
    /// If not specified, the user will be created without a password (passwordless login).
    ///
    /// **NOTE:** Keep in mind that the hashed password will be stored in the final image,
    /// so anyone with access to the image can potentially retrieve it.
    pub password: Option<String>,
    /// # UID
    /// Numeric user ID.
    pub uid: Option<u32>,
    /// # Shell
    /// Login shell path.
    pub shell: Option<String>,
    /// # Home
    /// Home directory path.
    pub home: Option<String>,
    /// # Groups
    /// Extra groups to add the user to.
    pub groups: Vec<String>,
}

/// # Users
/// Users to create in the image.
///
/// Key is the username, value is account settings.
#[derive(Debug, Default, Deserialize, Merge, JsonSchema)]
#[schemars(example = r#"users:
  azari:
    fullname: "Azari User"
    shell: /bin/bash
    groups:
      - wheel
      - audio
"#)]
#[serde(transparent)]
pub struct UsersField(RecipeMap<String, UserEntry>);

impl RecipeField for UsersField {
    type Value = Vec<(String, UserEntry)>;

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
    fn build(self, builder: &mut Builder) -> Result<(), BuildError> {
        let users = self.value()?;

        if users.is_empty() {
            return Ok(());
        }

        let distro = builder.distro()?;

        for (username, entry) in users {
            let config = UserConfig {
                username,
                fullname: entry.fullname,
                password: entry.password,
                uid: entry.uid,
                shell: entry.shell,
                home: entry.home,
                groups: entry.groups,
            };

            distro.add_user(builder, &config);
        }

        Ok(())
    }
}
