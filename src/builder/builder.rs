use tempfile::TempDir;

use crate::builder::utils::{get_timestamp_str, make_build_dir};
use crate::distro::Distro;
use crate::receipt::{Receipt, ReceiptError};

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
    name: Option<String>,
    base_image: Option<String>,
    created: String,
    lines: Vec<String>,
    build_dir: TempDir,
}

/// Options for [`Builder::from_receipt_with`].
#[derive(Debug, Default)]
pub struct BuilderOptions {
    pub version: Option<String>,
    pub build_dir: Option<std::path::PathBuf>,
    pub image: Option<String>,
}

/// Maximum number of layers to allow when rechunking with chunkah.
const CHUNKAH_MAX_LAYERS: usize = 128;

impl Builder {
    /// Builds containerfile lines from a receipt.
    pub fn from_receipt(receipt: Receipt) -> Result<Self, ReceiptError> {
        Self::from_receipt_with(receipt, BuilderOptions::default())
    }

    /// Builds containerfile lines from a receipt with additional options.
    pub fn from_receipt_with(
        receipt: Receipt,
        options: BuilderOptions,
    ) -> Result<Self, ReceiptError> {
        let mut builder = Builder {
            distro: None,
            image: options.image,
            version: options.version,
            name: None,
            base_image: None,
            created: get_timestamp_str(),
            lines: Vec::new(),
            build_dir: make_build_dir(options.build_dir)?,
        };

        receipt.build(&mut builder)?;
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

    /// Stores the image pretty name for use in OCI annotations.
    pub(crate) fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// Stores the resolved base image reference (the FROM image).
    pub(crate) fn set_base_image(&mut self, base_image: String) {
        self.base_image = Some(base_image);
    }

    /// Returns the image ref name.
    pub fn image(&self) -> Result<&str, ReceiptError> {
        self.image.as_deref().ok_or(ReceiptError::ImageNotSpecified)
    }

    /// Returns the version.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Returns the pretty name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the resolved base image reference.
    pub fn base_image(&self) -> Option<&str> {
        self.base_image.as_deref()
    }

    /// Returns the RFC-3339 timestamp captured when the receipt was processed.
    pub fn created(&self) -> &str {
        &self.created
    }

    /// Returns the OCI labels key-value pairs to embed in the image.
    pub(crate) fn oci_labels(&self) -> Vec<(&'static str, String)> {
        let image = self.image();
        let version = self.version();

        let mut pairs = vec![("org.opencontainers.image.created", self.created.clone())];

        if let Some(version) = version {
            pairs.push(("org.opencontainers.image.version", version.to_owned()));

            if let Ok(image) = image {
                let ref_name = format!("{image}:{version}");
                pairs.push(("org.opencontainers.image.ref.name", ref_name));
            }
        } else if let Ok(image) = image {
            pairs.push(("org.opencontainers.image.ref.name", image.to_owned()));
        }

        if let Some(name) = &self.name {
            pairs.push(("org.opencontainers.image.title", name.clone()));
        }

        if let Some(base_image) = &self.base_image {
            pairs.push(("org.opencontainers.image.base.name", base_image.clone()));
        }

        pairs
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

    /// Appends final instructions to the Containerfile.
    pub fn add_trailer(&mut self, rechunk: bool) {
        self.push("RUN bootc container lint --no-truncate");

        if rechunk {
            self.push("");
            self.push("FROM quay.io/coreos/chunkah AS chunkah");
            self.push(format!(
                "RUN {} \\\n  {} \\\n  {}",
                "--mount=type=bind,target=/usr/lib/azari/chunkah,rw",
                "--mount=from=builder,target=/chunkah,ro",
                format_args!("chunkah build --max-layers {CHUNKAH_MAX_LAYERS} --output oci:/usr/lib/azari/chunkah/out")
            ));
            self.push("");
            self.push("FROM oci:chunkah/out");
        }

        let mut labels = self
            .oci_labels()
            .iter()
            .map(|(k, v)| format!(r#""{k}"="{v}""#))
            .collect::<Vec<_>>();

        labels.push(r#""containers.bootc"="1""#.into());
        labels.push(r#""azari.managed"="true""#.into());

        self.push(format!("LABEL {}", labels.join(" \\\n    "),));
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::receipt::Receipt;

    fn receipt_fixture() -> Receipt {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/receipts/builder.yaml");
        Receipt::from_file(&path).unwrap()
    }

    #[test]
    fn write_containerfile_creates_file() {
        let builder = Builder::from_receipt(receipt_fixture()).unwrap();
        let path = builder.write_containerfile().unwrap();
        assert!(path.exists());
    }

    #[test]
    fn write_containerfile_content_matches() {
        let builder = Builder::from_receipt(receipt_fixture()).unwrap();
        let file_path = builder.write_containerfile().unwrap();
        let written = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, builder.to_containerfile());
    }
}
