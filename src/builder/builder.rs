use crate::distro::Distro;
use crate::receipt::{Receipt, ReceiptError};

use super::BuildDir;

/// Trait for building a Containerfile from a receipt field.
pub trait Build {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError>;
}

/// In-memory Containerfile builder.
#[derive(Debug)]
pub struct Builder {
    distro: Option<Distro>,
    lines: Vec<String>,
    build_dir: BuildDir,
}

impl Builder {
    /// Builds containerfile lines from a receipt.
    pub fn from_receipt(receipt: Receipt, build_dir: BuildDir) -> Result<Self, ReceiptError> {
        let mut builder = Builder {
            distro: None,
            lines: Vec::new(),
            build_dir,
        };

        // `distro` must be built first — it populates `builder.distro`,
        // which other fields read from during their build step.
        receipt.distro.build(&mut builder)?;
        receipt.from.build(&mut builder)?;
        receipt.name.build(&mut builder)?;
        receipt.hostname.build(&mut builder)?;
        receipt.preinstall.build(&mut builder)?;
        receipt.packages.build(&mut builder)?;
        receipt.files.build(&mut builder)?;
        receipt.postinstall.build(&mut builder)?;

        Ok(builder)
    }

    /// Stores the resolved distro.
    ///
    /// Must be called before [`Builder::distro`].
    pub(crate) fn set_distro(&mut self, distro: Distro) {
        self.distro = Some(distro);
    }

    /// Returns the resolved distro.
    ///
    /// [`Builder::set_distro`] must have been called beforehand.
    pub(crate) fn distro(&self) -> Result<Distro, ReceiptError> {
        self.distro.ok_or(ReceiptError::DistroNotSpecified)
    }

    /// Appends a single Containerfile instruction line.
    pub(crate) fn push(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }

    /// Returns the path to the build directory.
    pub fn build_dir(&self) -> &std::path::Path {
        self.build_dir.path()
    }

    /// Renders all instruction lines into a single Containerfile string.
    pub fn to_containerfile(&self) -> String {
        if self.lines.is_empty() {
            return String::new();
        }

        let mut out = self.lines.join("\n");
        out.push('\n');
        out
    }

    /// Writes the Containerfile to `<build_dir>/Containerfile` and returns
    /// the path to the written file.
    pub fn write_containerfile(&self) -> Result<std::path::PathBuf, ReceiptError> {
        let path = self.build_dir.path().join("Containerfile");
        std::fs::write(&path, self.to_containerfile())?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::builder::BuildDir;
    use crate::receipt::Receipt;

    fn receipt_fixture() -> Receipt {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/receipts/builder.yaml");
        Receipt::from_file(&path).unwrap()
    }

    #[test]
    fn write_containerfile_creates_file() {
        let build_dir = BuildDir::temp().unwrap();
        let path = build_dir.path().to_owned();
        let builder = Builder::from_receipt(receipt_fixture(), build_dir).unwrap();
        builder.write_containerfile().unwrap();
        assert!(path.join("Containerfile").exists());
    }

    #[test]
    fn write_containerfile_content_matches() {
        let build_dir = BuildDir::temp().unwrap();
        let builder = Builder::from_receipt(receipt_fixture(), build_dir).unwrap();
        let file_path = builder.write_containerfile().unwrap();
        let written = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, builder.to_containerfile());
    }
}
