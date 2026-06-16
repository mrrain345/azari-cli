use std::path::PathBuf;

use tempfile::TempDir;

use crate::receipt::ReceiptError;

/// Returns the isolated storage root used for user-side builds.
pub fn user_storage() -> PathBuf {
    let home = std::env::home_dir().expect("Failed to get home directory");
    PathBuf::from(home).join(".cache/azari/storage")
}

/// Returns the temporary directory used for user-side builds.
pub fn user_tmp_dir() -> PathBuf {
    let home = std::env::home_dir().expect("Failed to get home directory");
    let dir = PathBuf::from(home).join(".cache/azari/tmp");

    // Ensure the directory exists
    std::fs::create_dir_all(&dir).expect("Failed to create user tmp directory");
    dir
}

/// Creates a temporary build directory.
///
/// If `build_dir` is provided, creates a persistent directory at the specified path.
/// Otherwise, creates a temporary directory that will be automatically deleted when dropped.
pub fn make_build_dir(build_dir: Option<std::path::PathBuf>) -> Result<TempDir, ReceiptError> {
    let cleanup = build_dir.is_none();
    let path = build_dir.unwrap_or_else(user_tmp_dir);

    std::fs::create_dir_all(&path)?;

    let dir = tempfile::Builder::new()
        .prefix("azari-build-")
        .disable_cleanup(!cleanup)
        .tempdir_in(path)?;

    Ok(dir)
}

/// Gets the current timestamp as an RFC3339 string, for use in OCI labels.
pub fn get_timestamp_str() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Checks whether `name` is available on `PATH`, returning an error if not.
pub fn require_command(name: &str) -> Result<(), ReceiptError> {
    let found = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false);
    if !found {
        return Err(ReceiptError::CommandNotFound(name.to_owned()));
    }
    Ok(())
}

/// Executes the given command, returning an error if it fails to start or exits with a non-zero code.
pub fn execute_command(
    mut cmd: std::process::Command,
    name: impl Into<String>,
) -> Result<(), ReceiptError> {
    let name = name.into();

    let status = cmd
        .status()
        .map_err(|e| ReceiptError::CommandFailed(name.clone(), e.raw_os_error().unwrap_or(0)))?;

    if !status.success() {
        return Err(ReceiptError::CommandFailed(
            name,
            status.code().unwrap_or(0),
        ));
    }

    Ok(())
}
