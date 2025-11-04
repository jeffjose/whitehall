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
    /// Transpile a single .wh file to Kotlin (no Android project generation)
    Compile {
        /// Path to .wh file
        file: String,
    },
    /// Build the project (transpile .wh files to Kotlin + generate Android project)
    /// Works with both project directories (whitehall.toml) and single .wh files
    Build {
        /// Path to project directory or .wh file (defaults to current directory)
        #[arg(default_value = ".")]
        target: String,
    },
    /// Watch for changes and rebuild automatically
    /// Works with both project directories (whitehall.toml) and single .wh files
    Watch {
        /// Path to project directory or .wh file (defaults to current directory)
        #[arg(default_value = ".")]
        target: String,
    },
    /// Build, install, and run the app on a connected device
    /// Works with both project directories (whitehall.toml) and single .wh files
    Run {
        /// Path to project directory or .wh file (defaults to current directory)
        #[arg(default_value = ".")]
        target: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            commands::init::execute(&name)?;
        }
        Commands::Compile { file } => {
            commands::compile::execute(&file)?;
        }
        Commands::Build { target } => {
            commands::build::execute(&target)?;
        }
        Commands::Watch { target } => {
            commands::watch::execute(&target)?;
        }
        Commands::Run { target } => {
            commands::run::execute(&target)?;
        }
    }

    Ok(())
}
