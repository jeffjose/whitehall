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
    Build {
        /// Path to whitehall.toml (defaults to ./whitehall.toml)
        #[arg(long, default_value = "whitehall.toml")]
        manifest_path: String,
    },
    /// Watch for changes and rebuild automatically
    Watch {
        /// Path to whitehall.toml (defaults to ./whitehall.toml)
        #[arg(long, default_value = "whitehall.toml")]
        manifest_path: String,
    },
    /// Build, install, and run the app on a connected device
    Run {
        /// Path to whitehall.toml (defaults to ./whitehall.toml)
        #[arg(long, default_value = "whitehall.toml")]
        manifest_path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            commands::init::execute(&name)?;
        }
        Commands::Build { manifest_path } => {
            commands::build::execute(&manifest_path)?;
        }
        Commands::Watch { manifest_path } => {
            commands::watch::execute(&manifest_path)?;
        }
        Commands::Run { manifest_path } => {
            commands::run::execute(&manifest_path)?;
        }
    }

    Ok(())
}
