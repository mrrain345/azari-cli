use crate::builder::error::BuildError;
use crate::builder::utils::get_timestamp_str;

/// OCI image metadata collected while processing a recipe.
#[derive(Debug)]
pub struct ImageMetadata {
    output_image: Option<String>,
    version: Option<String>,
    pretty_name: Option<String>,
    base_image: Option<String>,
    created: String,
}

impl ImageMetadata {
    /// Stores the output image name.
    pub fn set_output_image(&mut self, image: String) {
        self.output_image = Some(image);
    }

    /// Stores the version.
    pub fn set_version(&mut self, version: String) {
        self.version = Some(version);
    }

    /// Stores the image pretty name for use in OCI annotations.
    pub fn set_pretty_name(&mut self, name: String) {
        self.pretty_name = Some(name);
    }

    /// Stores the resolved base image reference (the FROM image).
    pub fn set_base_image(&mut self, base_image: String) {
        self.base_image = Some(base_image);
    }

    /// Returns the output image name.
    pub fn output_image(&self) -> Result<&str, BuildError> {
        self.output_image
            .as_deref()
            .ok_or(BuildError::ImageNotSpecified)
    }

    /// Returns the version.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Returns the pretty name.
    pub fn pretty_name(&self) -> Option<&str> {
        self.pretty_name.as_deref()
    }

    /// Returns the resolved base image reference.
    pub fn base_image(&self) -> Option<&str> {
        self.base_image.as_deref()
    }

    /// Returns the RFC-3339 timestamp captured when the recipe was processed.
    pub fn created(&self) -> &str {
        &self.created
    }

    /// Returns the OCI labels key-value pairs to embed in the image.
    pub fn oci_labels(&self) -> Vec<(&'static str, String)> {
        let image = self.output_image();
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

        if let Some(name) = &self.pretty_name {
            pairs.push(("org.opencontainers.image.title", name.clone()));
        }

        if let Some(base_image) = &self.base_image {
            pairs.push(("org.opencontainers.image.base.name", base_image.clone()));
        }

        pairs
    }
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self {
            output_image: None,
            version: None,
            pretty_name: None,
            base_image: None,
            created: get_timestamp_str(),
        }
    }
}
