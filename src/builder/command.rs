use std::path::Path;
use std::process::Stdio;

use crate::builder::utils::{execute_command, require_command, user_storage, user_tmp_dir};
use crate::receipt::ReceiptError;

use super::Builder;

/// Builds the image according to the provided `Builder` and the receipt it was constructed from,
/// tagging it as `<image>:latest` and `<image>:<version>` (when `version` is set).
///
/// When `dry` is `true`, prints the `podman build` command that would be run without executing it,
/// and adds it as a comment trailer to the Containerfile.
pub(crate) fn podman_build(
    builder: &mut Builder,
    dry: bool,
    no_cache: bool,
) -> Result<(), ReceiptError> {
    require_command("podman")?;
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

    if no_cache {
        cmd.arg("--no-cache");
    }

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

    execute_command(cmd, "podman build")
}

/// Pushes `image` from the user's isolated storage to its remote registry.
pub(crate) fn podman_push(
    image: &str,
    version: Option<&str>,
    push_latest: bool,
) -> Result<(), ReceiptError> {
    require_command("podman")?;

    let push_tag = |tag: &str| -> Result<(), ReceiptError> {
        let mut cmd = std::process::Command::new("podman");
        cmd.arg("--root")
            .arg(user_storage())
            .arg("push")
            .arg(format!("{image}:{tag}"));

        execute_command(cmd, "podman push")
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
    require_command("sudo")?;
    require_command("podman")?;

    let mut save = std::process::Command::new("podman")
        .arg("--root")
        .arg(user_storage())
        .arg("save")
        .arg(format!("{image}:{tag}"))
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| {
            ReceiptError::CommandFailed("podman save".into(), e.raw_os_error().unwrap_or(0))
        })?;

    let save_stdout = save.stdout.take().expect("save stdout is piped");

    // Load into root's default storage (/var/lib/containers/storage) so that
    // the bind-mount paths inside the install container are consistent with
    // the metadata embedded in the image layers.
    let load_status = std::process::Command::new("sudo")
        .arg("podman")
        .arg("load")
        .stdin(save_stdout)
        .status()
        .map_err(|e| {
            ReceiptError::CommandFailed("podman load".into(), e.raw_os_error().unwrap_or(0))
        })?;

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
    require_command("sudo")?;
    require_command("fallocate")?;

    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("fallocate").arg("-l").arg(size).arg(path);

    execute_command(cmd, "fallocate")
}

/// Install the image to the target device.
pub(crate) fn podman_install(
    image: &str,
    version: &str,
    device: &str,
    wipe: bool,
    via_loopback: bool,
) -> Result<(), ReceiptError> {
    require_command("sudo")?;
    require_command("podman")?;

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
        let host_path = Path::new(device).canonicalize()?;
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

    execute_command(cmd, "bootc install")
}

/// Switch the running bootc image to `image:version` via sudo.
pub(crate) fn bootc_switch(image: &str, version: &str, local: bool) -> Result<(), ReceiptError> {
    require_command("sudo")?;
    require_command("bootc")?;

    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("bootc").arg("switch");

    if local {
        cmd.arg("--transport=containers-storage");
    }

    cmd.arg(format!("{image}:{version}"));

    execute_command(cmd, "bootc switch")
}

/// Run `bootc upgrade` on the host via sudo.
pub(crate) fn bootc_upgrade(version: Option<&str>) -> Result<(), ReceiptError> {
    require_command("sudo")?;
    require_command("bootc")?;

    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("bootc").arg("upgrade");

    if let Some(version) = version {
        cmd.arg(format!("--tag={version}"));
    }

    execute_command(cmd, "bootc upgrade")
}

/// List images in the user's isolated storage.
pub(crate) fn podman_images() -> Result<(), ReceiptError> {
    require_command("podman")?;

    let mut cmd = std::process::Command::new("podman");
    cmd.arg("--root").arg(user_storage()).arg("images");

    execute_command(cmd, "podman images")
}

/// Prune all images from the user's isolated storage except `<image>:latest`.
pub(crate) fn podman_clear(image: Option<&str>) -> Result<(), ReceiptError> {
    require_command("podman")?;

    // Prune all unused images and volumes
    let mut cmd = std::process::Command::new("podman");
    cmd.arg("--root")
        .arg(user_storage())
        .arg("image")
        .arg("prune")
        .arg("--all")
        .arg("--force");

    // Exclude the current image:latest from pruning if specified
    if let Some(image) = image {
        cmd.arg(format!("--filter=reference!={image}:latest"));
    }

    execute_command(cmd, "podman image prune")?;

    // Remove all images except the current image:latest if specified
    let mut rmi_cmd = std::process::Command::new("podman");
    rmi_cmd
        .arg("--root")
        .arg(user_storage())
        .arg("image")
        .arg("rm")
        .arg("--force");

    let filter = image.map(|image| format!("reference!={image}:latest"));
    let ids = podman_get_image_ids(filter.as_deref())?;

    if ids.is_empty() {
        return Ok(());
    }

    rmi_cmd.args(ids);
    execute_command(rmi_cmd, "podman image rm")
}

pub(crate) fn podman_get_image_ids(filter: Option<&str>) -> Result<Vec<String>, ReceiptError> {
    require_command("podman")?;

    let mut cmd = std::process::Command::new("podman");
    cmd.arg("--root")
        .arg(user_storage())
        .arg("images")
        .arg("--quiet");

    if let Some(filter) = filter {
        cmd.arg(format!("--filter={}", filter));
    }

    let output = cmd.output().map_err(|e| {
        ReceiptError::CommandFailed("podman images".into(), e.raw_os_error().unwrap_or(0))
    })?;

    if !output.status.success() {
        return Err(ReceiptError::CommandFailed(
            "podman images".into(),
            output.status.code().unwrap_or(0),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ids: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(ids)
}

/// Run `bootc usr-overlay` on the host via sudo, making /usr writable.
pub(crate) fn bootc_unlock() -> Result<(), ReceiptError> {
    require_command("sudo")?;
    require_command("bootc")?;

    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("bootc").arg("usr-overlay");

    execute_command(cmd, "bootc usr-overlay")
}

/// Run `bootc status` on the host via sudo.
pub(crate) fn bootc_status() -> Result<(), ReceiptError> {
    require_command("sudo")?;
    require_command("bootc")?;

    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("bootc").arg("status");

    execute_command(cmd, "bootc status")
}

/// Run `bootc rollback` on the host via sudo.
pub(crate) fn bootc_rollback() -> Result<(), ReceiptError> {
    require_command("sudo")?;
    require_command("bootc")?;

    let mut cmd = std::process::Command::new("sudo");
    cmd.arg("bootc").arg("rollback");

    execute_command(cmd, "bootc rollback")
}
