use anyhow::{Context, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::path::Path;

use crate::build_pipeline;
use crate::config;
use crate::single_file;
use crate::commands::{detect_target, Target};

pub fn execute(target: &str) -> Result<()> {
    // Detect if we're building a project or single file
    match detect_target(target) {
        Target::Project(manifest_path) => execute_project(&manifest_path),
        Target::SingleFile(file_path) => execute_single_file(&file_path),
    }
}

/// Build a single .wh file
fn execute_single_file(file_path: &str) -> Result<()> {
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

    // Restore original directory
    env::set_current_dir(&original_dir)?;

    // Report results
    if !result.errors.is_empty() {
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    println!("{}", format!("   Finished transpiling {} file(s) to {}/build", result.files_transpiled, temp_project_dir.display()).green().bold());

    Ok(())
}

/// Build a project (existing behavior)
fn execute_project(manifest_path: &str) -> Result<()> {
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

    // 4. Restore original directory if we changed it
    if project_dir != original_dir {
        env::set_current_dir(&original_dir)?;
    }

    // 5. Report results
    if !result.errors.is_empty() {
        eprintln!("{} build failed with {} error(s)", "error:".red().bold(), result.errors.len());
        for error in &result.errors {
            eprintln!("  {} - {}", error.file.display(), error.message);
        }
        anyhow::bail!("Build failed");
    }

    // Make the output path relative to where the user ran the command
    let output_path = project_dir.join(&result.output_dir);
    let display_path = if output_path.starts_with(&original_dir) {
        output_path.strip_prefix(&original_dir).unwrap().to_path_buf()
    } else {
        output_path
    };

    println!("{}", format!("   Finished transpiling {} file(s) to {}", result.files_transpiled, display_path.display()).green().bold());

    Ok(())
}
