use clap::{Parser, Subcommand};
use colored::Colorize;
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
        /// Custom package name (default: com.example.app)
        #[arg(long)]
        package: Option<String>,
        /// Omit package declaration (for pasting into existing files)
        #[arg(long)]
        no_package: bool,
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

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name } => {
            commands::init::execute(&name)
        }
        Commands::Compile { file, package, no_package } => {
            commands::compile::execute(&file, package.as_deref(), no_package)
        }
        Commands::Build { target } => {
            commands::build::execute(&target)
        }
        Commands::Watch { target } => {
            commands::watch::execute(&target)
        }
        Commands::Run { target } => {
            commands::run::execute(&target)
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}
