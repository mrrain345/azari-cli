use clap::Args;
use schemars::{generate::SchemaSettings, transform::AddNullable};

use crate::builder::BuildError;
use crate::recipe::Recipe;

use super::Cli;

#[derive(Debug, Args)]
pub struct SchemaArgs {}

impl SchemaArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), BuildError> {
        let settings = SchemaSettings::draft07().with(|s| {
            s.inline_subschemas = true;
            s.transforms = vec![Box::new(AddNullable::default())];
        });

        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<Recipe>();
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());

        Ok(())
    }
}
