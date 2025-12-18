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
use crate::toolchain::Toolchain;

pub fn execute(target: &str, watch: bool) -> Result<()> {
    // Detect if we're building a project or single file
    match detect_target(target) {
        Target::Project(manifest_path) => {
            if watch {
                execute_project_watch(&manifest_path)
            } else {
                execute_project(&manifest_path)
            }
        }
        Target::SingleFile(file_path) => {
            if watch {
                execute_single_file_watch(&file_path)
            } else {
                execute_single_file(&file_path)
            }
        }
    }
}

/// Build a single .wh file
fn execute_single_file(file_path: &str) -> Result<()> {
    let start = Instant::now();

    // Parse frontmatter
    let file_path = Path::new(file_path);
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("failed to read `{}`: {}", file_path.display(), e))?;

    let (single_config, code) = single_file::parse_frontmatter(&content)?;

    // Generate temporary project
    let temp_project_dir = single_file::generate_temp_project(file_path, &single_config, &code)?;

    // Change to temp project directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(&temp_project_dir)?;

    // Load config from generated whitehall.toml
    let config = config::load_config("whitehall.toml")?;

    // Run build pipeline
    let result = build_pipeline::execute_build(&config, true)?;

    // Report results
    if !result.errors.is_empty() {
        env::set_current_dir(&original_dir)?;
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("   {} transpilation for `{}`",
        "Finished".green().bold(),
        single_config.app.name
    );

    // Build APK (skip if FFI-only mode)
    if !config.ffi.ffi_only {
        // Initialize toolchain and build APK
        let toolchain = Toolchain::new()?;
        toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

        build_with_gradle(&toolchain, &config, &result.output_dir)?;

        let elapsed = start.elapsed();
        let apk_path = result.output_dir.join("app/build/outputs/apk/debug/app-debug.apk");
        println!("   {} APK for `{}` v{} ({}) in {:.2}s",
            "Built".green().bold(),
            single_config.app.name,
            config.project.version,
            single_config.app.package,
            elapsed.as_secs_f64()
        );
        println!("        {} {}", "APK:".dimmed(), apk_path.display());
    } else {
        let elapsed = start.elapsed();
        println!("   {} FFI code generation for `{}` in {:.2}s (FFI-only mode)",
            "Completed".green().bold(),
            single_config.app.name,
            elapsed.as_secs_f64()
        );
    }

    // Restore original directory
    env::set_current_dir(&original_dir)?;

    Ok(())
}

/// Build a project (existing behavior)
fn execute_project(manifest_path: &str) -> Result<()> {
    let start = Instant::now();

    // 1. Determine project directory from manifest path
    let manifest_path = Path::new(manifest_path);
    let original_dir = env::current_dir()?;

    // If manifest_path is just "whitehall.toml" (default), use cwd
    // Otherwise, change to the directory containing the manifest
    let project_dir = if manifest_path == Path::new("whitehall.toml") {
        original_dir.clone()
    } else {
        let dir = manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        // Make it absolute
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

    // 3. Run build pipeline (with clean)
    let result = build_pipeline::execute_build(&config, true)?;

    // 4. Report results
    if !result.errors.is_empty() {
        if project_dir != original_dir {
            env::set_current_dir(&original_dir)?;
        }
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("   {} transpilation for `{}`",
        "Finished".green().bold(),
        config.project.name
    );

    // 5. Build APK (skip if FFI-only mode)
    if !config.ffi.ffi_only {
        // Initialize toolchain and build APK
        let toolchain = Toolchain::new()?;
        toolchain.ensure_all_for_build(&config.toolchain.java, &config.toolchain.gradle)?;

        build_with_gradle(&toolchain, &config, &result.output_dir)?;

        let elapsed = start.elapsed();
        let apk_path = result.output_dir.join("app/build/outputs/apk/debug/app-debug.apk");
        println!("   {} APK for `{}` v{} ({}) in {:.2}s",
            "Built".green().bold(),
            config.project.name,
            config.project.version,
            config.android.package,
            elapsed.as_secs_f64()
        );
        println!("        {} {}", "APK:".dimmed(), apk_path.display());
    } else {
        let elapsed = start.elapsed();
        println!("   {} FFI code generation for `{}` in {:.2}s (FFI-only mode)",
            "Completed".green().bold(),
            config.project.name,
            elapsed.as_secs_f64()
        );
    }

    // 6. Restore original directory if we changed it
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

// ============================================================================
// Watch mode implementations
// ============================================================================

/// Watch a single .wh file for changes and rebuild
fn execute_single_file_watch(file_path: &str) -> Result<()> {
    let file_path_buf = PathBuf::from(file_path);
    let original_dir = env::current_dir()?;

    // Load gitignore from the file's directory
    let watch_dir = file_path_buf.parent().unwrap_or(Path::new("."));
    let gitignore = load_gitignore(watch_dir);

    // Initial build
    let start = Instant::now();
    match run_single_file_build_watch(&file_path_buf, &original_dir) {
        Ok(_) => print_build_status(start.elapsed(), true),
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
        }
    }

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

    // Watch loop with debouncing
    let mut last_build = Instant::now();
    loop {
        // Check for keyboard input first
        match keyboard::poll_key(Duration::from_millis(100))? {
            KeyAction::Quit => {
                println!("\n   Exiting watch mode");
                return Ok(());
            }
            KeyAction::Rebuild => {
                println!("\n   Rebuilding...");
                let start = Instant::now();
                match run_single_file_build_watch(&file_path_buf, &original_dir) {
                    Ok(_) => print_build_status(start.elapsed(), true),
                    Err(e) => {
                        eprintln!("{} {}", "error:".red().bold(), e);
                    }
                }
                last_build = Instant::now();
                continue;
            }
            KeyAction::None => {}
        }

        // Check for file system events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            if should_rebuild(&event, &gitignore) {
                // Debounce: skip if we just built within 100ms
                if last_build.elapsed() < Duration::from_millis(100) {
                    continue;
                }
                // Drain any additional pending events
                while rx.try_recv().is_ok() {}

                let start = Instant::now();
                match run_single_file_build_watch(&file_path_buf, &original_dir) {
                    Ok(_) => print_build_status(start.elapsed(), true),
                    Err(e) => {
                        eprintln!("{} {}", "error:".red().bold(), e);
                    }
                }
                last_build = Instant::now();
            }
        }
    }
}

/// Build a single file for watch mode (transpilation only)
fn run_single_file_build_watch(file_path: &Path, original_dir: &Path) -> Result<()> {
    // Parse frontmatter
    let content = fs::read_to_string(file_path)?;
    let (single_config, code) = single_file::parse_frontmatter(&content)?;

    // Generate temporary project
    let temp_project_dir = single_file::generate_temp_project(file_path, &single_config, &code)?;

    // Change to temp project directory
    env::set_current_dir(&temp_project_dir)?;

    // Load config
    let config = config::load_config("whitehall.toml")?;

    // Run build (incremental, transpilation only)
    let result = build_pipeline::execute_build(&config, false)?;

    // Restore directory
    env::set_current_dir(original_dir)?;

    if !result.errors.is_empty() {
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed with {} error(s)", result.errors.len());
    }

    Ok(())
}

/// Watch a project for changes and rebuild
fn execute_project_watch(manifest_path: &str) -> Result<()> {
    // Determine project directory from manifest path
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

    // Load gitignore from project directory
    let gitignore = load_gitignore(&env::current_dir()?);

    // Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)?;

    // Initial build
    let start = Instant::now();
    match run_build_watch(&config) {
        Ok(_) => print_build_status(start.elapsed(), true),
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
        }
    }

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

    // Watch loop with debouncing
    let mut last_build = Instant::now();
    loop {
        // Check for keyboard input first
        match keyboard::poll_key(Duration::from_millis(100))? {
            KeyAction::Quit => {
                println!("\n   Exiting watch mode");
                return Ok(());
            }
            KeyAction::Rebuild => {
                println!("\n   Rebuilding...");
                let start = Instant::now();
                match run_build_watch(&config) {
                    Ok(_) => print_build_status(start.elapsed(), true),
                    Err(e) => {
                        eprintln!("{} {}", "error:".red().bold(), e);
                    }
                }
                last_build = Instant::now();
                continue;
            }
            KeyAction::None => {}
        }

        // Check for file system events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            if should_rebuild(&event, &gitignore) {
                // Debounce: skip if we just built within 100ms
                if last_build.elapsed() < Duration::from_millis(100) {
                    continue;
                }
                // Drain any additional pending events
                while rx.try_recv().is_ok() {}

                let start = Instant::now();
                match run_build_watch(&config) {
                    Ok(_) => print_build_status(start.elapsed(), true),
                    Err(e) => {
                        eprintln!("{} {}", "error:".red().bold(), e);
                    }
                }
                last_build = Instant::now();
            }
        }
    }
}

/// Run transpilation build for watch mode
fn run_build_watch(config: &crate::config::Config) -> Result<()> {
    // Run build with clean=false for incremental builds
    let result = build_pipeline::execute_build(config, false)?;

    if !result.errors.is_empty() {
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed with {} error(s)", result.errors.len());
    }

    Ok(())
}

/// Print build status on a new line
fn print_build_status(elapsed: Duration, _success: bool) {
    let ms = elapsed.as_millis();
    println!("   {} transpilation in {}ms", "Finished".green().bold(), format!("{}", ms).cyan());
}

/// Load gitignore from the current directory if it exists
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
            // Check if any path is a .wh file or whitehall.toml
            // AND is not ignored by gitignore
            event.paths.iter().any(|p| {
                let is_relevant = p.extension().map_or(false, |ext| ext == "wh")
                    || p.file_name().map_or(false, |name| name == "whitehall.toml");

                if !is_relevant {
                    return false;
                }

                // Check if path is ignored by gitignore
                // matched() returns Match which has is_ignore() method
                !gitignore.matched(p, p.is_dir()).is_ignore()
            })
        }
        _ => false,
    }
}
