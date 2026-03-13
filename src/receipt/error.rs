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
}
