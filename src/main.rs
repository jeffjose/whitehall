use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;

#[derive(Parser)]
#[command(name = "whitehall")]
#[command(about = "A unified Rust toolchain for Android app development", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Whitehall project
    Init {
        /// Name of the project
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            commands::init::execute(&name)?;
        }
    }

    Ok(())
}
