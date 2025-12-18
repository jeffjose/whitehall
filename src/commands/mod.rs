pub mod init;
pub mod build;
pub mod watch;
pub mod run;
pub mod install;
pub mod compile;
pub mod toolchain;
pub mod emulator;
pub mod device;
pub mod doctor;
pub mod clean;
pub mod check;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;

use crate::toolchain::Toolchain;

/// Represents the type of target we're working with
#[derive(Debug, Clone)]
pub enum Target {
    /// A Whitehall project directory (contains whitehall.toml)
    Project(String), // manifest path
    /// A single .wh file
    SingleFile(String), // file path
}

/// Detect whether the target is a project directory or a single file
pub fn detect_target(target: &str) -> Target {
    let path = Path::new(target);

    // Check if it's a .wh file
    if target.ends_with(".wh") {
        return Target::SingleFile(target.to_string());
    }

    // Check if it's a directory
    if path.is_dir() {
        // Look for whitehall.toml in the directory
        let manifest_path = path.join("whitehall.toml");
        if manifest_path.exists() {
            return Target::Project(manifest_path.to_str().unwrap().to_string());
        }
        // Default to whitehall.toml if directory exists
        return Target::Project("whitehall.toml".to_string());
    }

    // If target is "." or similar, assume project mode
    if target == "." || target == "./" {
        return Target::Project("whitehall.toml".to_string());
    }

    // Check if it looks like a manifest path
    if target.ends_with("whitehall.toml") {
        return Target::Project(target.to_string());
    }

    // Default to project mode
    Target::Project(target.to_string())
}

/// Build APK with Gradle
///
/// Deletes existing APK first to force gradle to re-package,
/// fixing gradle's incremental build not detecting new dex files.
/// Also clears installed hash files to ensure reinstall after rebuild.
pub fn build_with_gradle(
    toolchain: &Toolchain,
    config: &crate::config::Config,
    output_dir: &Path,
    release: bool,
) -> Result<()> {
    // Delete existing APK to force gradle to re-package
    // This fixes gradle's incremental build not detecting new dex files
    let debug_apk = output_dir.join("app/build/outputs/apk/debug/app-debug.apk");
    let release_apk = output_dir.join("app/build/outputs/apk/release/app-release-unsigned.apk");
    let _ = fs::remove_file(&debug_apk);
    let _ = fs::remove_file(&release_apk);

    // Create a spinner to show progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.dim} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    let build_type = if release { "release" } else { "debug" };
    pb.set_message(format!("Building {} APK with Gradle...", build_type));
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut gradle = toolchain.gradle_cmd(&config.toolchain.java, &config.toolchain.gradle)?;

    let task = if release { "assembleRelease" } else { "assembleDebug" };
    let status = gradle
        .current_dir(output_dir)
        .args([task, "--console=plain", "--quiet"])
        .status()
        .context("Failed to run Gradle")?;

    // Clear the progress bar
    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!("Gradle build failed");
    }

    Ok(())
}
