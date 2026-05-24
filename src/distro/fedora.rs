use crate::distro::{DistroOps, UserConfig, common};

pub struct Fedora;

impl DistroOps for Fedora {
    fn distro(&self) -> &'static str {
        "fedora"
    }

    fn default_image(&self) -> &'static str {
        "quay.io/fedora/fedora-bootc:latest"
    }

    fn set_hostname(&self, hostname: &str) -> Option<String> {
        Some(common::set_hostname(hostname))
    }

    fn install_packages(&self, packages: &[&str]) -> Option<String> {
        if packages.is_empty() {
            return None;
        }

        Some(format!(
            "RUN dnf install -y {} && dnf clean all",
            packages.join(" ")
        ))
    }

    fn add_user(&self, config: &UserConfig) -> Vec<String> {
        common::add_user(config)
    }
}
