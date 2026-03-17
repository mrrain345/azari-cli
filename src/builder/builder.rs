use crate::distro::Distro;
use crate::receipt::{Receipt, ReceiptError};

/// Trait for building a Containerfile from a receipt field.
pub trait Build {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError>;
}

/// In-memory Containerfile builder.
#[derive(Debug, Default)]
pub struct Builder {
    distro: Option<Distro>,
    lines: Vec<String>,
}

impl Builder {
    /// Builds containerfile lines from a receipt.
    pub fn from_receipt(receipt: Receipt) -> Result<Self, ReceiptError> {
        let mut builder = Builder::default();

        // `distro` must be built first — it populates `builder.distro`,
        // which other fields read from during their build step.
        receipt.distro.build(&mut builder)?;
        receipt.from.build(&mut builder)?;
        receipt.name.build(&mut builder)?;
        receipt.hostname.build(&mut builder)?;
        receipt.packages.build(&mut builder)?;

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

    /// Renders all instruction lines into a single Containerfile string.
    pub fn to_containerfile(&self) -> String {
        if self.lines.is_empty() {
            return String::new();
        }

        let mut out = self.lines.join("\n");
        out.push('\n');
        out
    }
}
