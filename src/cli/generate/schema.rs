use std::path::PathBuf;

use clap::Args;
use schemars::{generate::SchemaSettings, transform::AddNullable};

use crate::{builder::BuildError, recipe::Recipe};

use super::{resolve_path, write_output};

const INSTALL_PATH: &str = "/usr/lib/azari/schema.json";

#[derive(Debug, Args)]
pub struct GenerateSchemaArgs {
    /// Output path (defaults to stdout)
    #[arg(value_name = "PATH", conflicts_with = "install")]
    pub path: Option<PathBuf>,

    /// Install schema to system path (requires root privileges)
    #[arg(short, long)]
    pub install: bool,
}

impl GenerateSchemaArgs {
    pub fn run(self) -> Result<(), BuildError> {
        let content = generate_schema()?;

        let path = if self.install {
            Some(PathBuf::from(INSTALL_PATH))
        } else {
            resolve_path(self.path.as_ref(), "schema.json")
        };

        write_output(content.as_bytes(), path)
    }
}

fn generate_schema() -> Result<String, BuildError> {
    let settings = SchemaSettings::draft07().with(|s| {
        s.inline_subschemas = true;
        s.transforms = vec![Box::new(AddNullable::default())];
    });

    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<Recipe>();
    let mut content = serde_json::to_string_pretty(&schema).unwrap();
    content.push('\n');

    Ok(content)
}
