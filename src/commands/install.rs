use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::fs;
use std::path::Path;

use crate::build_pipeline;
use crate::config;
use crate::single_file;
use crate::commands::{detect_target, Target};
use crate::commands::device;
use crate::toolchain::Toolchain;

pub fn execute(target: &str, device_query: Option<&str>) -> Result<()> {
    // Detect if we're running a project or single file
    match detect_target(target) {
        Target::Project(manifest_path) => execute_project(&manifest_path, device_query),
        Target::SingleFile(file_path) => execute_single_file(&file_path, device_query),
    }
}

/// Install a single .wh file
fn execute_single_file(file_path: &str, device_query: Option<&str>) -> Result<()> {
    // Parse frontmatter
    let file_path_obj = Path::new(file_path);
    let content = fs::read_to_string(file_path_obj)
        .context(format!("Failed to read {}", file_path_obj.display()))?;

    let (single_config, code) = single_file::parse_frontmatter(&content)
        .context("Failed to parse frontmatter")?;

    // Generate temporary project
    let temp_project_dir = single_file::generate_temp_project(file_path_obj, &single_config, &code)
        .context("Failed to generate temporary project")?;

    // Change to temp project directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(&temp_project_dir)
        .context("Failed to change to temp project directory")?;

    // Load config from generated whitehall.toml
    let config = config::load_config("whitehall.toml")
        .context("Failed to load generated whitehall.toml")?;

    // Build project
    let result = build_pipeline::execute_build(&config, true)?;

    if !result.errors.is_empty() {
        env::set_current_dir(&original_dir)?;
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("   {} `{}` ({})",
        "Finished".green().bold(),
        single_config.app.name,
        single_config.app.package
    );

    // Initialize toolchain manager
    let toolchain = Toolchain::new()?;

    // Ensure all toolchains are ready (download in parallel if needed)
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // Resolve device
    let device_id = device::resolve_device(&toolchain, device_query)?;
    println!("   {} {}", "Device".cyan(), device_id);

    // Build APK and install
    build_with_gradle(&toolchain, &config, &result.output_dir)?;
    install_apk(&toolchain, &result.output_dir, &device_id)?;

    println!(
        "  {} `{}` on {}",
        "Installed".green().bold(),
        single_config.app.name,
        device_id
    );

    // Restore original directory
    env::set_current_dir(&original_dir)?;

    Ok(())
}

/// Install a project (existing behavior)
fn execute_project(manifest_path: &str, device_query: Option<&str>) -> Result<()> {
    // 1. Determine project directory from manifest path (same as build command)
    let manifest_path = Path::new(manifest_path);
    let original_dir = env::current_dir()?;

    let project_dir = if manifest_path == Path::new("whitehall.toml") {
        original_dir.clone()
    } else {
        let dir = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        if dir.is_relative() {
            original_dir.join(dir)
        } else {
            dir
        }
    };

    // Change to project directory if needed
    if project_dir != original_dir {
        env::set_current_dir(&project_dir)?;
    }

    // 2. Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    // 3. Build project
    let result = build_pipeline::execute_build(&config, true)?;

    if !result.errors.is_empty() {
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("   {} `{}` v{} ({})",
        "Finished".green().bold(),
        config.project.name,
        config.project.version,
        config.android.package
    );

    // Initialize toolchain manager
    let toolchain = Toolchain::new()?;

    // Ensure all toolchains are ready (download in parallel if needed)
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // Resolve device
    let device_id = device::resolve_device(&toolchain, device_query)?;
    println!("   {} {}", "Device".cyan(), device_id);

    // Build APK and install
    build_with_gradle(&toolchain, &config, &result.output_dir)?;
    install_apk(&toolchain, &result.output_dir, &device_id)?;

    println!(
        "  {} `{}` on {}",
        "Installed".green().bold(),
        config.project.name,
        device_id
    );

    // Restore original directory
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    Ok(())
}

fn build_with_gradle(toolchain: &Toolchain, config: &crate::config::Config, output_dir: &Path) -> Result<()> {
    // Create a spinner to show progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.dim} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    pb.set_message("Building APK with Gradle...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut gradle = toolchain.gradle_cmd(&config.toolchain.java, &config.toolchain.gradle)?;

    let status = gradle
        .current_dir(output_dir)
        .args(["assembleDebug", "--console=plain", "--quiet"])
        .status()
        .context("Failed to run Gradle")?;

    // Clear the progress bar (it disappears)
    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!("Gradle build failed");
    }

    Ok(())
}

fn install_apk(toolchain: &Toolchain, output_dir: &Path, device_id: &str) -> Result<()> {
    let apk_path = output_dir.join("app/build/outputs/apk/debug/app-debug.apk");

    if !apk_path.exists() {
        anyhow::bail!("APK not found at {}", apk_path.display());
    }

    let status = toolchain
        .adb_cmd()?
        .args(["-s", device_id, "install", "-r", apk_path.to_str().unwrap()])
        .status()
        .context("Failed to install APK")?;

    if !status.success() {
        anyhow::bail!("APK installation failed");
    }

    Ok(())
}
