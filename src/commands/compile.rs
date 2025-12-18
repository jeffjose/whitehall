use anyhow::{Context, Result};
use colored::Colorize;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::{Event, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::build_pipeline;
use crate::config;
use crate::keyboard::{self, KeyAction, RawModeGuard};
use crate::transpiler;
use crate::commands::{detect_target, Target};

/// Compile a .wh file or project to Kotlin (transpile only, no APK)
pub fn execute(target: &str, package: Option<&str>, no_package: bool, watch: bool) -> Result<()> {
    // Detect if we're compiling a project or single file
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
                execute_single_file(&file_path, package, no_package)
            }
        }
    }
}

/// Compile a project (transpile only)
fn execute_project(manifest_path: &str) -> Result<()> {
    let start = Instant::now();

    // 1. Determine project directory from manifest path
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

    // 3. Run build pipeline (transpile only)
    let result = build_pipeline::execute_build(&config, true)?;

    // 4. Restore original directory if we changed it
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    // 5. Report results
    if !result.errors.is_empty() {
        eprintln!("{} compilation failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Compilation failed");
    }

    let elapsed = start.elapsed();
    println!("   {} `{}` v{} ({}) in {:.2}s",
        "Compiled".green().bold(),
        config.project.name,
        config.project.version,
        config.android.package,
        elapsed.as_secs_f64()
    );

    Ok(())
}

/// Compile a single .wh file to Kotlin (print to stdout)
fn execute_single_file(file_path: &str, package: Option<&str>, no_package: bool) -> Result<()> {
    // Validate file exists and has .wh extension
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }
    if !file_path.ends_with(".wh") {
        anyhow::bail!("File must have .wh extension: {}", file_path);
    }

    // Read source file
    let source = fs::read_to_string(path)
        .context(format!("Failed to read {}", file_path))?;

    // Strip frontmatter if present (for compile, we don't need it)
    let code = strip_frontmatter(&source);

    // Get component name from filename
    let component_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            // Capitalize first letter for component name
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .unwrap_or_else(|| "Component".to_string());

    // Determine package name
    let package_name = package.unwrap_or("com.example.app");

    // Transpile to Kotlin
    let result = transpiler::transpile(&code, package_name, &component_name, None)
        .map_err(|e| anyhow::anyhow!("Compilation error: {}", e))?;

    // Get all output files
    let files = result.files();

    // Output all Kotlin files
    for (i, (suffix, kotlin_code)) in files.iter().enumerate() {
        // Add separator between multiple files
        if i > 0 {
            println!("\n{}", "// ─────────────────────────────────────────────────────────────────".dimmed());
            let file_label = if suffix.is_empty() {
                format!("// {}.kt", component_name)
            } else {
                format!("// {}{}.kt", component_name, suffix)
            };
            println!("{}\n", file_label.dimmed());
        }

        if no_package {
            // Strip package declaration for pasting into existing files
            let code_without_package = kotlin_code
                .lines()
                .skip_while(|line| line.trim().is_empty() || line.starts_with("package "))
                .collect::<Vec<_>>()
                .join("\n");
            println!("{}", code_without_package);
        } else {
            println!("{}", kotlin_code);
        }
    }

    Ok(())
}

/// Strip frontmatter (/// comments) from source code
fn strip_frontmatter(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Skip shebang and frontmatter lines
            !trimmed.starts_with("#!") && !trimmed.starts_with("///")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================================
// Watch mode implementations
// ============================================================================

/// Watch a project for changes and recompile
fn execute_project_watch(manifest_path: &str) -> Result<()> {
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

    // Initial compile
    let start = Instant::now();
    match run_compile_watch(&config) {
        Ok(_) => print_compile_status(start.elapsed(), &config.project.name),
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
                print!("\r\n   Exiting watch mode\r\n");
                return Ok(());
            }
            KeyAction::Rebuild => {
                print!("\r\n   Rebuilding...\r\n");
                let start = Instant::now();
                match run_compile_watch(&config) {
                    Ok(_) => print_compile_status(start.elapsed(), &config.project.name),
                    Err(e) => {
                        eprint!("{} {}\r\n", "error:".red().bold(), e);
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
                match run_compile_watch(&config) {
                    Ok(_) => print_compile_status(start.elapsed(), &config.project.name),
                    Err(e) => {
                        eprint!("{} {}\r\n", "error:".red().bold(), e);
                    }
                }
                last_build = Instant::now();
            }
        }
    }
}

/// Watch a single .wh file for changes and recompile
fn execute_single_file_watch(file_path: &str) -> Result<()> {
    let file_path_buf = PathBuf::from(file_path);

    // Load gitignore from the file's directory
    let watch_dir = file_path_buf.parent().unwrap_or(Path::new("."));
    let gitignore = load_gitignore(watch_dir);

    // Get file name for display
    let file_name = file_path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path);

    // Initial compile
    let start = Instant::now();
    match run_single_file_compile_watch(&file_path_buf) {
        Ok(_) => print_compile_status(start.elapsed(), file_name),
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
                print!("\r\n   Exiting watch mode\r\n");
                return Ok(());
            }
            KeyAction::Rebuild => {
                print!("\r\n   Rebuilding...\r\n");
                let start = Instant::now();
                match run_single_file_compile_watch(&file_path_buf) {
                    Ok(_) => print_compile_status(start.elapsed(), file_name),
                    Err(e) => {
                        eprint!("{} {}\r\n", "error:".red().bold(), e);
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
                match run_single_file_compile_watch(&file_path_buf) {
                    Ok(_) => print_compile_status(start.elapsed(), file_name),
                    Err(e) => {
                        eprint!("{} {}\r\n", "error:".red().bold(), e);
                    }
                }
                last_build = Instant::now();
            }
        }
    }
}

/// Run project compilation for watch mode
fn run_compile_watch(config: &crate::config::Config) -> Result<()> {
    // Run build with clean=false for incremental builds
    let result = build_pipeline::execute_build(config, false)?;

    if !result.errors.is_empty() {
        for error in &result.errors {
            eprint!("  {} - {}\r\n", error.file.display(), error.message);
        }
        anyhow::bail!("Compilation failed with {} error(s)", result.errors.len());
    }

    Ok(())
}

/// Run single file compilation for watch mode
fn run_single_file_compile_watch(file_path: &Path) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .context(format!("Failed to read {}", file_path.display()))?;

    // Strip frontmatter
    let code = strip_frontmatter(&source);

    // Get component name from filename
    let component_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .unwrap_or_else(|| "Component".to_string());

    // Transpile to Kotlin (use default package for watch mode)
    let _result = transpiler::transpile(&code, "com.example.app", &component_name, None)
        .map_err(|e| anyhow::anyhow!("Compilation error: {}", e))?;

    Ok(())
}

/// Print compile status
fn print_compile_status(elapsed: Duration, name: &str) {
    let ms = elapsed.as_millis();
    // Use \r\n for raw mode compatibility
    print!("   {} `{}` in {}ms\r\n", "Compiled".green().bold(), name, format!("{}", ms).cyan());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let source = r#"#!/usr/bin/env whitehall
/// [app]
/// name = "Test"

var x = 5
<Text>Hello</Text>
"#;

        let result = strip_frontmatter(source);
        assert!(!result.contains("#!/"));
        assert!(!result.contains("///"));
        assert!(result.contains("var x = 5"));
        assert!(result.contains("<Text>Hello</Text>"));
    }
}
