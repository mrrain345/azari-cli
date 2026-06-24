use clap::Args;

use crate::recipe::{Recipe, RecipeError};

use super::Cli;

#[derive(Debug, Args)]
pub struct SchemaArgs {}

impl SchemaArgs {
    pub fn run(&self, _cli: &Cli) -> Result<(), RecipeError> {
        let schema = schemars::schema_for!(Recipe);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        Ok(())
    }
}
