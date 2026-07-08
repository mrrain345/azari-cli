#[allow(clippy::module_inception)]
mod builder;
pub(crate) mod command;
mod error;
pub mod metadata;
pub mod stage;
pub(crate) mod utils;

pub use builder::{Builder, BuilderOptions};
pub use error::BuildError;
pub use metadata::ImageMetadata;
pub use stage::BuilderStage;

/// Trait for building a Containerfile from a recipe field.
pub trait Build {
    fn build(self, builder: &mut Builder) -> Result<(), BuildError>;
}
