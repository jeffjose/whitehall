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
use crate::toolchain::Toolchain;

pub fn execute(target: &str) -> Result<()> {
    // Detect if we're running a project or single file
    match detect_target(target) {
        Target::Project(manifest_path) => execute_project(&manifest_path),
        Target::SingleFile(file_path) => execute_single_file(&file_path),
    }
}

/// Run a single .wh file
fn execute_single_file(file_path: &str) -> Result<()> {
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

    // Continue with device check, gradle, install, and launch
    check_device_connected(&toolchain)?;
    build_with_gradle(&toolchain, &config, &result.output_dir)?;
    install_apk(&toolchain, &result.output_dir)?;
    launch_app(&toolchain, &config.android.package)?;

    println!("{}", format!("    Running on device").green().bold());
    println!();

    // Stream logcat
    stream_logcat(&toolchain, &config.android.package)?;

    // Restore original directory
    env::set_current_dir(&original_dir)?;

    Ok(())
}

/// Run a project (existing behavior)
fn execute_project(manifest_path: &str) -> Result<()> {
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

    // 3.5. Ensure all toolchains are ready (download in parallel if needed)
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // 4. Check if device/emulator is connected
    check_device_connected(&toolchain)?;

    // 5. Build APK with Gradle
    build_with_gradle(&toolchain, &config, &result.output_dir)?;

    // 6. Install on device
    install_apk(&toolchain, &result.output_dir)?;

    // 7. Launch app
    launch_app(&toolchain, &config.android.package)?;

    println!("{}", format!("    Running on device").green().bold());
    println!();

    // 8. Stream logcat filtered to this app
    stream_logcat(&toolchain, &config.android.package)?;

    // Restore original directory
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    Ok(())
}

fn check_device_connected(toolchain: &Toolchain) -> Result<()> {
    let output = toolchain
        .adb_cmd()?
        .args(&["devices"])
        .output()
        .context("Failed to run 'adb devices'")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let device_count = stdout
        .lines()
        .skip(1) // Skip header
        .filter(|line| line.contains("device") && !line.contains("offline"))
        .count();

    if device_count == 0 {
        anyhow::bail!(
            "No devices connected. Please:\n  \
            1. Connect a device via USB with USB debugging enabled, or\n  \
            2. Start an emulator"
        );
    }

    println!("   Found {} device(s)", device_count);
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
        .args(&["assembleDebug", "--console=plain", "--quiet"])
        .status()
        .context("Failed to run Gradle")?;

    // Clear the progress bar (it disappears)
    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!("Gradle build failed");
    }

    Ok(())
}

fn install_apk(toolchain: &Toolchain, output_dir: &Path) -> Result<()> {
    let apk_path = output_dir.join("app/build/outputs/apk/debug/app-debug.apk");

    if !apk_path.exists() {
        anyhow::bail!("APK not found at {}", apk_path.display());
    }

    let status = toolchain
        .adb_cmd()?
        .args(&["install", "-r", apk_path.to_str().unwrap()])
        .status()
        .context("Failed to install APK")?;

    if !status.success() {
        anyhow::bail!("APK installation failed");
    }

    Ok(())
}

fn launch_app(toolchain: &Toolchain, package: &str) -> Result<()> {
    let activity = format!("{}/.MainActivity", package);

    let status = toolchain
        .adb_cmd()?
        .args(&["shell", "am", "start", "-n", &activity])
        .status()
        .context("Failed to launch app")?;

    if !status.success() {
        anyhow::bail!("App launch failed");
    }

    Ok(())
}

fn stream_logcat(toolchain: &Toolchain, package: &str) -> Result<()> {
    use std::io::{BufRead, BufReader};

    println!("{} (press Ctrl+C to stop)", "Streaming logcat".cyan().bold());
    println!("{}", "─".repeat(80).dimmed());

    // Clear logcat first to only show new logs
    let _ = toolchain
        .adb_cmd()?
        .args(&["logcat", "-c"])
        .output();

    // Stream logcat with brief format - we'll filter in Rust
    let mut child = toolchain
        .adb_cmd()?
        .args(&["logcat", "-v", "brief"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("Failed to start logcat")?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Filter: show line if it contains the package name OR mentions the app
                    let relevant = line.contains(package)
                        || line.contains("AndroidRuntime")  // Runtime errors/crashes
                        || (line.contains("ActivityManager") && line.contains(package));

                    if !relevant {
                        continue;
                    }

                    // Skip noisy logs that aren't helpful for debugging
                    if line.contains("AiAiEcho")
                        || line.contains("PackageUpdatedTask")
                        || line.contains("ApkAssets")
                        || line.contains("ziparchive")
                        || line.contains("nativeloader")
                        || line.contains("ProximityAuth")
                        || line.contains("SQLiteLog")
                        || (line.contains("ActivityThread") && line.contains("REPLACED"))
                        || line.contains("Finsky")
                        || line.contains("InputManager-JNI")
                        || line.contains("CoreBackPreview")
                    {
                        continue;
                    }

                    // Color code based on log level
                    if line.contains(" E ") || line.contains("ERROR") || line.contains("FATAL") {
                        println!("{}", line.red());
                    } else if line.contains(" W ") || line.contains("WARN") {
                        println!("{}", line.yellow());
                    } else if line.contains(" I ") || line.contains("INFO") {
                        println!("{}", line.white());
                    } else if line.contains(" D ") || line.contains("DEBUG") {
                        println!("{}", line.dimmed());
                    } else if line.contains(" V ") || line.contains("VERBOSE") {
                        println!("{}", line.dimmed());
                    } else {
                        println!("{}", line);
                    }
                }
                Err(_) => break,
            }
        }
    }

    // Kill the logcat process when we exit
    let _ = child.kill();

    Ok(())
}
