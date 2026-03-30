use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::receipt::ReceiptError;

/// Returns the isolated storage root used for user-side builds.
///
/// Uses `$HOME/.cache/azari/storage`, kept separate from the user's regular
/// podman storage so that azari images never appear in — or are affected by —
/// a plain `podman images`.
fn user_storage() -> PathBuf {
    let home = std::env::home_dir().expect("Failed to get home directory");
    PathBuf::from(home).join(".cache/azari/storage")
}

/// Runs `podman build` in `build_dir` using an isolated `storage_root` so
/// the built image is stored separately from the user's regular images.
///
/// Always tags the image as `<image>:latest`. When `version` is `Some`, also
/// tags it as `<image>:<version>`.
pub(crate) fn podman_build(
    build_dir: &Path,
    image: &str,
    version: Option<&str>,
) -> Result<(), ReceiptError> {
    let mut cmd = std::process::Command::new("podman");
    cmd.arg("--root")
        .arg(user_storage())
        .arg("build")
        .arg("--pull=newer")
        .arg("--cap-add=all")
        .arg("--security-opt=label=type:container_runtime_t")
        .arg("--device=/dev/fuse")
        .arg("--network=host")
        .arg("-f=Containerfile")
        .arg(format!("-t={image}:latest"));

    if let Some(ver) = version {
        cmd.arg(format!("-t={image}:{ver}"));
    }

    let status = cmd.arg(".").current_dir(build_dir).status()?;

    if !status.success() {
        return Err(ReceiptError::PodmanBuildFailed(status.code().unwrap_or(-1)));
    }

    Ok(())
}

/// Removes dangling (untagged, unreferenced) layers from the user's isolated
/// storage (`~/.cache/azari/storage`) after a build.
///
/// When a tag moves to a freshly built image the previous image becomes
/// dangling and is pruned here. Errors are silently ignored.
pub(crate) fn podman_prune() {
    let _ = std::process::Command::new("podman")
        .arg("--root")
        .arg(user_storage())
        .arg("image")
        .arg("prune")
        .arg("--filter=until=720h") // 30 days
        .arg("--force")
        .status();
}

/// Removes images matching `image` from root's default containers-storage
/// that are older than 30 days.
///
/// Only images whose reference (name) matches `image` are considered, so
/// no other images present in root storage are ever touched. Errors are
/// silently ignored — this is best-effort cleanup.
pub(crate) fn podman_prune_old_root_images(image: &str) {
    let Ok(output) = std::process::Command::new("sudo")
        .arg("podman")
        .arg("images")
        .arg(format!("--filter=reference={image}"))
        .arg("--filter=until=720h") // 30 days
        .arg("--quiet")
        .output()
    else {
        return;
    };

    let ids: Vec<&str> = std::str::from_utf8(&output.stdout)
        .unwrap_or("")
        .lines()
        .filter(|s| !s.is_empty())
        .collect();

    if ids.is_empty() {
        return;
    }

    let _ = std::process::Command::new("sudo")
        .arg("podman")
        .arg("rmi")
        .arg("--force")
        .args(&ids)
        .status();
}

/// Transfers `image:tag` from the user's isolated storage to root's
/// storage.
pub(crate) fn podman_transfer(image: &str, tag: &str) -> Result<(), ReceiptError> {
    let mut save = std::process::Command::new("podman")
        .arg("--root")
        .arg(user_storage())
        .arg("save")
        .arg(format!("{image}:{tag}"))
        .stdout(Stdio::piped())
        .spawn()?;

    let save_stdout = save.stdout.take().expect("save stdout is piped");

    // Load into root's default storage (/var/lib/containers/storage) so that
    // the bind-mount paths inside the install container are consistent with
    // the metadata embedded in the image layers.
    let load_status = std::process::Command::new("sudo")
        .arg("podman")
        .arg("load")
        .stdin(save_stdout)
        .status()?;

    let save_status = save.wait()?;

    if !save_status.success() {
        return Err(ReceiptError::PodmanTransferFailed(
            save_status.code().unwrap_or(-1),
        ));
    }
    if !load_status.success() {
        return Err(ReceiptError::PodmanTransferFailed(
            load_status.code().unwrap_or(-1),
        ));
    }

    Ok(())
}

/// Pre-allocate a file.
pub(crate) fn fallocate(path: &Path, size: &str) -> Result<(), ReceiptError> {
    let status = std::process::Command::new("sudo")
        .arg("fallocate")
        .arg("-l")
        .arg(size)
        .arg(path)
        .status()?;

    if !status.success() {
        return Err(ReceiptError::FallocateFailed(status.code().unwrap_or(-1)));
    }

    Ok(())
}

/// Install the image to the target device.
pub(crate) fn podman_install(
    image: &str,
    version: &str,
    device: &str,
    wipe: bool,
    via_loopback: bool,
) -> Result<(), ReceiptError> {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("podman")
        .arg("run")
        .arg("-it")
        .arg("--rm")
        .arg("--privileged")
        .arg("--pid=host")
        .arg("-v=/dev:/dev")
        .arg("-v=/var/lib/containers:/var/lib/containers:Z")
        .arg("-v=/etc/containers:/etc/containers:Z")
        .arg("-v=/sys/fs/selinux:/sys/fs/selinux")
        .arg("--security-opt=label=type:unconfined_t");

    let loopback_path: String;

    if via_loopback {
        let host_path = Path::new(device)
            .canonicalize()
            .map_err(|e| ReceiptError::Io(e))?;
        let parent = host_path.parent().unwrap_or(Path::new("/"));
        let filename = host_path
            .file_name()
            .expect("loopback device path has no filename")
            .to_string_lossy();

        loopback_path = format!("/run/azari-install/{filename}");
        cmd.arg(format!("-v={}:/run/azari-install:Z", parent.display()));
    } else {
        loopback_path = String::new();
    }

    cmd.arg(format!("{image}:{version}"))
        .arg("bootc")
        .arg("install")
        .arg("to-disk")
        .arg("--composefs-backend")
        .arg("--bootloader=systemd")
        .arg("--target-transport=containers-storage");

    if wipe {
        cmd.arg("--wipe");
    }

    if via_loopback {
        cmd.arg("--via-loopback");
        cmd.arg(loopback_path);
    } else {
        cmd.arg(device);
    }

    println!("Running command:\n{:?}", cmd);

    let status = cmd.status()?;
    if !status.success() {
        return Err(ReceiptError::InstallFailed(status.code().unwrap_or(-1)));
    }

    Ok(())
}
