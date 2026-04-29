use crate::distro::{DistroOps, UserConfig};

pub struct Arch;

impl DistroOps for Arch {
    fn distro(&self) -> &'static str {
        "arch"
    }

    fn default_image(&self) -> &'static str {
        "ghcr.io/bootcrew/arch-bootc:latest"
    }

    fn set_hostname(&self, hostname: &str) -> Option<String> {
        Some(format!("RUN echo '{}' > /etc/hostname", hostname))
    }

    fn install_packages(&self, packages: &[&str]) -> Option<String> {
        if packages.is_empty() {
            return None;
        }

        Some(format!(
            "RUN --mount=type=tmpfs,dst=/tmp \
            --mount=type=cache,dst=/usr/lib/sysimage/cache/pacman \
            pacman -Syu --noconfirm {}",
            packages.join(" ")
        ))
    }

    fn add_user(&self, config: &UserConfig) -> Vec<String> {
        crate::distro::common::add_user(config)
    }
}
