use anyhow::{Context, Result};
use colored::Colorize;

use crate::config;
use crate::toolchain::Toolchain;

/// Install toolchains required by the current project
pub fn execute_install(manifest_path: &str) -> Result<()> {
    // Load project config to get toolchain requirements
    let config = config::load_config(manifest_path)?;

    // Initialize toolchain manager
    let toolchain = Toolchain::new()?;

    // Download all toolchains in parallel for faster installation
    toolchain.ensure_all_parallel(
        &config.toolchain.java,
        &config.toolchain.gradle
    )?;

    Ok(())
}

/// List installed toolchains
pub fn execute_list() -> Result<()> {
    let toolchain = Toolchain::new()?;
    let root = toolchain.root();

    println!("{} toolchains", "Installed".green().bold());
    println!("Location: {}\n", root.display());

    // List Java versions
    let java_dir = root.join("java");
    if java_dir.exists() {
        println!("{}:", "Java".cyan().bold());
        for entry in std::fs::read_dir(&java_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let version = entry.file_name().to_string_lossy().to_string();
                let size = get_dir_size(&entry.path())?;
                println!("  - {} ({})", version, human_size(size));
            }
        }
    } else {
        println!("{}: None installed", "Java".cyan().bold());
    }

    // List Gradle versions
    let gradle_dir = root.join("gradle");
    if gradle_dir.exists() {
        println!("\n{}:", "Gradle".cyan().bold());
        for entry in std::fs::read_dir(&gradle_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let version = entry.file_name().to_string_lossy().to_string();
                let size = get_dir_size(&entry.path())?;
                println!("  - {} ({})", version, human_size(size));
            }
        }
    } else {
        println!("\n{}: None installed", "Gradle".cyan().bold());
    }

    // Android SDK
    let android_dir = root.join("android");
    if android_dir.exists() {
        let size = get_dir_size(&android_dir)?;
        println!("\n{}:", "Android SDK".cyan().bold());
        println!("  - Installed ({})", human_size(size));
    } else {
        println!("\n{}: Not installed", "Android SDK".cyan().bold());
    }

    Ok(())
}

/// Clean (remove) all installed toolchains
pub fn execute_clean() -> Result<()> {
    let toolchain = Toolchain::new()?;
    let root = toolchain.root();

    println!("{} Do you want to delete all toolchains?", "Warning:".yellow().bold());
    println!("Location: {}", root.display());
    print!("\nType 'yes' to confirm: ");

    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim() != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    println!("\n{} toolchains...", "Removing".red().bold());

    if root.exists() {
        std::fs::remove_dir_all(root)?;
        println!("{} All toolchains removed", "Success:".green().bold());
    } else {
        println!("No toolchains directory found.");
    }

    Ok(())
}

/// Get directory size recursively
fn get_dir_size(path: &std::path::Path) -> Result<u64> {
    let mut size = 0;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                size += get_dir_size(&entry.path())?;
            } else {
                size += metadata.len();
            }
        }
    }
    Ok(size)
}

/// Execute a command with the project's toolchain environment
pub fn execute_exec(manifest_path: &str, command: &str, args: &[String]) -> Result<()> {
    // Load project config to get toolchain requirements
    let config = config::load_config(manifest_path)?;

    // Initialize toolchain manager
    let toolchain = Toolchain::new()?;

    // Ensure toolchains are available
    let java_home = toolchain.ensure_java(&config.toolchain.java)?;
    let gradle_bin = toolchain.ensure_gradle(&config.toolchain.gradle)?;
    let android_home = toolchain.ensure_android_sdk()?;

    // Build PATH with toolchain binaries
    let mut path_components = vec![
        java_home.join("bin").display().to_string(),
        gradle_bin.parent().unwrap().display().to_string(),
        android_home.join("emulator").display().to_string(),
        android_home.join("platform-tools").display().to_string(),
        android_home.join("cmdline-tools/latest/bin").display().to_string(),
    ];

    // Add existing PATH
    if let Ok(existing_path) = std::env::var("PATH") {
        path_components.push(existing_path);
    }

    let new_path = path_components.join(":");

    // Execute command with toolchain environment
    let status = std::process::Command::new(command)
        .args(args)
        .env("JAVA_HOME", java_home)
        .env("GRADLE_HOME", gradle_bin.parent().unwrap())
        .env("ANDROID_HOME", &android_home)
        .env_remove("ANDROID_SDK_ROOT")  // Prevent conflicts with ANDROID_HOME
        .env("PATH", new_path)
        .status()
        .with_context(|| format!("Failed to execute command: {}", command))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Launch an interactive shell with the project's toolchain environment
pub fn execute_shell(manifest_path: &str) -> Result<()> {
    use colored::Colorize;

    // Load project config
    let config = config::load_config(manifest_path)?;

    // Initialize toolchain manager
    let toolchain = Toolchain::new()?;

    // Ensure toolchains are available
    let java_home = toolchain.ensure_java(&config.toolchain.java)?;
    let gradle_bin = toolchain.ensure_gradle(&config.toolchain.gradle)?;
    let android_home = toolchain.ensure_android_sdk()?;

    println!("{} toolchain environment", "Activating".green().bold());
    println!("  Java: {} ({})", config.toolchain.java, java_home.display());
    println!("  Gradle: {} ({})", config.toolchain.gradle, gradle_bin.display());
    println!("  Android SDK: {}", android_home.display());
    println!();
    println!("Commands available:");
    println!("  java --version");
    println!("  gradle --version");
    println!("  adb devices");
    println!();
    println!("Type 'exit' to return to normal shell.\n");

    // Build PATH with toolchain binaries
    let mut path_components = vec![
        java_home.join("bin").display().to_string(),
        gradle_bin.parent().unwrap().display().to_string(),
        android_home.join("emulator").display().to_string(),
        android_home.join("platform-tools").display().to_string(),
        android_home.join("cmdline-tools/latest/bin").display().to_string(),
    ];

    // Add existing PATH
    if let Ok(existing_path) = std::env::var("PATH") {
        path_components.push(existing_path);
    }

    let new_path = path_components.join(":");

    // Determine shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

    // Launch shell with toolchain environment
    let status = std::process::Command::new(&shell)
        .env("JAVA_HOME", java_home)
        .env("GRADLE_HOME", gradle_bin.parent().unwrap())
        .env("ANDROID_HOME", &android_home)
        .env_remove("ANDROID_SDK_ROOT")  // Prevent conflicts with ANDROID_HOME
        .env("PATH", new_path)
        .env("PS1", format!("(whitehall:{}) \\w $ ", config.project.name))
        .status()
        .with_context(|| format!("Failed to launch shell: {}", shell))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Format bytes as human-readable size
fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}
