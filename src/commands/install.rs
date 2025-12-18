use anyhow::{Context, Result};
use colored::Colorize;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use notify::{Event, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::build_pipeline;
use crate::config;
use crate::keyboard::{self, KeyAction, RawModeGuard};
use crate::single_file;
use crate::commands::{detect_target, Target};
use crate::commands::device;
use crate::toolchain::Toolchain;

pub fn execute(target: &str, device_query: Option<&str>, watch: bool) -> Result<()> {
    // Smart argument detection:
    // If target doesn't exist as a file/dir, but whitehall.toml exists in current dir,
    // treat target as device_query instead
    let path = Path::new(target);
    let is_current_dir = target == "." || target == "./";
    let target_exists = path.exists() || target.ends_with(".wh") || target.ends_with("whitehall.toml");
    let has_local_project = Path::new("whitehall.toml").exists();

    let (actual_target, actual_device) = if !is_current_dir && !target_exists && has_local_project {
        // Target doesn't exist but we have a local project - treat target as device
        (".", Some(target))
    } else {
        (target, device_query)
    };

    // Detect if we're running a project or single file
    match detect_target(actual_target) {
        Target::Project(manifest_path) => {
            if watch {
                execute_project_watch(&manifest_path, actual_device)
            } else {
                execute_project(&manifest_path, actual_device)
            }
        }
        Target::SingleFile(file_path) => {
            if watch {
                execute_single_file_watch(&file_path, actual_device)
            } else {
                execute_single_file(&file_path, actual_device)
            }
        }
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
    let device = device::resolve_device(&toolchain, device_query)?;
    println!("    {} {}", "Device".cyan(), device.display_name());

    // Build APK and install
    build_with_gradle(&toolchain, &config, &result.output_dir)?;
    install_apk(&toolchain, &result.output_dir, &device.id)?;

    println!(
        "  {} `{}` on {}",
        "Installed".green().bold(),
        single_config.app.name,
        device.short_name()
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
    let device = device::resolve_device(&toolchain, device_query)?;
    println!("    {} {}", "Device".cyan(), device.display_name());

    // Build APK and install
    build_with_gradle(&toolchain, &config, &result.output_dir)?;
    install_apk(&toolchain, &result.output_dir, &device.id)?;

    println!(
        "  {} `{}` on {}",
        "Installed".green().bold(),
        config.project.name,
        device.short_name()
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

// ============================================================================
// Watch mode implementations
// ============================================================================

/// Watch and install a single .wh file
fn execute_single_file_watch(file_path: &str, device_query: Option<&str>) -> Result<()> {
    let file_path_buf = PathBuf::from(file_path);
    let file_path_obj = Path::new(file_path);
    let original_dir = env::current_dir()?;

    // Load gitignore from the file's directory
    let watch_dir = file_path_buf.parent().unwrap_or(Path::new("."));
    let gitignore = load_gitignore(watch_dir);

    // Parse frontmatter for initial build
    let content = fs::read_to_string(file_path_obj)
        .context(format!("Failed to read {}", file_path_obj.display()))?;
    let (single_config, code) = single_file::parse_frontmatter(&content)?;

    // Generate temporary project
    let temp_project_dir = single_file::generate_temp_project(file_path_obj, &single_config, &code)?;

    // Change to temp project directory
    env::set_current_dir(&temp_project_dir)?;

    // Load config
    let config = config::load_config("whitehall.toml")?;

    // Initialize toolchain
    let toolchain = Toolchain::new()?;
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // Resolve device once at start
    let device = device::resolve_device(&toolchain, device_query)?;
    println!("    {} {}", "Device".cyan(), device.display_name());

    // Initial build and install
    let start = Instant::now();
    match install_cycle(&toolchain, &config, &device.id) {
        Ok(_) => {
            print_install_status(&single_config.app.name, &device.id, start.elapsed());
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
        }
    }

    // Restore to original directory for watching
    env::set_current_dir(&original_dir)?;

    println!("\n{}", "Watching for changes...".cyan().bold());
    keyboard::print_shortcuts();

    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    // Watch the single .wh file
    watcher.watch(&file_path_buf, RecursiveMode::NonRecursive)?;

    // Enable raw mode for keyboard input
    let _raw_guard = RawModeGuard::new()?;

    // Helper closure to run a full install cycle
    let run_reinstall = || -> Result<()> {
        // Re-read and regenerate project
        let content = fs::read_to_string(&file_path_buf)?;
        let (single_config, code) = single_file::parse_frontmatter(&content)?;
        let temp_project_dir = single_file::generate_temp_project(&file_path_buf, &single_config, &code)?;

        env::set_current_dir(&temp_project_dir)?;
        let config = config::load_config("whitehall.toml")?;

        print!("{}\r\n", "─".repeat(60).dimmed());
        let start = Instant::now();
        match install_cycle(&toolchain, &config, &device.id) {
            Ok(_) => {
                print_install_status(&single_config.app.name, &device.id, start.elapsed());
            }
            Err(e) => {
                eprint!("{} {}\r\n", "error:".red().bold(), e);
            }
        }

        env::set_current_dir(&original_dir)?;
        Ok(())
    };

    // Watch loop with debouncing
    let mut last_build = Instant::now();
    loop {
        // Check for keyboard input first
        match keyboard::poll_key(Duration::from_millis(100))? {
            KeyAction::Quit => {
                print!("\r\n   Exiting watch mode\r\n");
                return Ok(());
            }
            KeyAction::Rebuild => {
                print!("\r\n   Rebuilding...\r\n");
                run_reinstall()?;
                last_build = Instant::now();
                continue;
            }
            KeyAction::None => {}
        }

        // Check for file system events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            if should_rebuild(&event, &gitignore) {
                // Debounce
                if last_build.elapsed() < Duration::from_millis(100) {
                    continue;
                }
                while rx.try_recv().is_ok() {}

                run_reinstall()?;
                last_build = Instant::now();
            }
        }
    }
}

/// Watch and install a project
fn execute_project_watch(manifest_path: &str, device_query: Option<&str>) -> Result<()> {
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

    if project_dir != original_dir {
        env::set_current_dir(&project_dir)?;
    }

    // Load gitignore
    let gitignore = load_gitignore(&env::current_dir()?);

    // Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    // Initialize toolchain
    let toolchain = Toolchain::new()?;
    toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

    // Resolve device once at start
    let device = device::resolve_device(&toolchain, device_query)?;
    println!("    {} {}", "Device".cyan(), device.display_name());

    // Initial build and install
    let start = Instant::now();
    match install_cycle(&toolchain, &config, &device.id) {
        Ok(_) => {
            print_install_status(&config.project.name, &device.id, start.elapsed());
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
        }
    }

    println!("\n{}", "Watching for changes...".cyan().bold());
    keyboard::print_shortcuts();

    // Set up file watcher
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    // Watch src/ directory and whitehall.toml
    watcher.watch(Path::new("src"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new(manifest_file), RecursiveMode::NonRecursive)?;

    // Enable raw mode for keyboard input
    let _raw_guard = RawModeGuard::new()?;

    // Helper closure to run a full install cycle
    let run_reinstall = || {
        print!("{}\r\n", "─".repeat(60).dimmed());
        let start = Instant::now();
        match install_cycle(&toolchain, &config, &device.id) {
            Ok(_) => {
                print_install_status(&config.project.name, &device.id, start.elapsed());
            }
            Err(e) => {
                eprint!("{} {}\r\n", "error:".red().bold(), e);
            }
        }
    };

    // Watch loop with debouncing
    let mut last_build = Instant::now();
    loop {
        // Check for keyboard input first
        match keyboard::poll_key(Duration::from_millis(100))? {
            KeyAction::Quit => {
                print!("\r\n   Exiting watch mode\r\n");
                return Ok(());
            }
            KeyAction::Rebuild => {
                print!("\r\n   Rebuilding...\r\n");
                run_reinstall();
                last_build = Instant::now();
                continue;
            }
            KeyAction::None => {}
        }

        // Check for file system events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            if should_rebuild(&event, &gitignore) {
                // Debounce
                if last_build.elapsed() < Duration::from_millis(100) {
                    continue;
                }
                while rx.try_recv().is_ok() {}

                run_reinstall();
                last_build = Instant::now();
            }
        }
    }
}

/// Run install cycle: transpile, gradle, install (no launch)
fn install_cycle(
    toolchain: &Toolchain,
    config: &crate::config::Config,
    device_id: &str,
) -> Result<()> {
    // 1. Run transpilation (incremental)
    let result = build_pipeline::execute_build(config, false)?;

    if !result.errors.is_empty() {
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed with {} error(s)", result.errors.len());
    }

    // 2. Build APK with Gradle
    build_with_gradle(toolchain, config, &result.output_dir)?;

    // 3. Install APK
    install_apk(toolchain, &result.output_dir, device_id)?;

    Ok(())
}

/// Print install status
fn print_install_status(name: &str, _device_id: &str, elapsed: Duration) {
    let ms = elapsed.as_millis();
    // Use \r\n for raw mode compatibility
    print!("  {} `{}` in {}ms\r\n", "Installed".green().bold(), name, format!("{}", ms).cyan());
}

/// Load gitignore from the directory if it exists
fn load_gitignore(dir: &Path) -> Gitignore {
    let gitignore_path = dir.join(".gitignore");
    let mut builder = GitignoreBuilder::new(dir);

    if gitignore_path.exists() {
        let _ = builder.add(&gitignore_path);
    }

    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

/// Check if an event should trigger a rebuild
fn should_rebuild(event: &notify::Event, gitignore: &Gitignore) -> bool {
    use notify::EventKind::*;

    match event.kind {
        Modify(_) | Create(_) | Remove(_) => {
            event.paths.iter().any(|p| {
                let is_relevant = p.extension().map_or(false, |ext| ext == "wh")
                    || p.file_name().map_or(false, |name| name == "whitehall.toml");

                if !is_relevant {
                    return false;
                }

                !gitignore.matched(p, p.is_dir()).is_ignore()
            })
        }
        _ => false,
    }
}
