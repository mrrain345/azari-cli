use crate::receipt::ReceiptError;
use serde::Deserialize;
use std::{ops::Deref, str::FromStr};

pub mod arch;
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
    type Err = ReceiptError;

    fn from_str(distro: &str) -> Result<Self, Self::Err> {
        serde_saphyr::from_str::<Self>(distro)
            .map_err(|_| ReceiptError::UnsupportedDistro(distro.to_owned()))
    }
}

/// Distro specific operations.
pub trait DistroOps {
    /// Codename of this distro.
    fn distro(&self) -> &'static str;

    /// Default OCI image.
    fn default_image(&self) -> &'static str;

    /// Build instruction for setting hostname.
    fn set_hostname(&self, hostname: &str) -> Option<String>;

    /// Build instruction for installing packages.
    fn install_packages(&self, packages: &[&str]) -> Option<String>;
}

#[cfg(test)]
mod tests {
    use super::Distro;
    use std::str::FromStr;

    const VARIANTS: &[Distro] = &[Distro::Arch, Distro::Fedora, Distro::Ubuntu, Distro::Debian];

    #[test]
    fn parses_supported_distros() {
        assert_eq!(Distro::from_str("arch").unwrap(), Distro::Arch);
        assert_eq!(Distro::from_str("fedora").unwrap(), Distro::Fedora);
        assert_eq!(Distro::from_str("ubuntu").unwrap(), Distro::Ubuntu);
        assert_eq!(Distro::from_str("debian").unwrap(), Distro::Debian);
    }

    #[test]
    fn parses_using_str_parse() {
        let distro: Distro = "arch".parse().unwrap();
        assert_eq!(distro, Distro::Arch);
    }

    #[test]
    fn rejects_unsupported_distro() {
        let err = Distro::from_str("nixos").unwrap_err();
        assert!(
            matches!(err, crate::receipt::ReceiptError::UnsupportedDistro(_)),
            "expected UnsupportedDistro, got: {err:?}"
        );
    }

    #[test]
    fn empty_packages_return_none_for_all_distros() {
        let variants = [Distro::Arch, Distro::Fedora, Distro::Ubuntu, Distro::Debian];
        for distro in variants {
            assert_eq!(distro.install_packages(&[]), None);
        }
    }

    #[test]
    fn package_instruction_is_emitted_for_non_empty_packages() {
        for distro in VARIANTS {
            let instr = distro.install_packages(&["vim"]).unwrap();
            assert!(instr.starts_with("RUN "), "unexpected instruction: {instr}");
            assert!(
                instr.contains("vim"),
                "missing package in instruction: {instr}"
            );
        }
    }

    #[test]
    fn hostname_instruction_is_emitted() {
        for distro in VARIANTS {
            let instr = distro.set_hostname("azari").unwrap();
            assert!(instr.starts_with("RUN "), "unexpected instruction: {instr}");
            assert!(
                instr.contains("azari"),
                "missing hostname in instruction: {instr}"
            );
        }
    }

    #[test]
    fn distro_names_match_variants() {
        assert_eq!(Distro::Arch.distro(), "arch");
        assert_eq!(Distro::Fedora.distro(), "fedora");
        assert_eq!(Distro::Ubuntu.distro(), "ubuntu");
        assert_eq!(Distro::Debian.distro(), "debian");
    }
}
