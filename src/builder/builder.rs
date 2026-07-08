use tempfile::TempDir;

use crate::builder::Build;
use crate::builder::error::BuildError;
use crate::builder::metadata::ImageMetadata;
use crate::builder::stage::BuilderStage;
use crate::builder::utils::make_build_dir;
use crate::distro::Distro;
use crate::recipe::Recipe;

/// Maximum number of layers to allow when rechunking with chunkah.
const CHUNKAH_MAX_LAYERS: usize = 128;

/// In-memory Containerfile builder.
#[derive(Debug)]
pub struct Builder {
    /// Base distro for the recipe.
    distro: Option<Distro>,
    /// Metadata for the output image.
    meta: ImageMetadata,
    /// Stages of the Containerfile.
    stages: Vec<BuilderStage>,
    /// Build directory for all files included in the Containerfile.
    build_dir: TempDir,
}

/// Options for [`Builder::from_recipe_with`].
#[derive(Debug, Default)]
pub struct BuilderOptions {
    /// Override the output image name.
    pub output_image: Option<String>,
    /// Override the version.
    pub version: Option<String>,
    /// Override the build directory.
    pub build_dir: Option<std::path::PathBuf>,
}

impl Builder {
    /// Builds containerfile from a recipe.
    pub fn from_recipe(recipe: Recipe) -> Result<Self, BuildError> {
        Self::from_recipe_with(recipe, BuilderOptions::default())
    }

    /// Builds containerfile from a recipe with additional options.
    pub fn from_recipe_with(recipe: Recipe, options: BuilderOptions) -> Result<Self, BuildError> {
        let mut builder = Builder {
            distro: None,
            meta: ImageMetadata::default(),
            stages: vec![BuilderStage::default()],
            build_dir: make_build_dir(options.build_dir)?,
        };

        recipe.build(&mut builder)?;

        // Apply image override
        if let Some(image) = options.output_image {
            builder.meta_mut().set_output_image(image);
        }

        // Apply version override
        if let Some(version) = options.version {
            builder.meta_mut().set_version(version);
        }

        Ok(builder)
    }

    /// Stores the resolved distro.
    ///
    /// Must be called before [`Builder::distro`].
    pub fn set_distro(&mut self, distro: Distro) {
        self.distro = Some(distro);
    }

    /// Returns a reference to the image metadata.
    pub fn meta(&self) -> &ImageMetadata {
        &self.meta
    }

    /// Returns a mutable reference to the image metadata.
    pub fn meta_mut(&mut self) -> &mut ImageMetadata {
        &mut self.meta
    }

    /// Returns a reference to the current stage.
    pub fn current(&self) -> &BuilderStage {
        self.stages.last().expect("stages is never empty")
    }

    /// Returns the mutable reference to the current stage.
    pub fn current_mut(&mut self) -> &mut BuilderStage {
        self.stages.last_mut().expect("stages is never empty")
    }

    /// Returns the auto-generated name for stage at `index` (1-based: "stage_1", "stage_2", …).
    fn stage_name(index: usize) -> String {
        format!("stage_{}", index + 1)
    }

    /// Returns the resolved distro.
    ///
    /// [`Builder::set_distro`] must have been called beforehand.
    pub fn distro(&self) -> Result<Distro, BuildError> {
        self.distro.ok_or(BuildError::DistroNotSpecified)
    }

    /// Appends a new stage with the given `from` image reference.
    pub fn add_stage(&mut self, from: String) {
        let mut stage = BuilderStage::default();
        stage.set_from(from);
        self.stages.push(stage);
    }

    /// Sets the FROM image reference on the current stage.
    pub fn set_stage_from(&mut self, from: String) {
        self.current_mut().set_from(from);
    }

    /// Appends a single Containerfile instruction line.
    pub fn push(&mut self, line: impl Into<String>) {
        self.current_mut().push(line);
    }

    /// Appends an early Containerfile instruction line.
    pub fn push_early(&mut self, line: impl Into<String>) {
        self.current_mut().push_early(line);
    }

    /// Appends a late Containerfile instruction line.
    pub fn push_late(&mut self, line: impl Into<String>) {
        self.current_mut().push_late(line);
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
            let stage_name = Self::stage_name(self.stages.len() - 1);
            self.add_stage("quay.io/coreos/chunkah".into());
            self.push(format!(
                "RUN {} \\\n  {} \\\n  {}",
                "--mount=type=bind,target=/usr/lib/azari/chunkah,rw",
                format_args!("--mount=from={stage_name},target=/chunkah,ro"),
                format_args!("chunkah build --max-layers {CHUNKAH_MAX_LAYERS} --output oci:/usr/lib/azari/chunkah/out")
            ));
            self.add_stage("oci:chunkah/out".into());
        }

        let labels = self
            .meta
            .oci_labels()
            .iter()
            .map(|(k, v)| format!(r#""{k}"="{v}""#))
            .chain([
                r#""containers.bootc"="1""#.into(),
                r#""azari.managed"="true""#.into(),
            ])
            .collect::<Vec<_>>();

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
