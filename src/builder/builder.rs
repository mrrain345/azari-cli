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
    image: Option<String>,
    version: Option<String>,
    lines: Vec<String>,
    build_dir: BuildDir,
}

impl Builder {
    /// Builds containerfile lines from a receipt.
    pub fn from_receipt(
        receipt: Receipt,
        build_dir: BuildDir,
        version: Option<String>,
    ) -> Result<Self, ReceiptError> {
        let mut builder = Builder {
            distro: None,
            image: None,
            version,
            lines: Vec::new(),
            build_dir,
        };

        // `distro` must be built first — it populates `builder.distro`,
        // which other fields read from during their build step.
        receipt.distro.build(&mut builder)?;
        receipt.image.build(&mut builder)?;
        receipt.from.build(&mut builder)?;
        receipt.name.build(&mut builder)?;
        receipt.hostname.build(&mut builder)?;
        receipt.files.build(&mut builder)?;
        receipt.preinstall.build(&mut builder)?;
        receipt.users.build(&mut builder)?;
        receipt.packages.build(&mut builder)?;
        receipt.postinstall.build(&mut builder)?;

        Ok(builder)
    }

    /// Stores the resolved distro.
    ///
    /// Must be called before [`Builder::distro`].
    pub(crate) fn set_distro(&mut self, distro: Distro) {
        self.distro = Some(distro);
    }

    /// Stores the image name for use as a base for `podman build -t` tags.
    pub(crate) fn set_image(&mut self, image: String) {
        self.image = Some(image);
    }

    /// Returns the image name, or `None` if not set.
    pub fn image(&self) -> Option<&str> {
        self.image.as_deref()
    }

    /// Returns the version, or `None` if not set.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
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

    /// Runs `podman build` in the build directory using the generated Containerfile.
    ///
    /// Uses the `image` field as the base name. Tags the image as `<image>:latest`
    /// always, and additionally as `<image>:<version>` when a version was provided
    /// to [`Builder::from_receipt`].
    ///
    /// Returns [`ReceiptError::ImageNotSpecified`] if no image name was set.
    pub fn podman_build(&self) -> Result<(), ReceiptError> {
        let image = self
            .image
            .as_deref()
            .ok_or(ReceiptError::ImageNotSpecified)?;

        let mut cmd = std::process::Command::new("podman");
        cmd.arg("build")
            .arg("--cap-add=all")
            .arg("--security-opt=label=type:container_runtime_t")
            .arg("--device")
            .arg("/dev/fuse")
            .arg("--network=host")
            .arg("-f")
            .arg("Containerfile")
            .arg("-t")
            .arg(format!("{image}:latest"));

        if let Some(ver) = self.version.as_deref() {
            cmd.arg("-t").arg(format!("{image}:{ver}"));
        }

        let status = cmd.arg(".").current_dir(self.build_dir.path()).status()?;

        if !status.success() {
            return Err(ReceiptError::PodmanBuildFailed(status.code().unwrap_or(-1)));
        }

        Ok(())
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
        let builder = Builder::from_receipt(receipt_fixture(), build_dir, None).unwrap();
        builder.write_containerfile().unwrap();
        assert!(path.join("Containerfile").exists());
    }

    #[test]
    fn write_containerfile_content_matches() {
        let build_dir = BuildDir::temp().unwrap();
        let builder = Builder::from_receipt(receipt_fixture(), build_dir, None).unwrap();
        let file_path = builder.write_containerfile().unwrap();
        let written = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, builder.to_containerfile());
    }
}
