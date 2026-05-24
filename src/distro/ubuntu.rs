use crate::distro::{DistroOps, UserConfig, common};

pub struct Ubuntu;

impl DistroOps for Ubuntu {
    fn distro(&self) -> &'static str {
        "ubuntu"
    }

    fn default_image(&self) -> &'static str {
        "ghcr.io/bootcrew/ubuntu-bootc:latest"
    }

    fn set_hostname(&self, hostname: &str) -> Option<String> {
        Some(common::set_hostname(hostname))
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
        common::add_user(config)
    }
}
