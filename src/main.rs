use clap::{Parser, Subcommand};
use anyhow::Result;
use whitehall::commands;

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
    /// Build the project (transpile .wh files to Kotlin + generate Android project)
    Build,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            commands::init::execute(&name)?;
        }
        Commands::Build => {
            commands::build::execute()?;
        }
    }

    Ok(())
}
