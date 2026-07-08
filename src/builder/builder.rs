use tempfile::TempDir;

use crate::builder::error::BuildError;
use crate::builder::stage::BuilderStage;
use crate::builder::utils::{get_timestamp_str, make_build_dir};
use crate::distro::Distro;
use crate::recipe::Recipe;

/// Trait for building a Containerfile from a recipe field.
pub trait Build {
    fn build(self, builder: &mut Builder) -> Result<(), BuildError>;
}

/// In-memory Containerfile builder.
#[derive(Debug)]
pub struct Builder {
    distro: Option<Distro>,
    image: Option<String>,
    version: Option<String>,
    pretty_name: Option<String>,
    base_image: Option<String>,
    created: String,
    stages: Vec<BuilderStage>,
    build_dir: TempDir,
}

/// Options for [`Builder::from_recipe_with`].
#[derive(Debug, Default)]
pub struct BuilderOptions {
    pub version: Option<String>,
    pub build_dir: Option<std::path::PathBuf>,
    pub image: Option<String>,
}

/// Maximum number of layers to allow when rechunking with chunkah.
const CHUNKAH_MAX_LAYERS: usize = 128;

impl Builder {
    /// Builds containerfile from a recipe.
    pub fn from_recipe(recipe: Recipe) -> Result<Self, BuildError> {
        Self::from_recipe_with(recipe, BuilderOptions::default())
    }

    /// Builds containerfile from a recipe with additional options.
    pub fn from_recipe_with(recipe: Recipe, options: BuilderOptions) -> Result<Self, BuildError> {
        let mut builder = Builder {
            distro: None,
            image: options.image,
            version: options.version,
            pretty_name: None,
            base_image: None,
            created: get_timestamp_str(),
            stages: vec![BuilderStage::default()],
            build_dir: make_build_dir(options.build_dir)?,
        };

        recipe.build(&mut builder)?;
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
    pub(crate) fn set_pretty_name(&mut self, name: String) {
        self.pretty_name = Some(name);
    }

    /// Stores the resolved base image reference (the FROM image).
    pub(crate) fn set_base_image(&mut self, base_image: String) {
        self.base_image = Some(base_image);
    }

    /// Returns the image ref name.
    pub fn image(&self) -> Result<&str, BuildError> {
        self.image.as_deref().ok_or(BuildError::ImageNotSpecified)
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

        if let Some(name) = &self.pretty_name {
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
    pub(crate) fn distro(&self) -> Result<Distro, BuildError> {
        self.distro.ok_or(BuildError::DistroNotSpecified)
    }

    /// Returns the mutable reference to the current (last) stage.
    fn current_stage_mut(&mut self) -> &mut BuilderStage {
        self.stages.last_mut().expect("stages is never empty")
    }

    /// Returns the auto-generated name for stage at `index` (1-based: "stage_1", "stage_2", …).
    fn stage_name(index: usize) -> String {
        format!("stage_{}", index + 1)
    }

    /// Appends a new stage with the given `from` image reference.
    pub(crate) fn add_stage(&mut self, from: String) {
        let mut stage = BuilderStage::default();
        stage.set_from(from);
        self.stages.push(stage);
    }

    /// Sets the FROM image reference on the current stage.
    pub(crate) fn set_stage_from(&mut self, from: String) {
        self.current_stage_mut().set_from(from);
    }

    /// Appends a single Containerfile instruction line.
    pub(crate) fn push(&mut self, line: impl Into<String>) {
        self.current_stage_mut().push(line);
    }

    /// Appends an early Containerfile instruction line.
    pub(crate) fn push_early(&mut self, line: impl Into<String>) {
        self.current_stage_mut().push_early(line);
    }

    /// Appends a late Containerfile instruction line.
    pub(crate) fn push_late(&mut self, line: impl Into<String>) {
        self.current_stage_mut().push_late(line);
    }

    /// Returns the path to the build directory.
    pub fn build_dir(&self) -> &std::path::Path {
        self.build_dir.path()
    }

    /// Renders all stages into a single Containerfile string.
    pub fn to_containerfile(&self) -> String {
        self.stages
            .iter()
            .enumerate()
            .map(|(i, stage)| stage.to_containerfile(&Self::stage_name(i)))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Writes the Containerfile to `<build_dir>/Containerfile` and returns
    /// the path to the written file.
    pub fn write_containerfile(&self) -> Result<std::path::PathBuf, BuildError> {
        let path = self.build_dir.path().join("Containerfile");
        std::fs::write(&path, self.to_containerfile())?;
        Ok(path)
    }

    /// Appends final instructions to the Containerfile.
    pub fn add_trailer(&mut self, rechunk: bool) {
        self.push("RUN bootc container lint --no-truncate");

        if rechunk {
            let builder_stage_name = Self::stage_name(self.stages.len() - 1);
            self.add_stage("quay.io/coreos/chunkah".into());
            self.push(format!(
                "RUN {} \\\n  {} \\\n  {}",
                "--mount=type=bind,target=/usr/lib/azari/chunkah,rw",
                format_args!("--mount=from={builder_stage_name},target=/chunkah,ro"),
                format_args!("chunkah build --max-layers {CHUNKAH_MAX_LAYERS} --output oci:/usr/lib/azari/chunkah/out")
            ));
            self.add_stage("oci:chunkah/out".into());
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
    use crate::recipe::Recipe;

    fn recipe_fixture() -> Recipe {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/recipes/builder.yaml");
        Recipe::from_file(&path).unwrap()
    }

    #[test]
    fn write_containerfile_creates_file() {
        let builder = Builder::from_recipe(recipe_fixture()).unwrap();
        let path = builder.write_containerfile().unwrap();
        assert!(path.exists());
    }

    #[test]
    fn write_containerfile_content_matches() {
        let builder = Builder::from_recipe(recipe_fixture()).unwrap();
        let file_path = builder.write_containerfile().unwrap();
        let written = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, builder.to_containerfile());
    }
}
