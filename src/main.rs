use azari::{cli::Cli, recipe::RecipeError};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    cli.command.run(&cli).unwrap_or_else(|e| {
        print_error(&e);
        std::process::exit(1);
    });
}

fn print_error(error: &RecipeError) {
    match error {
        RecipeError::Aggregate(errors) => {
            for error in errors {
                print_error(error);
            }
        }
        _ => eprintln!("ERROR: {error}"),
    }
}
