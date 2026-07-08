use crate::builder::Builder;
use crate::distro::{DistroOps, UserConfig, common};

pub struct Arch;

impl DistroOps for Arch {
    fn distro(&self) -> &'static str {
        "arch"
    }

    fn default_image(&self) -> &'static str {
        "ghcr.io/bootcrew/arch-bootc:latest"
    }

    fn set_hostname(&self, builder: &mut Builder, hostname: &str) {
        common::set_hostname(builder, hostname);
    }

    fn install_packages(&self, builder: &mut Builder, packages: &[&str]) {
        if packages.is_empty() {
            return;
        }

        builder.push(format!(
            "RUN --mount=type=tmpfs,dst=/tmp \
            --mount=type=cache,dst=/usr/lib/sysimage/cache/pacman \
            pacman -Syu --noconfirm {}",
            packages.join(" ")
        ));
    }

    fn add_user(&self, builder: &mut Builder, config: &UserConfig) {
        common::add_user(builder, config);
    }
}
