use crate::distro::DistroOps;

pub struct Fedora;

impl DistroOps for Fedora {
    fn distro(&self) -> &'static str {
        "fedora"
    }

    fn default_image(&self) -> &'static str {
        "quay.io/fedora/fedora-bootc:latest"
    }

    fn set_hostname(&self, hostname: &str) -> Option<String> {
        Some(format!("RUN hostnamectl set-hostname {}", hostname))
    }

    fn install_packages(&self, packages: &[&str]) -> Option<String> {
        if packages.is_empty() {
            return None;
        }

        Some(format!("RUN dnf install -y {}", packages.join(" ")))
    }
}
