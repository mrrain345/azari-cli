use std::{os::unix::fs::PermissionsExt, path::PathBuf};

use tempfile::TempDir;

use crate::builder::error::BuildError;

/// Returns the XDG cache base, falling back to `~/.cache`.
fn xdg_cache_home() -> PathBuf {
    match std::env::var_os("XDG_CACHE_HOME") {
        Some(val) if !val.is_empty() => PathBuf::from(val),
        _ => std::env::home_dir()
            .expect("Failed to get home directory")
            .join(".cache"),
    }
}

/// Returns the isolated storage root used for user-side builds.
pub fn user_storage() -> PathBuf {
    xdg_cache_home().join("azari/storage")
}

/// Returns the temporary directory used for user-side builds.
pub fn user_tmp_dir() -> PathBuf {
    let dir = xdg_cache_home().join("azari/tmp");
    std::fs::create_dir_all(&dir).expect("Failed to create user tmp directory");
    dir
}

/// Creates a temporary build directory.
///
/// If `build_dir` is provided, creates a persistent directory at the specified path.
/// Otherwise, creates a temporary directory that will be automatically deleted when dropped.
pub fn make_build_dir(build_dir: Option<std::path::PathBuf>) -> Result<TempDir, BuildError> {
    let cleanup = build_dir.is_none();
    let path = build_dir.unwrap_or_else(user_tmp_dir);
    std::fs::create_dir_all(&path)?;

    tempfile::Builder::new()
        .prefix("azari-build-")
        .disable_cleanup(!cleanup)
        .tempdir_in(path)
        .map_err(BuildError::Io)
}

/// Clears the user temporary directory of all files and subdirectories.
pub fn clear_tmp_dir() -> std::io::Result<()> {
    let tmp_dir = user_tmp_dir();

    for entry in std::fs::read_dir(&tmp_dir)? {
        let path = entry?.path();
        set_permissions_recursively(&path)?;
        std::fs::remove_dir_all(path)?;
    }

    Ok(())
}

/// Delete the entire azari cache directory, fixing permissions first.
pub fn remove_cache_dir() -> Result<(), BuildError> {
    let cache_dir = xdg_cache_home().join("azari");

    if cache_dir.exists() {
        set_permissions_recursively(&cache_dir)?;
        std::fs::remove_dir_all(&cache_dir)?;
    }

    Ok(())
}

/// Recursively sets permissions to 700 for the given path and all its children.
fn set_permissions_recursively(path: &std::path::Path) -> std::io::Result<()> {
    if path.is_symlink() {
        return Ok(()); // Skip symlinks to avoid affecting files outside the cache directory
    }

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            set_permissions_recursively(&entry?.path())?;
        }
    }

    Ok(())
}

/// Gets the current timestamp as an RFC3339 string, for use in OCI labels.
pub fn get_timestamp_str() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Checks whether `name` is available on `PATH`, returning an error if not.
pub fn require_command(name: &str) -> Result<(), BuildError> {
    let found = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false);

    match found {
        true => Ok(()),
        false => Err(BuildError::CommandNotFound(name.to_owned())),
    }
}

/// Executes the given command, returning an error if it fails to start or exits with a non-zero code.
pub fn execute_command(
    mut cmd: std::process::Command,
    name: impl Into<String>,
) -> Result<(), BuildError> {
    let name = name.into();

    let status = cmd
        .status()
        .map_err(|e| BuildError::CommandFailed(name.clone(), e.raw_os_error().unwrap_or(0)))?;

    match status.success() {
        true => Ok(()),
        false => Err(BuildError::CommandFailed(name, status.code().unwrap_or(0))),
    }
}
