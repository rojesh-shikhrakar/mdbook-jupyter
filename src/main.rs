use clap::{Parser, Subcommand};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_jupyter::cli;
use mdbook_jupyter::JupyterPreprocessor;
use std::io;
use std::process;

#[derive(Parser)]
#[clap(
    name = "mdbook-jupyter",
    about = "mdbook preprocessor for Jupyter notebooks",
    version
)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Install the preprocessor into book.toml
    Install,
    /// Check if the preprocessor supports a given renderer
    Supports { renderer: String },
}

fn main() {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Command::Install => {
                if let Err(e) = cli::handle_install() {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
            Command::Supports { renderer } => {
                let preprocessor = JupyterPreprocessor::new();
                let supported = cli::handle_supports(&preprocessor, &renderer);
                process::exit(if supported { 0 } else { 1 });
            }
        }
    } else if let Err(e) = handle_preprocessing() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn handle_preprocessing() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
    cli::check_version_compatibility(&ctx.mdbook_version)?;

    let preprocessor = JupyterPreprocessor::new();
    let processed_book = preprocessor.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
