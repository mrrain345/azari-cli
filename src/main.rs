use azari_cli::cli::Cli;
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.command.run(&cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
