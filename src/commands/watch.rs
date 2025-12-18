use anyhow::Result;
use colored::Colorize;
use notify::{Event, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::build_pipeline;
use crate::config;
use crate::keyboard::{self, KeyAction, RawModeGuard};
use crate::single_file;
use crate::commands::{detect_target, Target};

pub fn execute(target: &str) -> Result<()> {
    // Detect if we're watching a project or single file
    match detect_target(target) {
        Target::Project(manifest_path) => execute_project(&manifest_path),
        Target::SingleFile(file_path) => execute_single_file(&file_path),
    }
}

/// Watch a single .wh file
fn execute_single_file(file_path: &str) -> Result<()> {
    let file_path_buf = PathBuf::from(file_path);
    let original_dir = env::current_dir()?;

    // Initial build
    match run_single_file_build(&file_path_buf, &original_dir) {
        Ok(_) => println!("{}", "   Watching for changes...".green().bold()),
        Err(e) => {
            eprintln!("{} initial build failed: {}", "error:".red().bold(), e);
            eprintln!("Watching anyway (will retry on file changes)...");
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

    // Watch loop
    loop {
        // Check for keyboard input first
        match keyboard::poll_key(Duration::from_millis(100))? {
            KeyAction::Quit => {
                println!("\n   Exiting watch mode");
                return Ok(());
            }
            KeyAction::Rebuild => {
                println!("\n   Rebuilding...");
                match run_single_file_build(&file_path_buf, &original_dir) {
                    Ok(_) => println!("   {}", "Finished".green().bold()),
                    Err(e) => eprintln!("{} build failed: {}", "error:".red().bold(), e),
                }
                continue;
            }
            KeyAction::None => {}
        }

        // Check for file system events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            if should_rebuild(&event) {
                println!("\nChange detected in {}", file_path);

                match run_single_file_build(&file_path_buf, &original_dir) {
                    Ok(_) => println!("   {}", "Finished".green().bold()),
                    Err(e) => eprintln!("{} build failed: {}", "error:".red().bold(), e),
                }
            }
        }
    }
}

/// Build a single file (helper for watch mode)
fn run_single_file_build(file_path: &Path, original_dir: &Path) -> Result<()> {
    // Parse frontmatter
    let content = fs::read_to_string(file_path)?;
    let (single_config, code) = single_file::parse_frontmatter(&content)?;

    // Generate temporary project
    let temp_project_dir = single_file::generate_temp_project(file_path, &single_config, &code)?;

    // Change to temp project directory
    env::set_current_dir(&temp_project_dir)?;

    // Load config
    let config = config::load_config("whitehall.toml")?;

    // Run build (incremental)
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

/// Watch a project (existing behavior)
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

    // 3. Initial build
    match run_build(&config) {
        Ok(_) => println!("{}", "   Watching for changes...".green().bold()),
        Err(e) => {
            eprintln!("{} initial build failed: {}", "error:".red().bold(), e);
            eprintln!("Watching anyway (will retry on file changes)...");
        }
    }

    keyboard::print_shortcuts();

    // 4. Set up file watcher
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

    // 5. Watch loop
    loop {
        // Check for keyboard input first
        match keyboard::poll_key(Duration::from_millis(100))? {
            KeyAction::Quit => {
                println!("\n   Exiting watch mode");
                return Ok(());
            }
            KeyAction::Rebuild => {
                println!("\n   Rebuilding...");
                match run_build(&config) {
                    Ok(_) => println!("   {}", "Finished".green().bold()),
                    Err(e) => eprintln!("{} build failed: {}", "error:".red().bold(), e),
                }
                continue;
            }
            KeyAction::None => {}
        }

        // Check for file system events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            if should_rebuild(&event) {
                // Get the changed file name for display
                let changed_file = event
                    .paths
                    .first()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");

                println!("\nChange detected in {}", changed_file);

                match run_build(&config) {
                    Ok(_) => println!("   {}", "Finished".green().bold()),
                    Err(e) => eprintln!("{} build failed: {}", "error:".red().bold(), e),
                }
            }
        }
    }
}

fn run_build(config: &crate::config::Config) -> Result<()> {
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

fn should_rebuild(event: &notify::Event) -> bool {
    use notify::EventKind::*;

    match event.kind {
        Modify(_) | Create(_) | Remove(_) => {
            // Check if any path is a .wh file or whitehall.toml
            event.paths.iter().any(|p| {
                p.extension().map_or(false, |ext| ext == "wh")
                    || p.file_name().map_or(false, |name| name == "whitehall.toml")
            })
        }
        _ => false,
    }
}
