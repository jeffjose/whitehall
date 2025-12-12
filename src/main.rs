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
    /// Transpile to Kotlin (no APK build)
    /// Works with both project directories (whitehall.toml) and single .wh files
    Compile {
        /// Path to project directory or .wh file (defaults to current directory)
        #[arg(default_value = ".")]
        target: String,
        /// Custom package name (default: com.example.app) - only for single files
        #[arg(long)]
        package: Option<String>,
        /// Omit package declaration (for pasting) - only for single files
        #[arg(long)]
        no_package: bool,
    },
    /// Transpile + build APK
    /// Works with both project directories (whitehall.toml) and single .wh files
    Build {
        /// Path to project directory or .wh file (defaults to current directory)
        #[arg(default_value = ".")]
        target: String,
        /// Watch for changes and rebuild automatically
        #[arg(long, short)]
        watch: bool,
    },
    /// Watch for changes and rebuild automatically
    /// Works with both project directories (whitehall.toml) and single .wh files
    Watch {
        /// Path to project directory or .wh file (defaults to current directory)
        #[arg(default_value = ".")]
        target: String,
    },
    /// Build and install the app on a connected device (without launching)
    /// Works with both project directories (whitehall.toml) and single .wh files
    Install {
        /// Path to project directory or .wh file (defaults to current directory)
        #[arg(default_value = ".")]
        target: String,
        /// Device ID (partial match supported) - auto-selects if only one device connected
        device: Option<String>,
    },
    /// Build, install, and run the app on a connected device
    /// Works with both project directories (whitehall.toml) and single .wh files
    Run {
        /// Path to project directory or .wh file (defaults to current directory)
        #[arg(default_value = ".")]
        target: String,
        /// Device ID (partial match supported) - auto-selects if only one device connected
        device: Option<String>,
    },
    /// Manage toolchains (Java, Gradle, Android SDK)
    Toolchain {
        #[command(subcommand)]
        command: ToolchainCommands,
    },
    /// Manage Android emulators
    Emulator {
        #[command(subcommand)]
        command: Option<EmulatorCommands>,
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(long, default_value = "whitehall.toml")]
        manifest: String,
    },
    /// List connected devices
    #[command(visible_alias = "devices")]
    Device,
    /// Execute a command with the project's toolchain environment
    Exec {
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(long, default_value = "whitehall.toml")]
        manifest: String,
        /// Command and arguments to execute
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
    /// Find the path to a command in the toolchain environment (alias for 'exec which')
    Which {
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(long, default_value = "whitehall.toml")]
        manifest: String,
        /// Command to locate
        command: String,
    },
    /// Launch an interactive shell with the project's toolchain environment
    Shell {
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(long, default_value = "whitehall.toml")]
        manifest: String,
    },
    /// Check system health and toolchain status
    Doctor {
        /// Path to whitehall.toml (defaults to current directory, optional)
        #[arg(default_value = "whitehall.toml")]
        manifest: String,
    },
}

#[derive(Subcommand)]
enum ToolchainCommands {
    /// Install toolchains required by the current project (Java, Gradle, Android SDK, Emulator, System Images)
    Install {
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(default_value = "whitehall.toml")]
        manifest: String,
    },
    /// List installed toolchains
    List,
    /// Remove all installed toolchains
    Clean,
    /// Execute a command with the project's toolchain environment
    Exec {
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(long, default_value = "whitehall.toml")]
        manifest: String,
        /// Command to execute
        command: String,
        /// Arguments to pass to the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Launch an interactive shell with the project's toolchain environment
    Shell {
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(default_value = "whitehall.toml")]
        manifest: String,
    },
}

#[derive(Subcommand)]
enum EmulatorCommands {
    /// List available emulators
    #[command(visible_alias = "ls")]
    List {
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(default_value = "whitehall.toml")]
        manifest: String,
    },
    /// Start an emulator
    Start {
        /// Name of the emulator to start
        name: String,
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(long, default_value = "whitehall.toml")]
        manifest: String,
    },
    /// Create a new emulator
    Create {
        /// Name for the new emulator (defaults to 'whitehall')
        name: Option<String>,
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(long, default_value = "whitehall.toml")]
        manifest: String,
    },
    /// Delete an emulator
    #[command(visible_aliases = ["remove", "rm"])]
    Delete {
        /// ID or name of the emulator to delete
        name: String,
        /// Path to whitehall.toml (defaults to current directory)
        #[arg(long, default_value = "whitehall.toml")]
        manifest: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name } => {
            commands::init::execute(&name)
        }
        Commands::Compile { target, package, no_package } => {
            commands::compile::execute(&target, package.as_deref(), no_package)
        }
        Commands::Build { target, watch } => {
            commands::build::execute(&target, watch)
        }
        Commands::Watch { target } => {
            commands::watch::execute(&target)
        }
        Commands::Install { target, device } => {
            commands::install::execute(&target, device.as_deref())
        }
        Commands::Run { target, device } => {
            commands::run::execute(&target, device.as_deref())
        }
        Commands::Toolchain { command } => {
            match command {
                ToolchainCommands::Install { manifest } => {
                    commands::toolchain::execute_install(&manifest)
                }
                ToolchainCommands::List => {
                    commands::toolchain::execute_list()
                }
                ToolchainCommands::Clean => {
                    commands::toolchain::execute_clean()
                }
                ToolchainCommands::Exec { manifest, command, args } => {
                    commands::toolchain::execute_exec(&manifest, &command, &args)
                }
                ToolchainCommands::Shell { manifest } => {
                    commands::toolchain::execute_shell(&manifest)
                }
            }
        }
        Commands::Emulator { command, manifest } => {
            match command {
                None => {
                    commands::emulator::execute_list(&manifest)
                }
                Some(EmulatorCommands::List { manifest }) => {
                    commands::emulator::execute_list(&manifest)
                }
                Some(EmulatorCommands::Start { name, manifest }) => {
                    commands::emulator::execute_start(&manifest, &name)
                }
                Some(EmulatorCommands::Create { name, manifest }) => {
                    commands::emulator::execute_create(&manifest, name.as_deref())
                }
                Some(EmulatorCommands::Delete { name, manifest }) => {
                    commands::emulator::execute_delete(&manifest, &name)
                }
            }
        }
        Commands::Device => {
            commands::device::execute_list()
        }
        Commands::Exec { manifest, command } => {
            if command.is_empty() {
                eprintln!("{} No command specified", "error:".red().bold());
                std::process::exit(1);
            }
            commands::toolchain::execute_exec(&manifest, &command[0], &command[1..])
        }
        Commands::Which { manifest, command } => {
            // Alias for 'exec which'
            commands::toolchain::execute_exec(&manifest, "which", &[command])
        }
        Commands::Shell { manifest } => {
            commands::toolchain::execute_shell(&manifest)
        }
        Commands::Doctor { manifest } => {
            commands::doctor::execute(&manifest)
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}
