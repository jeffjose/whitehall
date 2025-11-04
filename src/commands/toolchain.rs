use anyhow::Result;
use colored::Colorize;

use crate::config;
use crate::toolchain::Toolchain;

/// Install toolchains required by the current project
pub fn execute_install(manifest_path: &str) -> Result<()> {
    println!("{} toolchains for current project...", "Installing".green().bold());

    // Load project config to get toolchain requirements
    let config = config::load_config(manifest_path)?;

    println!("Project: {} v{}", config.project.name, config.project.version);
    println!("Toolchain requirements:");
    println!("  - Java {}", config.toolchain.java);
    println!("  - Gradle {}", config.toolchain.gradle);
    println!("  - AGP {}", config.toolchain.agp);
    println!("  - Kotlin {}", config.toolchain.kotlin);
    println!();

    // Initialize toolchain manager
    let toolchain = Toolchain::new()?;

    // Download/verify Java
    println!("{} Java {}...", "Ensuring".cyan().bold(), config.toolchain.java);
    let java_home = toolchain.ensure_java(&config.toolchain.java)?;
    println!("  ✓ Java {} available at {}", config.toolchain.java, java_home.display());

    // Download/verify Gradle
    println!("{} Gradle {}...", "Ensuring".cyan().bold(), config.toolchain.gradle);
    let gradle_bin = toolchain.ensure_gradle(&config.toolchain.gradle)?;
    println!("  ✓ Gradle {} available at {}", config.toolchain.gradle, gradle_bin.display());

    // Download/verify Android SDK
    println!("{} Android SDK...", "Ensuring".cyan().bold());
    let android_home = toolchain.ensure_android_sdk()?;
    println!("  ✓ Android SDK available at {}", android_home.display());

    println!();
    println!("{} All toolchains ready!", "Success:".green().bold());
    println!("  Location: {}", toolchain.root().display());

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
