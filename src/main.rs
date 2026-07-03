use std::io::{self, IsTerminal};

use azari::{builder::BuildError, cli::Cli, recipe::RecipeError};
use clap::Parser;
use colored::Colorize;

fn main() {
    // Disable colored output if stderr is not a terminal
    if !io::stderr().is_terminal() {
        colored::control::set_override(false);
    }

    let cli = Cli::parse();

    cli.run().unwrap_or_else(|e| {
        print_error(&e);
        std::process::exit(1);
    });
}

fn print_error(error: &BuildError) {
    match error {
        BuildError::Recipe(recipe_error) => print_recipe_error(recipe_error),
        _ => eprintln!("{}\n", error.to_string().red()),
    }
}

fn print_recipe_error(error: &RecipeError) {
    match error {
        RecipeError::Aggregate(errors) => {
            for error in errors {
                print_recipe_error(error);
            }
        }
        _ => eprintln!("{}\n", error.to_string().red()),
    }
}
