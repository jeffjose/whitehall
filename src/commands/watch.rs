use anyhow::{Context, Result};
use notify::{Event, RecursiveMode, Watcher};
use std::env;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::build_pipeline;
use crate::config;

pub fn execute(manifest_path: &str) -> Result<()> {
    println!("👀 Watching Whitehall project for changes...");
    println!("   Press Ctrl+C to stop\n");

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
        env::set_current_dir(&project_dir)
            .context(format!("Failed to change to directory: {}", project_dir.display()))?;
    }

    // 2. Load configuration
    let manifest_file = manifest_path.file_name().unwrap().to_str().unwrap();
    let config = config::load_config(manifest_file)
        .context(format!("Failed to load {}. Are you in a Whitehall project directory?", manifest_file))?;

    // 3. Initial build
    println!("🔨 Initial build...");
    match run_build(&config) {
        Ok(_) => println!("✅ Ready! Watching for changes...\n"),
        Err(e) => {
            eprintln!("❌ Initial build failed: {}\n", e);
            eprintln!("Watching anyway (will retry on file changes)...\n");
        }
    }

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

    // 5. Watch loop
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                if should_rebuild(&event) {
                    // Get the changed file name for display
                    let changed_file = event
                        .paths
                        .first()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("file");

                    println!("\n📝 Change detected: {}", changed_file);

                    match run_build(&config) {
                        Ok(_) => println!("✅ Build successful\n"),
                        Err(e) => eprintln!("❌ Build failed: {}\n", e),
                    }
                }
            }
            Err(_) => {
                // Timeout, continue loop (allows Ctrl+C to work)
            }
        }
    }
}

fn run_build(config: &crate::config::Config) -> Result<()> {
    // Run build with clean=false for incremental builds
    let result = build_pipeline::execute_build(config, false)?;

    if !result.errors.is_empty() {
        for error in &result.errors {
            eprintln!("  ❌ {} - {}", error.file.display(), error.message);
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
