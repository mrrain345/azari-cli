use crate::distro::DistroOps;

pub struct Ubuntu;

impl DistroOps for Ubuntu {
    fn distro(&self) -> &'static str {
        "ubuntu"
    }

    fn default_image(&self) -> &'static str {
        "ghcr.io/bootcrew/ubuntu-bootc:latest"
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
}
