use crate::builder::Builder;
use crate::distro::{DistroOps, UserConfig, common};

pub struct Fedora;

impl DistroOps for Fedora {
    fn distro(&self) -> &'static str {
        "fedora"
    }

    fn default_image(&self) -> &'static str {
        "quay.io/fedora/fedora-bootc:latest"
    }

    fn set_hostname(&self, builder: &mut Builder, hostname: &str) {
        common::set_hostname(builder, hostname);
    }

    fn install_packages(&self, builder: &mut Builder, packages: &[&str]) {
        if packages.is_empty() {
            return;
        }

        builder.push(format!(
            "RUN dnf install -y {} && dnf clean all",
            packages.join(" ")
        ));
    }

    fn add_user(&self, builder: &mut Builder, config: &UserConfig) {
        common::add_user(builder, config);
    }
}
