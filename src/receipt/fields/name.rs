use std::path::PathBuf;

use serde::Deserialize;

use crate::builder::{Build, Builder};
use crate::receipt::error::ReceiptError;
use crate::receipt::field::ReceiptField;
use crate::receipt::unique::ReceiptUnique;

/// Field for the `name` key.
///
/// Sets the image name and version labels,
/// and updates the `NAME`, `VERSION`, and `PRETTY_NAME` fields in `/etc/os-release`.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct NameField(pub(crate) ReceiptUnique<String>);

impl ReceiptField for NameField {
    type Value = Option<String>;

    fn value(self) -> Result<Self::Value, ReceiptError> {
        self.0.value()
    }

    fn sources(&self) -> &[PathBuf] {
        self.0.sources()
    }

    fn merge(self, other: Self) -> Self {
        Self(self.0.merge(other.0))
    }
}

impl Build for NameField {
    fn build(self, builder: &mut Builder) -> Result<(), ReceiptError> {
        if let Some(name) = self.value()? {
            let version = builder.version().map(str::to_owned);
            let pretty = match &version {
                Some(v) => format!("{name} {v}"),
                None => name.clone(),
            };

            builder.push(format!(
                "LABEL org.opencontainers.image.title=\"{}\"",
                escape(&name)
            ));

            if let Some(v) = &version {
                builder.push(format!(
                    "LABEL org.opencontainers.image.version=\"{}\"",
                    escape(v)
                ));
            }

            builder.push(os_release_sed("NAME", &name));
            if let Some(v) = &version {
                builder.push(os_release_sed("VERSION", v));
            }
            builder.push(os_release_sed("PRETTY_NAME", &pretty));
        }
        Ok(())
    }
}

/// Produces a `RUN sed -i …` instruction that updates `KEY="value"` in
/// `/etc/os-release` in-place, or appends it if the key is not present.
fn os_release_sed(key: &str, value: &str) -> String {
    let value_e = escape(value);
    format!(
        "RUN sed -i '/^{key}=/{{s/.*/{key}=\"{value_e}\"/;:a;n;ba}};$a{key}=\"{value_e}\"' /etc/os-release"
    )
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
