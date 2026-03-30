use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReceiptError {
    #[error("Invalid receipt path: unable to resolve parent directory for {0}")]
    InvalidReceiptPath(std::path::PathBuf),

    #[error("Field is defined in multiple files")]
    FieldConflict,

    #[error("Receipt path not provided: use --receipt/-r <PATH> or set the AZARI_RECEIPT env var")]
    ReceiptNotProvided,

    #[error("Failed to read receipt file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse receipt file: {0}")]
    Parse(#[from] serde_saphyr::Error),

    #[error("Unsupported distro: {0}")]
    UnsupportedDistro(String),

    #[error("Distro not specified. Add a \"distro\" field to your receipt.")]
    DistroNotSpecified,

    #[error("Build directory is not empty: {0}")]
    BuildDirNotEmpty(std::path::PathBuf),

    #[error("podman build failed with exit code {0}")]
    PodmanBuildFailed(i32),

    #[error("podman transfer (save | load) failed with exit code {0}")]
    PodmanTransferFailed(i32),

    #[error("Image name not specified. Add an \"image\" field to your receipt.")]
    ImageNotSpecified,

    #[error("Install failed with exit code {0}")]
    InstallFailed(i32),

    #[error("Target file {0} already exists. Use --wipe to overwrite.")]
    FileExistsWithoutWipe(std::path::PathBuf),

    #[error("fallocate failed with exit code {0}")]
    FallocateFailed(i32),
}
