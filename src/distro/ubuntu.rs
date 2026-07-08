use crate::builder::Builder;
use crate::distro::{DistroOps, UserConfig, common};

pub struct Ubuntu;

impl DistroOps for Ubuntu {
    fn distro(&self) -> &'static str {
        "ubuntu"
    }

    fn default_image(&self) -> &'static str {
        "ghcr.io/bootcrew/ubuntu-bootc:latest"
    }

    fn set_hostname(&self, builder: &mut Builder, hostname: &str) {
        common::set_hostname(builder, hostname);
    }

    fn install_packages(&self, builder: &mut Builder, packages: &[&str]) {
        if packages.is_empty() {
            return;
        }

        builder.push(format!(
            "RUN apt-get update && apt-get install -y {} && apt-get clean && rm -rf /var/lib/apt/lists/*",
            packages.join(" ")
        ));
    }

    fn add_user(&self, builder: &mut Builder, config: &UserConfig) {
        common::add_user(builder, config);
    }
}
