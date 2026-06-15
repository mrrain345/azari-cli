use std::path::{Path, PathBuf};
use std::process::Stdio;

use crate::receipt::ReceiptError;

use super::Builder;

/// Returns the isolated storage root used for user-side builds.
fn user_storage() -> PathBuf {
    let home = std::env::home_dir().expect("Failed to get home directory");
    PathBuf::from(home).join(".cache/azari/storage")
}

/// Returns the temporary directory used for user-side builds.
fn user_tmp_dir() -> PathBuf {
    let home = std::env::home_dir().expect("Failed to get home directory");
    let dir = PathBuf::from(home).join(".cache/azari/tmp");

    // Ensure the directory exists
    std::fs::create_dir_all(&dir).expect("Failed to create user tmp directory");
    dir
}

/// Builds the image according to the provided `Builder` and the receipt it was constructed from,
/// tagging it as `<image>:latest` and `<image>:<version>` (when `version` is set).
///
/// When `dry` is `true`, prints the `podman build` command that would be run without executing it,
/// and adds it as a comment trailer to the Containerfile.
pub(crate) fn podman_build(builder: &mut Builder, dry: bool) -> Result<(), ReceiptError> {
    // TODO: Check if podman is installed
    let tmp_dir = user_tmp_dir();
    let image = builder.image()?;
    let build_dir = builder.build_dir();

    std::fs::create_dir(build_dir.join("chunkah"))?;

    let mut cmd = std::process::Command::new("podman");
    cmd.env("TMPDIR", &tmp_dir)
        .arg("--root")
        .arg(user_storage())
        .arg("build")
        .arg("--pull=newer")
        .arg("--cap-add=all")
        .arg("--security-opt=label=type:container_runtime_t")
        .arg("--skip-unused-stages=false")
        .arg(format!(
            "--volume={}/chunkah:/usr/lib/azari/chunkah",
            build_dir.display()
        ))
        .arg("--device=/dev/fuse")
        .arg("--network=host")
        .arg("--file=Containerfile")
        .arg(format!("--tag={image}:latest"));

    for (key, val) in builder.oci_labels() {
        cmd.arg(format!("--annotation={key}={val}"));
    }

    if let Some(ver) = builder.version() {
        cmd.arg(format!("--tag={image}:{ver}"));
    }

    cmd.arg(build_dir).current_dir(build_dir);

    if dry {
        println!("Build command:\n{cmd:?}");
        builder.push(format!("\n# Build command:\n# {cmd:?}"));
        builder.write_containerfile()?;
        return Ok(());
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(ReceiptError::CommandFailed(
            "podman build".into(),
            status.code().unwrap_or(0),
        ));
    }

    Ok(())
}

/// Pushes `image` from the user's isolated storage to its remote registry.
pub(crate) fn podman_push(
    image: &str,
    version: Option<&str>,
    push_latest: bool,
) -> Result<(), ReceiptError> {
    let push_tag = |tag: &str| -> Result<(), ReceiptError> {
        let status = std::process::Command::new("podman")
            .arg("--root")
            .arg(user_storage())
            .arg("push")
            .arg(format!("{image}:{tag}"))
            .status()?;

        if !status.success() {
            return Err(ReceiptError::CommandFailed(
                "podman push".into(),
                status.code().unwrap_or(0),
            ));
        }

        Ok(())
    };

    if let Some(ver) = version {
        push_tag(ver)?;
    }

    if push_latest {
        push_tag("latest")?;
    }

    Ok(())
}

/// Transfers `image:tag` from the user's isolated storage to root's storage.
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
        return Err(ReceiptError::CommandFailed(
            "podman save".into(),
            save_status.code().unwrap_or(0),
        ));
    }
    if !load_status.success() {
        return Err(ReceiptError::CommandFailed(
            "podman load".into(),
            load_status.code().unwrap_or(0),
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
        return Err(ReceiptError::CommandFailed(
            "fallocate".into(),
            status.code().unwrap_or(0),
        ));
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
        .arg("--pull=newer")
        .arg("--privileged")
        .arg("--pid=host")
        .arg("-v=/dev:/dev")
        .arg("-v=/var/lib/containers:/var/lib/containers:Z")
        .arg("-v=/etc/containers:/etc/containers:Z");
    // .arg("-v=/sys/fs/selinux:/sys/fs/selinux")
    // .arg("--security-opt=label=type:unconfined_t");

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
        .arg("--filesystem=btrfs")
        .arg("--disable-selinux");
    // .arg("--block-setup=tpm2-luks");
    // .arg("--target-transport=containers-storage");

    if wipe {
        cmd.arg("--wipe");
    }

    if via_loopback {
        cmd.arg("--generic-image");
        cmd.arg("--via-loopback");
        cmd.arg(loopback_path);
    } else {
        cmd.arg(device);
    }

    println!("Running command:\n{:?}", cmd);

    let status = cmd.status()?;
    if !status.success() {
        return Err(ReceiptError::CommandFailed(
            "bootc install".into(),
            status.code().unwrap_or(0),
        ));
    }

    Ok(())
}

/// Switch the running bootc image to `image:version` via sudo.
pub(crate) fn bootc_switch(image: &str, version: &str, local: bool) -> Result<(), ReceiptError> {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("bootc").arg("switch");

    if local {
        cmd.arg("--transport=containers-storage");
    }

    cmd.arg(format!("{image}:{version}"));

    let status = cmd.status()?;
    if !status.success() {
        return Err(ReceiptError::CommandFailed(
            "bootc switch".into(),
            status.code().unwrap_or(0),
        ));
    }

    Ok(())
}

/// Run `bootc upgrade` on the host via sudo.
pub(crate) fn bootc_upgrade(version: Option<&str>) -> Result<(), ReceiptError> {
    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("bootc").arg("upgrade");

    if let Some(version) = version {
        cmd.arg(format!("--tag={version}"));
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(ReceiptError::CommandFailed(
            "bootc upgrade".into(),
            status.code().unwrap_or(0),
        ));
    }

    Ok(())
}
