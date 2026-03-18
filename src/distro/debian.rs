use crate::distro::{DistroOps, UserConfig};

pub struct Debian;

impl DistroOps for Debian {
    fn distro(&self) -> &'static str {
        "debian"
    }

    fn default_image(&self) -> &'static str {
        "ghcr.io/bootcrew/debian-bootc:latest"
    }

    fn set_hostname(&self, hostname: &str) -> Option<String> {
        Some(format!("RUN hostnamectl set-hostname {}", hostname))
    }

    fn install_packages(&self, packages: &[&str]) -> Option<String> {
        if packages.is_empty() {
            return None;
        }

        Some(format!(
            "RUN apt-get update && apt-get install -y {} && apt-get clean && rm -rf /var/lib/apt/lists/*",
            packages.join(" ")
        ))
    }

    fn add_user(&self, config: &UserConfig) -> Vec<String> {
        crate::distro::common::add_user(config)
    }
}
