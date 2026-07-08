use crate::builder::{BuildError, Builder};
use serde::Deserialize;
use std::{ops::Deref, str::FromStr};

pub mod arch;
pub mod common;
pub mod debian;
pub mod fedora;
pub mod ubuntu;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Distro {
    Arch,
    Fedora,
    Ubuntu,
    Debian,
}

impl Deref for Distro {
    type Target = dyn DistroOps;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Arch => &arch::Arch,
            Self::Fedora => &fedora::Fedora,
            Self::Ubuntu => &ubuntu::Ubuntu,
            Self::Debian => &debian::Debian,
        }
    }
}

impl FromStr for Distro {
    type Err = BuildError;

    fn from_str(distro: &str) -> Result<Self, Self::Err> {
        serde_saphyr::from_str::<Self>(distro)
            .map_err(|_| BuildError::UnsupportedDistro(distro.to_owned()))
    }
}

/// Parameters for creating a user account inside a container image.
pub struct UserConfig {
    pub username: String,
    /// GECOS / display name.
    pub fullname: Option<String>,
    /// Pre-hashed (crypt(3)) password string, passed directly to `useradd -p`.
    /// When `None` the account is left passwordless via `passwd -d`.
    pub password: Option<String>,
    /// Numeric UID. `None` lets the system choose.
    pub uid: Option<u32>,
    /// Login shell path (e.g. `/bin/bash`).
    pub shell: Option<String>,
    /// Home directory path.
    pub home: Option<String>,
    /// Supplementary group names.
    pub groups: Vec<String>,
}

/// Distro specific operations.
pub trait DistroOps {
    /// Codename of this distro.
    fn distro(&self) -> &'static str;

    /// Default OCI image.
    fn default_image(&self) -> &'static str;

    /// Build instruction for setting hostname.
    fn set_hostname(&self, builder: &mut Builder, hostname: &str);

    /// Build instruction for installing packages.
    fn install_packages(&self, builder: &mut Builder, packages: &[&str]);

    /// Containerfile instructions to create a user account.
    fn add_user(&self, builder: &mut Builder, config: &UserConfig);
}

#[cfg(test)]
mod tests {
    use super::Distro;
    use std::str::FromStr;

    const VARIANTS: &[Distro] = &[Distro::Arch, Distro::Fedora, Distro::Ubuntu, Distro::Debian];

    #[test]
    fn parses_supported_distros() {
        assert_eq!("arch".parse::<Distro>().unwrap(), Distro::Arch);
        assert_eq!("fedora".parse::<Distro>().unwrap(), Distro::Fedora);
        assert_eq!("ubuntu".parse::<Distro>().unwrap(), Distro::Ubuntu);
        assert_eq!("debian".parse::<Distro>().unwrap(), Distro::Debian);
    }

    #[test]
    fn rejects_unsupported_distro() {
        let err = Distro::from_str("nixos").unwrap_err();
        assert!(
            matches!(err, crate::builder::BuildError::UnsupportedDistro(_)),
            "expected UnsupportedDistro, got: {err:?}"
        );
    }
}
